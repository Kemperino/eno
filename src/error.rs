use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnoError {
    #[error("Git error: {0}")]
    Git(String),

    #[error("Tmux error: {0}")]
    Tmux(String),

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Dialogue error: {0}")]
    Dialogue(String),

    #[error("No active session found")]
    NoActiveSession,

    #[error("Session already exists: {0}")]
    SessionExists(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(usize),

    #[error("Resource already locked: {resource} (held by agent {agent})")]
    ResourceLocked { resource: String, agent: usize },

    #[error("Maximum agents ({0}) exceeded")]
    MaxAgentsExceeded(usize),

    #[error("Repository not found at: {0}")]
    RepoNotFound(String),

    #[error("Tmux not installed. Please install tmux first.")]
    TmuxNotInstalled,

    #[error("Git not installed. Please install git first.")]
    GitNotInstalled,

    #[error("Not a git repository: {0}")]
    NotGitRepo(String),

    #[error("Tool '{tool}' is not installed.\n  {hint}")]
    ToolNotInstalled { tool: String, hint: String },
}

impl From<dialoguer::Error> for EnoError {
    fn from(err: dialoguer::Error) -> Self {
        EnoError::Dialogue(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, EnoError>;
