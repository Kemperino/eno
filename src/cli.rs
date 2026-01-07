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

    /// Manage resource locks
    Lock {
        #[command(subcommand)]
        action: LockAction,
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
}

#[derive(Subcommand)]
pub enum LockAction {
    /// Acquire a lock on a resource
    Acquire {
        /// Resource name to lock
        resource: String,

        /// Timeout in seconds (0 = no wait)
        #[arg(short, long, default_value = "30")]
        timeout: u64,
    },

    /// Release a lock on a resource
    Release {
        /// Resource name to release
        resource: String,
    },

    /// List all active locks
    List,

    /// Forcefully steal a lock (use with caution)
    Steal {
        /// Resource name to steal
        resource: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tool {
    Claude,
    Codex,
    Aider,
    Cursor,
    Custom,
}

impl Tool {
    /// Get the command name for this tool
    pub fn command(&self) -> &'static str {
        match self {
            Tool::Claude => "claude",
            Tool::Codex => "codex",
            Tool::Aider => "aider",
            Tool::Cursor => "cursor",
            Tool::Custom => "bash",
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
            Tool::Aider => "Install: pip install aider-chat",
            Tool::Cursor => "Install: Download from https://cursor.sh",
            Tool::Custom => "bash should be available on your system",
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
            "aider" => Ok(Tool::Aider),
            "cursor" => Ok(Tool::Cursor),
            "custom" => Ok(Tool::Custom),
            _ => Err(format!("Unknown tool: {}. Use claude, codex, aider, cursor, or custom", s)),
        }
    }
}
