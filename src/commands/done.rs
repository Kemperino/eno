use std::process::Command;

use colored::Colorize;

use crate::context::CONTEXT_FILENAME;
use crate::error::{EnoError, Result};
use crate::session::SessionState;

pub fn run_done(
    agent: Option<u8>,
    message: Option<String>,
    title: Option<String>,
    body: Option<String>,
    base: Option<String>,
    no_pr: bool,
) -> Result<()> {
    let session = SessionState::find_active()?
        .ok_or_else(|| EnoError::NoActiveSession)?;

    // Determine which agent - either specified or detect from current directory
    let agent_id = match agent {
        Some(id) => id as usize,
        None => detect_agent_from_cwd(&session)?,
    };

    let agent_state = session.agents.iter()
        .find(|a| a.id == agent_id)
        .ok_or_else(|| EnoError::Config(format!("Agent {} not found", agent_id)))?;

    println!("\n{}", format!("📦 Finishing Agent {} work", agent_id).bold());
    println!("Branch: {}", agent_state.branch.cyan());
    println!("Worktree: {}\n", agent_state.worktree.display().to_string().dimmed());

    let worktree = &agent_state.worktree;

    // Check for changes
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(worktree)
        .output()?;

    let status_output = String::from_utf8_lossy(&status.stdout);
    let has_changes = !status_output.trim().is_empty();

    if has_changes {
        // Show what will be committed (excluding eno files)
        println!("{}", "Changes to commit:".bold());
        for line in status_output.lines() {
            let file = line.get(3..).unwrap_or("");
            if !file.starts_with(".eno") {
                println!("  {}", line);
            }
        }
        println!();

        // Stage all files except eno files
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(worktree)
            .output()?;

        // Unstage eno files
        let _ = Command::new("git")
            .args(["reset", "HEAD", "--", CONTEXT_FILENAME, ".eno-env"])
            .current_dir(worktree)
            .output();

        // Commit message - use branch name if not provided
        let commit_msg = message.unwrap_or_else(|| agent_state.branch.clone());

        print!("Committing... ");
        let commit = Command::new("git")
            .args(["commit", "-m", &commit_msg])
            .current_dir(worktree)
            .output()?;

        if !commit.status.success() {
            let stderr = String::from_utf8_lossy(&commit.stderr);
            if !stderr.contains("nothing to commit") {
                return Err(EnoError::Git(format!("Commit failed: {}", stderr)));
            }
        }
        println!("{}", "✓".green());

        // Push
        print!("Pushing to origin... ");
        let push = Command::new("git")
            .args(["push", "-u", "origin", &agent_state.branch])
            .current_dir(worktree)
            .output()?;

        if !push.status.success() {
            let stderr = String::from_utf8_lossy(&push.stderr);
            println!("{}", "✗".red());
            return Err(EnoError::Git(format!("Push failed: {}", stderr)));
        }
        println!("{}", "✓".green());
    } else {
        println!("{}", "No new changes to commit.".dimmed());
    }

    // Create PR unless --no-pr
    if !no_pr {
        // Check if PR already exists
        let pr_check = Command::new("gh")
            .args(["pr", "view", &agent_state.branch])
            .current_dir(worktree)
            .output();

        if let Ok(output) = pr_check {
            if output.status.success() {
                println!("\n{}", "PR already exists for this branch.".yellow());
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Extract URL from output
                for line in stdout.lines() {
                    if line.starts_with("url:") || line.contains("github.com") {
                        println!("  {}", line.trim().cyan());
                        break;
                    }
                }
                return Ok(());
            }
        }

        // Determine base branch: provided > main > master > develop
        let pr_base = base.unwrap_or_else(|| detect_default_branch(worktree));
        let pr_title = title.unwrap_or_else(|| "@coderabbitai".to_string());
        let pr_body = body.unwrap_or_else(|| agent_state.branch.clone());

        print!("Creating PR (base: {})... ", pr_base.cyan());
        let pr = Command::new("gh")
            .args([
                "pr", "create",
                "--title", &pr_title,
                "--body", &pr_body,
                "--base", &pr_base,
                "--head", &agent_state.branch,
            ])
            .current_dir(worktree)
            .output()?;

        if !pr.status.success() {
            let stderr = String::from_utf8_lossy(&pr.stderr);
            println!("{}", "✗".red());
            // Don't fail completely, just warn
            println!("{} PR creation failed: {}", "⚠".yellow(), stderr.trim());
        } else {
            println!("{}", "✓".green());
            let stdout = String::from_utf8_lossy(&pr.stdout);
            let url = stdout.trim();
            println!("\n{}", "PR created!".green().bold());
            println!("  {}", url.cyan());
        }
    }

    println!("\n{}", "Done!".green().bold());

    Ok(())
}

/// Detect default branch: main > master > develop
fn detect_default_branch(worktree: &std::path::Path) -> String {
    for branch in ["main", "master", "develop"] {
        let check = Command::new("git")
            .args(["rev-parse", "--verify", &format!("origin/{}", branch)])
            .current_dir(worktree)
            .output();

        if let Ok(output) = check {
            if output.status.success() {
                return branch.to_string();
            }
        }
    }
    // Fallback to main even if it doesn't exist
    "main".to_string()
}

fn detect_agent_from_cwd(session: &SessionState) -> Result<usize> {
    let cwd = std::env::current_dir()?;

    // Check if we're in one of the agent worktrees
    for agent in &session.agents {
        if cwd.starts_with(&agent.worktree) {
            return Ok(agent.id);
        }
    }

    // Check ENO_AGENT_ID env var
    if let Ok(id) = std::env::var("ENO_AGENT_ID") {
        if let Ok(id) = id.parse::<usize>() {
            return Ok(id);
        }
    }

    Err(EnoError::Config(
        "Could not detect agent. Run from worktree or specify agent number.".to_string()
    ))
}
