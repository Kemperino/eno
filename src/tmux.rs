use std::collections::HashMap;
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

        // Set env vars and run command
        for (key, value) in &first_window.env {
            self.set_env(&first_window.name, key, value)?;
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

        for (key, value) in &config.env {
            self.set_env(&config.name, key, value)?;
        }
        if let Some(ref cmd) = config.command {
            self.send_keys_to_window(&config.name, cmd)?;
        }

        Ok(())
    }

    fn set_env(&self, window: &str, key: &str, value: &str) -> Result<()> {
        let target = format!("{}:{}", self.session_name, window);
        let env_cmd = format!("export {}='{}'", key, value);

        Command::new("tmux")
            .args(["send-keys", "-t", &target, &env_cmd, "Enter"])
            .output()?;
        Ok(())
    }

    pub fn send_keys_to_window(&self, window: &str, keys: &str) -> Result<()> {
        let target = format!("{}:{}", self.session_name, window);
        Command::new("tmux")
            .args(["send-keys", "-t", &target, keys, "Enter"])
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
