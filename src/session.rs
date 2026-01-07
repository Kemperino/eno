use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::cli::Tool;
use crate::error::Result;

const STATE_FILE: &str = "state.json";
const SESSIONS_DIR: &str = "eno-sessions";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub id: String,
    pub repo: PathBuf,
    pub base_ref: String,
    pub created_at: DateTime<Utc>,
    pub agents: Vec<AgentState>,
    pub state_dir: PathBuf,
    pub tmux_session: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: usize,
    pub tool: Tool,
    pub task: String,
    pub branch: String,
    pub worktree: PathBuf,
    pub port_base: u16,
}

impl SessionState {
    /// Create a new session state
    pub fn new(
        id: String,
        repo: PathBuf,
        base_ref: String,
        agents: Vec<AgentState>,
    ) -> Result<Self> {
        let state_dir = Self::sessions_base_dir()?.join(&id);
        fs::create_dir_all(&state_dir)?;

        let tmux_session = format!("eno-{}", &id);

        let state = Self {
            id,
            repo,
            base_ref,
            created_at: Utc::now(),
            agents,
            state_dir,
            tmux_session,
        };

        state.save()?;
        Ok(state)
    }

    /// Get the base directory for all sessions
    fn sessions_base_dir() -> Result<PathBuf> {
        let base = std::env::temp_dir().join(SESSIONS_DIR);
        fs::create_dir_all(&base)?;
        Ok(base)
    }

    /// Save the session state to disk
    pub fn save(&self) -> Result<()> {
        let path = self.state_dir.join(STATE_FILE);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load a session state from a state directory
    pub fn load(state_dir: &Path) -> Result<Self> {
        let path = state_dir.join(STATE_FILE);
        let content = fs::read_to_string(path)?;
        let state: SessionState = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// Find the active session (if any)
    pub fn find_active() -> Result<Option<Self>> {
        let base_dir = Self::sessions_base_dir()?;

        // Look for session directories
        let entries = fs::read_dir(&base_dir)?;
        let mut sessions: Vec<SessionState> = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(state) = Self::load(&path) {
                    // Verify the tmux session still exists
                    if crate::tmux::TmuxManager::new(state.tmux_session.clone())
                        .map(|tm| tm.session_exists())
                        .unwrap_or(false)
                    {
                        sessions.push(state);
                    }
                }
            }
        }

        // Return the most recently created session
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions.into_iter().next())
    }

    /// Clean up this session's resources
    pub fn cleanup(&self) -> Result<()> {
        // Remove the state directory
        if self.state_dir.exists() {
            fs::remove_dir_all(&self.state_dir)?;
        }
        Ok(())
    }

    /// Get an agent by ID
    pub fn get_agent(&self, id: usize) -> Option<&AgentState> {
        self.agents.iter().find(|a| a.id == id)
    }

    /// Generate a unique session ID
    pub fn generate_id() -> String {
        let now = Utc::now();
        format!("{}", now.format("%Y%m%d-%H%M%S"))
    }
}

impl AgentState {
    pub fn new(
        id: usize,
        tool: Tool,
        task: String,
        branch: String,
        worktree: PathBuf,
        port_base: u16,
    ) -> Self {
        Self {
            id,
            tool,
            task,
            branch,
            worktree,
            port_base,
        }
    }

    /// Get a display name for the agent
    pub fn display_name(&self) -> String {
        format!("agent-{}-{}", self.id, self.tool)
    }
}
