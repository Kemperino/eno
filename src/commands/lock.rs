use chrono::Utc;
use colored::Colorize;
use tabled::{builder::Builder, settings::Style};

use crate::cli::LockAction;
use crate::error::{EnoError, Result};
use crate::session::{LockInfo, SessionState};

pub fn run_lock(action: LockAction) -> Result<()> {
    let session = SessionState::find_active()?.ok_or(EnoError::NoActiveSession)?;
    let locks_dir = session.locks_dir();

    match action {
        LockAction::Acquire { resource, timeout } => {
            // Determine agent ID from environment, or default to 0 (manual)
            let agent_id: usize = std::env::var("ENO_AGENT_ID")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            println!(
                "Acquiring lock on '{}' (timeout: {}s)...",
                resource.cyan(),
                timeout
            );

            match LockInfo::acquire(&locks_dir, &resource, agent_id, timeout) {
                Ok(info) => {
                    println!(
                        "{} Lock acquired on '{}' by agent {}",
                        "✓".green(),
                        resource,
                        info.agent_id
                    );
                }
                Err(EnoError::ResourceLocked { resource, agent }) => {
                    println!(
                        "{} Resource '{}' is locked by agent {}",
                        "✗".red(),
                        resource,
                        agent
                    );
                    return Err(EnoError::ResourceLocked { resource, agent });
                }
                Err(e) => return Err(e),
            }
        }

        LockAction::Release { resource } => {
            LockInfo::release(&locks_dir, &resource)?;
            println!("{} Released lock on '{}'", "✓".green(), resource);
        }

        LockAction::List => {
            let locks = LockInfo::list(&locks_dir)?;

            if locks.is_empty() {
                println!("{}", "No active locks".dimmed());
            } else {
                println!("\n{}", "Active Locks".bold());

                let mut builder = Builder::new();
                builder.push_record(["Resource", "Agent", "Held For", "PID"]);

                let now = Utc::now();
                for lock in &locks {
                    let held_duration = now.signed_duration_since(lock.acquired_at);
                    let secs = held_duration.num_seconds();
                    let duration_str = if secs < 60 {
                        format!("{}s", secs)
                    } else if secs < 3600 {
                        format!("{}m {}s", secs / 60, secs % 60)
                    } else {
                        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                    };

                    builder.push_record([
                        lock.resource.clone(),
                        if lock.agent_id == 0 {
                            "manual".to_string()
                        } else {
                            format!("Agent {}", lock.agent_id)
                        },
                        duration_str,
                        lock.pid.to_string(),
                    ]);
                }

                let table = builder.build().with(Style::rounded()).to_string();
                println!("{}", table);
            }
        }

        LockAction::Steal { resource } => {
            // Determine agent ID from environment, or default to 0 (manual)
            let agent_id: usize = std::env::var("ENO_AGENT_ID")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            println!(
                "{} Stealing lock on '{}'...",
                "⚠".yellow(),
                resource.cyan()
            );

            let info = LockInfo::steal(&locks_dir, &resource, agent_id)?;
            println!(
                "{} Lock stolen on '{}' by agent {}",
                "✓".green(),
                resource,
                info.agent_id
            );
        }
    }

    Ok(())
}
