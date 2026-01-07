use serde::Deserialize;

use crate::cli::Tool;

/// Configuration file format (eno.yaml)
#[derive(Debug, Clone, Deserialize)]
pub struct EnoConfig {
    /// Base git ref to branch from
    #[serde(default)]
    pub base_ref: Option<String>,

    /// Agent configurations
    pub agents: Vec<AgentConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    /// Tool to use (claude or codex)
    pub tool: String,

    /// Task description
    pub task: String,

    /// Branch name (optional, auto-generated from task)
    #[serde(default)]
    pub branch: Option<String>,
}

impl EnoConfig {
    pub fn from_file(path: &std::path::PathBuf) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: EnoConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

/// Runtime agent specification
#[derive(Debug, Clone)]
pub struct AgentSpec {
    pub tool: Tool,
    pub task: String,
    pub branch: String,
}


/// Generate a concise branch name from a task description
pub fn task_to_branch_name(task: &str) -> String {
    let task_lower = task.to_lowercase();

    // Determine prefix based on task keywords
    let prefix = if task_lower.contains("fix") || task_lower.contains("bug") {
        "fix"
    } else if task_lower.contains("test") {
        "test"
    } else if task_lower.contains("doc") {
        "docs"
    } else if task_lower.contains("refactor") {
        "refactor"
    } else {
        "feature"
    };

    // Extract key words (skip common filler words and the prefix word itself)
    let slug: String = task_lower
        .split_whitespace()
        .filter(|word| {
            !matches!(
                *word,
                "the" | "a" | "an" | "to" | "for" | "of" | "in" | "on" | "with" | "and" | "or" |
                "fix" | "bug" | "test" | "docs" | "refactor" | "add" | "implement" | "create" |
                "update" | "upgrade" | "new" | "make" | "ensure" | "that" | "is" | "are" | "be"
            )
        })
        .take(3)
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect();

    // Truncate if too long (max 25 chars for slug)
    let slug = if slug.len() > 25 {
        slug[..25].trim_end_matches('-').to_string()
    } else {
        slug
    };

    format!("{}/{}", prefix, slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_to_branch_name() {
        // Refactor keyword triggers refactor/ prefix, filler words removed
        assert_eq!(
            task_to_branch_name("Refactor the authentication module"),
            "refactor/authentication-module"
        );
        // Fix/bug triggers fix/ prefix
        assert_eq!(
            task_to_branch_name("Fix bug in payment processing"),
            "fix/payment-processing"
        );
        // "test" keyword triggers test/ prefix
        assert_eq!(
            task_to_branch_name("Add comprehensive test coverage"),
            "test/comprehensive-coverage"
        );
        // "doc" keyword triggers docs/ prefix
        assert_eq!(
            task_to_branch_name("Update API documentation"),
            "docs/api-documentation"
        );
        // Simple feature - "add" is filtered out
        assert_eq!(
            task_to_branch_name("Add user authentication"),
            "feature/user-authentication"
        );
        // Long task gets truncated
        assert_eq!(
            task_to_branch_name("Implement a very complex feature with lots of functionality"),
            "feature/very-complex-feature"
        );
    }
}
