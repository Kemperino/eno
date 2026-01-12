use std::fs;
use std::path::Path;

use crate::coordinator::PORT_RANGE;
use crate::error::Result;
use crate::session::{AgentState, SessionState};

/// Context filename - use a unique name to avoid overwriting repo files
pub const CONTEXT_FILENAME: &str = ".eno-context.md";

/// Generate the context file content for an agent
pub fn generate_context_file(agent: &AgentState, session: &SessionState) -> String {
    let port_base = agent.port_base;
    let port_end = port_base + PORT_RANGE - 1;

    let mut content = String::new();

    // Header
    content.push_str("# Eno Agent Context\n\n");
    content.push_str(&format!(
        "You are **Agent {}** of **{}** in a coordinated eno swarm session.\n\n",
        agent.id,
        session.agents.len()
    ));

    // Task
    content.push_str("## Your Task\n\n");
    content.push_str(&format!("{}\n\n", agent.task));

    // Branch info
    content.push_str("## Git Branch\n\n");
    content.push_str(&format!("You are working on branch: `{}`\n\n", agent.branch));
    content.push_str(&format!("Base ref: `{}`\n\n", session.base_ref));

    // Resource allocation
    content.push_str("## Resource Allocation\n\n");
    content.push_str("| Resource | Your Assignment |\n");
    content.push_str("|----------|----------------|\n");
    content.push_str(&format!("| Port range | {}-{} |\n", port_base, port_end));
    content.push_str(&format!("| Docker prefix | `eno-{}-` |\n", agent.id));
    content.push_str(&format!("| Compose project | `eno-agent-{}` |\n", agent.id));
    content.push_str(&format!("| Test DB | `test_agent_{}` |\n", agent.id));
    content.push('\n');

    // Common ports
    content.push_str("### Common Port Mappings\n\n");
    content.push_str("| Service | Port | Env Var |\n");
    content.push_str("|---------|------|--------|\n");
    content.push_str(&format!("| HTTP | {} | `$ENO_HTTP_PORT` |\n", port_base));
    content.push_str(&format!("| HTTPS | {} | `$ENO_HTTPS_PORT` |\n", port_base + 1));
    content.push_str(&format!("| Database | {} | `$ENO_DB_PORT` |\n", port_base + 32));
    content.push_str(&format!("| Redis | {} | `$ENO_REDIS_PORT` |\n", port_base + 79));
    content.push('\n');

    // Coordination rules
    content.push_str("## Coordination Rules\n\n");
    content.push_str(&format!(
        "1. **Ports**: Only bind to ports in your range ({}-{})\n",
        port_base, port_end
    ));
    content.push_str(&format!(
        "2. **Docker**: Prefix all container names with `eno-{}-`\n",
        agent.id
    ));
    content.push_str("3. **Stay in your lane**: Focus on your task, avoid modifying files other agents are working on\n\n");

    // Other agents
    if session.agents.len() > 1 {
        content.push_str("## Other Agents (for awareness)\n\n");
        content.push_str("| Agent | Tool | Task | Branch |\n");
        content.push_str("|-------|------|------|--------|\n");

        for other in &session.agents {
            if other.id != agent.id {
                let task_preview = if other.task.len() > 40 {
                    format!("{}...", &other.task[..40])
                } else {
                    other.task.clone()
                };
                content.push_str(&format!(
                    "| {} | {} | {} | `{}` |\n",
                    other.id, other.tool, task_preview, other.branch
                ));
            }
        }
        content.push('\n');
    }

    // Session commands
    content.push_str("## Session Commands\n\n");
    content.push_str("```bash\n");
    content.push_str("eno status          # See all agents and their status\n");
    if session.agents.len() > 1 {
        content.push_str("eno send 2 \"msg\"    # Message another agent\n");
        content.push_str("eno broadcast \"msg\" # Message all agents\n");
    }
    content.push_str("```\n\n");

    // Environment variables
    content.push_str("## Environment Variables\n\n");
    content.push_str("The following environment variables are available:\n\n");
    content.push_str("```bash\n");
    content.push_str(&format!("ENO_AGENT_ID={}          # Your agent ID\n", agent.id));
    content.push_str(&format!(
        "ENO_AGENT_COUNT={}       # Total number of agents\n",
        session.agents.len()
    ));
    content.push_str(&format!("ENO_SESSION_ID={}  # Session identifier\n", session.id));
    content.push_str(&format!("ENO_PORT_BASE={}       # Start of your port range\n", port_base));
    content.push_str(&format!("ENO_PORT_RANGE={}       # Size of your port range\n", PORT_RANGE));
    content.push_str("```\n");

    content
}

/// Inject the context file into a worktree
pub fn inject_context(worktree: &Path, content: &str, filename: &str) -> Result<()> {
    let context_path = worktree.join(filename);
    fs::write(&context_path, content)?;

    // Add to .git/info/exclude so it's not committed
    let exclude_path = worktree.join(".git").join("info").join("exclude");
    if exclude_path.exists() {
        let exclude_content = fs::read_to_string(&exclude_path)?;
        let mut new_content = exclude_content.trim().to_string();
        let mut changed = false;
        if !exclude_content.contains(filename) {
            new_content.push_str(&format!("\n# Eno files\n{}\n", filename));
            changed = true;
        }
        if !exclude_content.contains(".eno-env") {
            new_content.push_str(".eno-env\n");
            changed = true;
        }
        if changed {
            fs::write(&exclude_path, new_content)?;
        }
    }

    Ok(())
}

/// Remove the context file from a worktree
pub fn remove_context(worktree: &Path, filename: &str) -> Result<()> {
    let context_path = worktree.join(filename);
    if context_path.exists() {
        fs::remove_file(&context_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Tool;
    use std::path::PathBuf;

    fn create_test_session() -> SessionState {
        SessionState {
            id: "test-session".to_string(),
            repo: PathBuf::from("/tmp/test-repo"),
            base_ref: "origin/main".to_string(),
            created_at: chrono::Utc::now(),
            agents: vec![
                AgentState {
                    id: 1,
                    tool: Tool::Claude,
                    task: "Refactor authentication".to_string(),
                    branch: "feature/auth-refactor".to_string(),
                    worktree: PathBuf::from("/tmp/wt-1"),
                    port_base: 9100,
                },
                AgentState {
                    id: 2,
                    tool: Tool::Codex,
                    task: "Add test coverage".to_string(),
                    branch: "test/coverage".to_string(),
                    worktree: PathBuf::from("/tmp/wt-2"),
                    port_base: 9200,
                },
            ],
            state_dir: PathBuf::from("/tmp/eno-sessions/test"),
            tmux_session: "eno-test".to_string(),
        }
    }

    #[test]
    fn test_generate_context_file() {
        let session = create_test_session();
        let agent = &session.agents[0];
        let content = generate_context_file(agent, &session);

        assert!(content.contains("Agent 1"));
        assert!(content.contains("of **2**")); // Markdown bold
        assert!(content.contains("Refactor authentication"));
        assert!(content.contains("9100-9199"));
        // Other agents are in a table format: | id | tool | task | branch |
        assert!(content.contains("| 2 |"));
        assert!(content.contains("Add test coverage"));
    }
}
