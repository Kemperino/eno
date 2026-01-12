use std::process::Command;
use std::thread;
use std::time::Duration;

use chrono::Utc;
use colored::Colorize;

use crate::error::{EnoError, Result};
use crate::session::{AgentState, SessionState};

pub fn run_status(watch: bool, interval: u64) -> Result<()> {
    if watch {
        run_status_watch(interval)
    } else {
        run_status_once()
    }
}

fn run_status_once() -> Result<()> {
    let session = SessionState::find_active()?.ok_or(EnoError::NoActiveSession)?;
    print_session_status(&session);
    Ok(())
}

fn run_status_watch(interval: u64) -> Result<()> {
    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        match SessionState::find_active() {
            Ok(Some(session)) => {
                print_session_status(&session);
                println!(
                    "\n{}",
                    format!("Refreshing every {}s... (Ctrl-C to stop)", interval).dimmed()
                );
            }
            Ok(None) => {
                println!("{}", "No active session".yellow());
                println!(
                    "\n{}",
                    format!("Refreshing every {}s... (Ctrl-C to stop)", interval).dimmed()
                );
            }
            Err(e) => {
                println!("{}: {}", "Error".red(), e);
            }
        }

        thread::sleep(Duration::from_secs(interval));
    }
}

fn print_session_status(session: &SessionState) {
    let now = Utc::now();
    let duration = now.signed_duration_since(session.created_at);
    let duration_str = format_duration(duration);

    println!("\n{}", "🎵 Eno Session Status".bold());
    println!();
    println!("Session:  {}", session.id.cyan());
    println!("Repo:     {}", session.repo.display().to_string().dimmed());
    println!("Base ref: {}", session.base_ref.dimmed());
    println!("Created:  {} ago", duration_str.dimmed());
    println!("Tmux:     {}", session.tmux_session.dimmed());

    // Agents list
    println!("\n{}", "Agents".bold());
    println!();

    for agent in &session.agents {
        let (status_icon, status_text, status_color) = get_agent_status(agent, &session.tmux_session);
        let changes = get_git_changes(agent);

        let task_display = if agent.task.len() > 50 {
            format!("{}...", &agent.task[..50])
        } else {
            agent.task.clone()
        };

        // Format with fixed widths before applying color
        let status_str = format!("{} {:7}", status_icon, status_text);
        let status_colored = match status_color {
            "green" => status_str.green(),
            "yellow" => status_str.yellow(),
            _ => status_str.normal(),
        };

        println!(
            "  {} {:5}  {}  {:8}  {}",
            format!("[{}]", agent.id).bold(),
            agent.tool.to_string(),
            status_colored,
            changes,
            task_display.dimmed()
        );
    }

    println!("\n{}", "Commands".dimmed());
    println!("  {} - attach to session", "eno attach".cyan());
    println!("  {} - commit & push agent work", "eno done <n>".cyan());
}

fn format_duration(duration: chrono::Duration) -> String {
    let secs = duration.num_seconds();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Get agent status: (icon, text, color)
fn get_agent_status(agent: &AgentState, tmux_session: &str) -> (&'static str, &'static str, &'static str) {
    // Check if branch has been pushed (eno done was run)
    let pushed = Command::new("git")
        .args(["rev-parse", "--verify", &format!("origin/{}", agent.branch)])
        .current_dir(&agent.worktree)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if pushed {
        return ("✓", "pushed", "green");
    }

    // Check tmux pane to see if claude/codex (node) is running
    let window_name = agent.display_name();
    let target = format!("{}:{}", tmux_session, window_name);

    let output = Command::new("tmux")
        .args(["list-panes", "-t", &target, "-F", "#{pane_current_command}"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let cmd = String::from_utf8_lossy(&output.stdout);
            let cmd = cmd.trim();
            // node = claude/codex running, zsh/bash = idle at prompt
            if cmd == "node" {
                return ("⚡", "working", "yellow");
            } else if cmd == "zsh" || cmd == "bash" || cmd == "fish" {
                return ("⏸", "idle", "dim");
            }
        }
    }

    ("?", "unknown", "dim")
}

/// Get git changes count (excluding eno files)
fn get_git_changes(agent: &AgentState) -> String {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&agent.worktree)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout);
            let changes: Vec<&str> = status
                .lines()
                .filter(|line| {
                    let file = line.get(3..).unwrap_or("");
                    !file.starts_with(".eno")
                })
                .collect();

            if changes.is_empty() {
                return "—".to_string();
            }

            // Count by type
            let added = changes.iter().filter(|l| l.starts_with("A ") || l.starts_with("??")).count();
            let modified = changes.iter().filter(|l| l.starts_with(" M") || l.starts_with("M ")).count();
            let deleted = changes.iter().filter(|l| l.starts_with(" D") || l.starts_with("D ")).count();

            let mut parts = Vec::new();
            if added > 0 { parts.push(format!("+{}", added)); }
            if modified > 0 { parts.push(format!("~{}", modified)); }
            if deleted > 0 { parts.push(format!("-{}", deleted)); }

            if parts.is_empty() {
                format!("{} files", changes.len())
            } else {
                parts.join(" ")
            }
        } else {
            "error".to_string()
        }
    } else {
        "error".to_string()
    }
}
