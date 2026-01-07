use std::thread;
use std::time::Duration;

use chrono::Utc;
use colored::Colorize;
use tabled::{builder::Builder, settings::Style};

use crate::coordinator::PORT_RANGE;
use crate::error::{EnoError, Result};
use crate::session::{LockInfo, SessionState};

pub fn run_status(watch: bool, interval: u64) -> Result<()> {
    if watch {
        run_status_watch(interval)
    } else {
        run_status_once()
    }
}

fn run_status_once() -> Result<()> {
    let session = SessionState::find_active()?.ok_or(EnoError::NoActiveSession)?;

    print_session_status(&session)?;
    Ok(())
}

fn run_status_watch(interval: u64) -> Result<()> {
    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        match SessionState::find_active() {
            Ok(Some(session)) => {
                print_session_status(&session)?;
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

fn print_session_status(session: &SessionState) -> Result<()> {
    let now = Utc::now();
    let duration = now.signed_duration_since(session.created_at);
    let duration_str = format_duration(duration);

    println!("\n{}", "🎵 Eno Session Status".bold());
    println!();
    println!(
        "Session:  {}",
        session.id.cyan()
    );
    println!(
        "Repo:     {}",
        session.repo.display().to_string().dimmed()
    );
    println!(
        "Base ref: {}",
        session.base_ref.dimmed()
    );
    println!(
        "Created:  {} ago",
        duration_str.dimmed()
    );
    println!(
        "Tmux:     {}",
        session.tmux_session.dimmed()
    );

    // Agents table
    println!("\n{}", "Agents".bold());

    let mut builder = Builder::new();
    builder.push_record(["Agent", "Tool", "Task", "Branch", "Ports"]);

    for agent in &session.agents {
        let task_display = if agent.task.len() > 35 {
            format!("{}...", &agent.task[..35])
        } else {
            agent.task.clone()
        };

        let branch_display = if agent.branch.len() > 25 {
            format!("{}...", &agent.branch[..25])
        } else {
            agent.branch.clone()
        };

        builder.push_record([
            agent.id.to_string(),
            agent.tool.to_string(),
            task_display,
            branch_display,
            format!("{}-{}", agent.port_base, agent.port_base + PORT_RANGE - 1),
        ]);
    }

    let table = builder.build().with(Style::rounded()).to_string();
    println!("{}", table);

    // Locks
    let locks = LockInfo::list(&session.locks_dir())?;
    if !locks.is_empty() {
        println!("\n{}", "Active Locks".bold());

        let mut lock_builder = Builder::new();
        lock_builder.push_record(["Resource", "Agent", "Held For"]);

        for lock in &locks {
            let held_duration = now.signed_duration_since(lock.acquired_at);
            lock_builder.push_record([
                lock.resource.clone(),
                format!("Agent {}", lock.agent_id),
                format_duration(held_duration),
            ]);
        }

        let lock_table = lock_builder.build().with(Style::rounded()).to_string();
        println!("{}", lock_table);
    }

    Ok(())
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
