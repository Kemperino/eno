use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::cli::Tool;
use crate::error::{EnoError, Result};

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
        fs::create_dir_all(state_dir.join("locks"))?;

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

    /// Get the locks directory
    pub fn locks_dir(&self) -> PathBuf {
        self.state_dir.join("locks")
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

/// Lock file management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub resource: String,
    pub agent_id: usize,
    pub acquired_at: DateTime<Utc>,
    pub pid: u32,
}

impl LockInfo {
    pub fn new(resource: String, agent_id: usize) -> Self {
        Self {
            resource,
            agent_id,
            acquired_at: Utc::now(),
            pid: std::process::id(),
        }
    }

    /// Acquire a lock
    pub fn acquire(locks_dir: &Path, resource: &str, agent_id: usize, timeout_secs: u64) -> Result<Self> {
        use std::time::{Duration, Instant};
        use fs2::FileExt;

        let lock_file = locks_dir.join(format!("{}.lock", resource));
        let info_file = locks_dir.join(format!("{}.json", resource));

        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            // Try to create/open the lock file
            let file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&lock_file)?;

            // Try to get an exclusive lock
            if file.try_lock_exclusive().is_ok() {
                // We got the lock, write the info file
                let info = LockInfo::new(resource.to_string(), agent_id);
                let content = serde_json::to_string_pretty(&info)?;
                fs::write(&info_file, content)?;
                return Ok(info);
            }

            // Check if we've timed out
            if start.elapsed() >= timeout {
                // Read who holds the lock
                if let Ok(content) = fs::read_to_string(&info_file) {
                    if let Ok(holder) = serde_json::from_str::<LockInfo>(&content) {
                        return Err(EnoError::ResourceLocked {
                            resource: resource.to_string(),
                            agent: holder.agent_id,
                        });
                    }
                }
                return Err(EnoError::Lock(format!(
                    "Timeout acquiring lock for '{}'",
                    resource
                )));
            }

            // Wait a bit before retrying
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Release a lock
    pub fn release(locks_dir: &Path, resource: &str) -> Result<()> {
        let lock_file = locks_dir.join(format!("{}.lock", resource));
        let info_file = locks_dir.join(format!("{}.json", resource));

        // Remove the files
        let _ = fs::remove_file(&lock_file);
        let _ = fs::remove_file(&info_file);

        Ok(())
    }

    /// List all active locks
    pub fn list(locks_dir: &Path) -> Result<Vec<LockInfo>> {
        let mut locks = Vec::new();

        if !locks_dir.exists() {
            return Ok(locks);
        }

        for entry in fs::read_dir(locks_dir)?.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(info) = serde_json::from_str::<LockInfo>(&content) {
                        locks.push(info);
                    }
                }
            }
        }

        Ok(locks)
    }

    /// Force-release a lock (steal)
    pub fn steal(locks_dir: &Path, resource: &str, new_agent_id: usize) -> Result<Self> {
        Self::release(locks_dir, resource)?;

        let info_file = locks_dir.join(format!("{}.json", resource));
        let lock_file = locks_dir.join(format!("{}.lock", resource));

        // Create new lock
        let _ = fs::File::create(&lock_file)?;
        let info = LockInfo::new(resource.to_string(), new_agent_id);
        let content = serde_json::to_string_pretty(&info)?;
        fs::write(&info_file, content)?;

        Ok(info)
    }
}
