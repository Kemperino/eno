use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "eno",
    about = "Minimalist agent orchestration tool",
    long_about = "eno - Like the composer, minimalist and simple.\n\n\
                  Orchestrate parallel AI coding agents with isolated worktrees,\n\
                  resource coordination, and tmux-based workflow.",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start a new agent swarm session
    Start {
        /// Path to the repository (defaults to current directory)
        #[arg(short, long)]
        repo: Option<PathBuf>,

        /// Base git ref to branch from (auto-detects origin/main or origin/master)
        #[arg(short, long)]
        base_ref: Option<String>,

        /// Number of agents to spawn (1-4)
        #[arg(short = 'n', long, value_parser = clap::value_parser!(u8).range(1..=4))]
        agents: Option<u8>,

        /// Configuration file (YAML)
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Interactive mode - prompt for each agent's task
        #[arg(short, long)]
        interactive: bool,

        /// Agent specifications in format: tool:task (can be repeated)
        #[arg(short = 'a', long = "agent", value_name = "TOOL:TASK")]
        agent_specs: Vec<String>,

        /// Don't attach to tmux session after creation
        #[arg(long)]
        no_attach: bool,
    },

    /// Show status of current swarm session
    Status {
        /// Watch mode - continuously update status
        #[arg(short, long)]
        watch: bool,

        /// Refresh interval in seconds for watch mode
        #[arg(long, default_value = "2")]
        interval: u64,
    },

    /// Send a message to an agent
    Send {
        /// Agent number to send to (1-4)
        #[arg(value_parser = clap::value_parser!(u8).range(1..=4))]
        agent: u8,

        /// Message to send
        message: String,
    },

    /// Broadcast a message to all agents
    Broadcast {
        /// Message to broadcast
        message: String,
    },

    /// Attach to the tmux session
    Attach,

    /// Clean up the current session
    Cleanup {
        /// Force cleanup without confirmation
        #[arg(short, long)]
        force: bool,

        /// Keep branches after cleanup
        #[arg(long)]
        keep_branches: bool,
    },

    /// Show info about a specific agent
    Agent {
        /// Agent number (1-4)
        #[arg(value_parser = clap::value_parser!(u8).range(1..=4))]
        number: u8,
    },

    /// Initialize eno in current repository
    Init {
        /// Force initialization even if already initialized
        #[arg(short, long)]
        force: bool,
    },

    /// Commit, push, and create PR for agent's work
    Done {
        /// Agent number (auto-detects if run from worktree)
        #[arg(value_parser = clap::value_parser!(u8).range(1..=4))]
        agent: Option<u8>,

        /// Custom commit message (default: uses task description)
        #[arg(short, long)]
        message: Option<String>,

        /// PR title (default: @coderabbitai)
        #[arg(long)]
        title: Option<String>,

        /// PR body (default: branch name)
        #[arg(long)]
        body: Option<String>,

        /// Base branch for PR (default: main)
        #[arg(long)]
        base: Option<String>,

        /// Skip PR creation
        #[arg(long)]
        no_pr: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tool {
    Claude,
    Codex,
}

impl Tool {
    /// Get the command name for this tool
    pub fn command(&self) -> &'static str {
        match self {
            Tool::Claude => "claude",
            Tool::Codex => "codex",
        }
    }

    /// Get the full launch command (reads task from ENO_TASK env var)
    pub fn launch_command(&self) -> String {
        match self {
            Tool::Claude => "claude --dangerously-skip-permissions \"$ENO_TASK\"".to_string(),
            Tool::Codex => "codex --dangerously-bypass-approvals-and-sandbox \"$ENO_TASK\"".to_string(),
        }
    }

    /// Check if the tool is installed
    pub fn is_installed(&self) -> bool {
        which::which(self.command()).is_ok()
    }

    /// Get install instructions for this tool
    pub fn install_hint(&self) -> &'static str {
        match self {
            Tool::Claude => "Install: npm install -g @anthropic-ai/claude-code",
            Tool::Codex => "Install: npm install -g @openai/codex",
        }
    }
}

impl std::fmt::Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.command())
    }
}

impl std::str::FromStr for Tool {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(Tool::Claude),
            "codex" => Ok(Tool::Codex),
            _ => Err(format!("Unknown tool: {}. Use claude or codex", s)),
        }
    }
}
