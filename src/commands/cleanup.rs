use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::context::remove_context;
use crate::error::{EnoError, Result};
use crate::git::GitManager;
use crate::session::SessionState;
use crate::tmux::TmuxManager;

pub fn run_cleanup(force: bool, keep_branches: bool) -> Result<()> {
    let session = SessionState::find_active()?.ok_or(EnoError::NoActiveSession)?;

    println!("\n{}", "🎵 Eno Cleanup".bold());
    println!();
    println!("Session: {}", session.id.cyan());
    println!("Agents:  {}", session.agents.len());

    if !force {
        let theme = ColorfulTheme::default();
        let confirmed = Confirm::with_theme(&theme)
            .with_prompt("Are you sure you want to cleanup this session?")
            .default(false)
            .interact()?;

        if !confirmed {
            println!("{}", "Cancelled".yellow());
            return Ok(());
        }
    }

    println!("\nCleaning up...\n");

    // Kill tmux session
    print!("  Killing tmux session ");
    let tmux = TmuxManager::new(session.tmux_session.clone())?;
    match tmux.kill_session() {
        Ok(_) => println!("{}", "✓".green()),
        Err(e) => println!("{} ({})", "⚠".yellow(), e),
    }

    // Initialize git manager
    let git = GitManager::new(session.repo.clone())?;

    // Remove worktrees and optionally branches
    for agent in &session.agents {
        // Remove context file first
        let _ = remove_context(&agent.worktree, "CLAUDE.md");

        // Remove worktree
        print!("  Removing worktree: {} ", agent.display_name().cyan());
        match git.remove_worktree(&agent.worktree, !keep_branches) {
            Ok(_) => println!("{}", "✓".green()),
            Err(e) => println!("{} ({})", "⚠".yellow(), e),
        }

        if keep_branches {
            println!(
                "    {} Kept branch: {}",
                "→".dimmed(),
                agent.branch.dimmed()
            );
        }
    }

    // Remove the worktree directory if empty
    let worktree_base = session.repo.join(".eno-worktrees");
    if worktree_base.exists() {
        if let Ok(entries) = std::fs::read_dir(&worktree_base) {
            if entries.count() == 0 {
                let _ = std::fs::remove_dir(&worktree_base);
            }
        }
    }

    // Prune stale worktrees
    let _ = git.prune_worktrees();

    // Remove session state
    print!("  Removing session state ");
    match session.cleanup() {
        Ok(_) => println!("{}", "✓".green()),
        Err(e) => println!("{} ({})", "⚠".yellow(), e),
    }

    println!("\n{}", "Cleanup complete!".green().bold());

    Ok(())
}
