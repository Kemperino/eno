use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::error::{EnoError, Result};

pub struct TmuxManager {
    session_name: String,
}

impl TmuxManager {
    pub fn new(session_name: String) -> Result<Self> {
        if which::which("tmux").is_err() {
            return Err(EnoError::TmuxNotInstalled);
        }
        Ok(Self { session_name })
    }

    pub fn session_exists(&self) -> bool {
        Command::new("tmux")
            .args(["has-session", "-t", &self.session_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn create_session(&self, first_window: &WindowConfig) -> Result<()> {
        if self.session_exists() {
            return Err(EnoError::SessionExists(self.session_name.clone()));
        }

        let output = Command::new("tmux")
            .args([
                "new-session", "-d",
                "-s", &self.session_name,
                "-n", &first_window.name,
                "-c", first_window.working_dir.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(EnoError::Tmux(format!(
                "Failed to create session: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Write env file and source it, then run command
        if !first_window.env.is_empty() {
            let env_file = first_window.working_dir.join(".eno-env");
            self.write_env_file(&env_file, &first_window.env)?;
            self.send_keys_to_window(&first_window.name, &format!("source {}", env_file.display()))?;
        }
        if let Some(ref cmd) = first_window.command {
            self.send_keys_to_window(&first_window.name, cmd)?;
        }

        Ok(())
    }

    pub fn add_window(&self, config: &WindowConfig) -> Result<()> {
        let output = Command::new("tmux")
            .args([
                "new-window",
                "-t", &self.session_name,
                "-n", &config.name,
                "-c", config.working_dir.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(EnoError::Tmux(format!(
                "Failed to create window '{}': {}",
                config.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Write env file and source it, then run command
        if !config.env.is_empty() {
            let env_file = config.working_dir.join(".eno-env");
            self.write_env_file(&env_file, &config.env)?;
            self.send_keys_to_window(&config.name, &format!("source {}", env_file.display()))?;
        }
        if let Some(ref cmd) = config.command {
            self.send_keys_to_window(&config.name, cmd)?;
        }

        Ok(())
    }

    fn write_env_file(&self, path: &PathBuf, env: &HashMap<String, String>) -> Result<()> {
        let mut content = String::from("#!/bin/bash\n");
        for (key, value) in env {
            // Use heredoc syntax for values to handle any special characters
            content.push_str(&format!(
                "export {}=$(cat <<'__ENO_EOF__'\n{}\n__ENO_EOF__\n)\n",
                key, value
            ));
        }
        fs::write(path, content)?;
        Ok(())
    }

    pub fn send_keys_to_window(&self, window: &str, keys: &str) -> Result<()> {
        let target = format!("{}:{}", self.session_name, window);
        // Use -l for literal mode to avoid tmux key interpretation issues
        Command::new("tmux")
            .args(["send-keys", "-t", &target, "-l", keys])
            .output()?;
        // Send Enter separately
        Command::new("tmux")
            .args(["send-keys", "-t", &target, "Enter"])
            .output()?;
        Ok(())
    }

    pub fn attach(&self) -> Result<()> {
        use std::os::unix::process::CommandExt;
        let err = Command::new("tmux")
            .args(["attach-session", "-t", &self.session_name])
            .exec();
        Err(EnoError::Tmux(format!("Failed to attach: {}", err)))
    }

    pub fn kill_session(&self) -> Result<()> {
        let output = Command::new("tmux")
            .args(["kill-session", "-t", &self.session_name])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("can't find session") {
                return Err(EnoError::Tmux(format!("Failed to kill session: {}", stderr)));
            }
        }
        Ok(())
    }

    /// Kill all eno-prefixed tmux sessions (for cleanup of stale sessions)
    pub fn kill_all_eno_sessions() -> Vec<String> {
        let mut killed = Vec::new();

        // List all tmux sessions
        let output = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let sessions = String::from_utf8_lossy(&output.stdout);
                for session in sessions.lines() {
                    if session.starts_with("eno-") {
                        let result = Command::new("tmux")
                            .args(["kill-session", "-t", session])
                            .output();
                        if result.map(|o| o.status.success()).unwrap_or(false) {
                            killed.push(session.to_string());
                        }
                    }
                }
            }
        }

        killed
    }
}

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub name: String,
    pub working_dir: PathBuf,
    pub command: Option<String>,
    pub env: HashMap<String, String>,
}

impl WindowConfig {
    pub fn new(name: String, working_dir: PathBuf) -> Self {
        Self {
            name,
            working_dir,
            command: None,
            env: HashMap::new(),
        }
    }

    pub fn with_command(mut self, command: String) -> Self {
        self.command = Some(command);
        self
    }

    pub fn with_env_map(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }
}
