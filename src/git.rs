use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{EnoError, Result};

pub struct GitManager {
    repo_path: PathBuf,
}

impl GitManager {
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        if which::which("git").is_err() {
            return Err(EnoError::GitNotInstalled);
        }

        let output = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(&repo_path)
            .output()?;

        if !output.status.success() {
            return Err(EnoError::NotGitRepo(repo_path.display().to_string()));
        }

        Ok(Self { repo_path })
    }

    pub fn create_worktree(&self, branch: &str, path: &Path, base_ref: &str) -> Result<()> {
        // Create the branch from base_ref if it doesn't exist
        if !self.branch_exists(branch)? {
            let output = Command::new("git")
                .args(["branch", branch, base_ref])
                .current_dir(&self.repo_path)
                .output()?;

            if !output.status.success() {
                return Err(EnoError::Git(format!(
                    "Failed to create branch '{}' from '{}': {}",
                    branch, base_ref,
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
        }

        let output = Command::new("git")
            .args(["worktree", "add", path.to_str().unwrap(), branch])
            .current_dir(&self.repo_path)
            .output()?;

        if !output.status.success() {
            return Err(EnoError::Git(format!(
                "Failed to create worktree at '{}': {}",
                path.display(),
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    pub fn remove_worktree(&self, path: &Path, delete_branch: bool) -> Result<()> {
        let branch = if delete_branch {
            self.worktree_branch(path).ok()
        } else {
            None
        };

        let _ = Command::new("git")
            .args(["worktree", "remove", "--force", path.to_str().unwrap()])
            .current_dir(&self.repo_path)
            .output();

        // Prune stale worktrees
        let _ = Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(&self.repo_path)
            .output();

        // Delete branch if requested
        if let Some(branch) = branch {
            let _ = Command::new("git")
                .args(["branch", "-D", &branch])
                .current_dir(&self.repo_path)
                .output();
        }

        Ok(())
    }

    fn worktree_branch(&self, path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(path)
            .output()?;

        if !output.status.success() {
            return Err(EnoError::Git("Failed to get worktree branch".to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn branch_exists(&self, branch: &str) -> Result<bool> {
        let output = Command::new("git")
            .args(["show-ref", "--verify", "--quiet", &format!("refs/heads/{}", branch)])
            .current_dir(&self.repo_path)
            .output()?;

        Ok(output.status.success())
    }

    pub fn detect_base_ref(&self) -> Result<String> {
        for refname in &[
            "refs/remotes/origin/main",
            "refs/remotes/origin/master",
            "refs/heads/main",
            "refs/heads/master",
        ] {
            let output = Command::new("git")
                .args(["show-ref", "--verify", "--quiet", refname])
                .current_dir(&self.repo_path)
                .output()?;

            if output.status.success() {
                return Ok(refname.replace("refs/remotes/", "").replace("refs/heads/", ""));
            }
        }

        Ok("HEAD".to_string())
    }

    pub fn prune_worktrees(&self) -> Result<()> {
        let _ = Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(&self.repo_path)
            .output();
        Ok(())
    }
}
