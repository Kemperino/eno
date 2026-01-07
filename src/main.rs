use clap::Parser;
use colored::Colorize;

mod cli;
mod commands;
mod config;
mod context;
mod coordinator;
mod error;
mod git;
mod session;
mod tmux;

use cli::{Cli, Commands};
use coordinator::PORT_RANGE;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Start {
            repo,
            base_ref,
            agents,
            config,
            interactive,
            agent_specs,
            no_attach,
        } => commands::run_start(repo, base_ref, agents, config, interactive, agent_specs, no_attach),

        Commands::Status { watch, interval } => commands::run_status(watch, interval),

        Commands::Send { agent, message } => commands::run_send(agent, message),

        Commands::Broadcast { message } => commands::run_broadcast(message),

        Commands::Lock { action } => commands::run_lock(action),

        Commands::Attach => commands::run_attach(),

        Commands::Cleanup { force, keep_branches } => commands::run_cleanup(force, keep_branches),

        Commands::Agent { number } => {
            // Show info about a specific agent
            match session::SessionState::find_active() {
                Ok(Some(session)) => {
                    match session.get_agent(number as usize) {
                        Some(agent) => {
                            println!("\n{}", format!("Agent {}", number).bold());
                            println!("Tool:     {}", agent.tool);
                            println!("Task:     {}", agent.task);
                            println!("Branch:   {}", agent.branch.cyan());
                            println!("Worktree: {}", agent.worktree.display().to_string().dimmed());
                            println!(
                                "Ports:    {}-{}",
                                agent.port_base,
                                agent.port_base + PORT_RANGE - 1
                            );
                            Ok(())
                        }
                        None => Err(error::EnoError::AgentNotFound(number as usize)),
                    }
                }
                Ok(None) => Err(error::EnoError::NoActiveSession),
                Err(e) => Err(e),
            }
        }

        Commands::Init { force } => {
            // Initialize eno in the current repository
            let cwd = std::env::current_dir().unwrap();

            // Check if already initialized
            let eno_dir = cwd.join(".eno-worktrees");
            if eno_dir.exists() && !force {
                println!(
                    "{} Already initialized. Use --force to reinitialize.",
                    "⚠".yellow()
                );
                return;
            }

            // Create .eno-worktrees directory
            std::fs::create_dir_all(&eno_dir).ok();

            // Add to .gitignore
            let gitignore = cwd.join(".gitignore");
            if gitignore.exists() {
                let content = std::fs::read_to_string(&gitignore).unwrap_or_default();
                if !content.contains(".eno-worktrees") {
                    let new_content = format!("{}\n# Eno worktrees\n.eno-worktrees/\n", content.trim());
                    std::fs::write(&gitignore, new_content).ok();
                    println!("{} Added .eno-worktrees to .gitignore", "✓".green());
                }
            }

            // Create example config
            let config_path = cwd.join("eno.yaml.example");
            if !config_path.exists() {
                let example = r#"# Eno session configuration
# Rename to eno.yaml and customize

# Base git ref to branch from (auto-detected if not provided)
# base_ref: origin/main

# Agent configurations
agents:
  - tool: claude
    task: Implement the main feature
    # branch: feature/main-feature  # Optional, auto-generated from task

  - tool: codex
    task: Add test coverage
    # branch: test/coverage
"#;
                std::fs::write(&config_path, example).ok();
                println!("{} Created eno.yaml.example", "✓".green());
            }

            println!("\n{}", "Eno initialized!".green().bold());
            println!("\nNext steps:");
            println!("  1. Copy eno.yaml.example to eno.yaml and customize");
            println!("  2. Run: {} --config eno.yaml", "eno start".cyan());
            println!("  3. Or run: {} for interactive setup", "eno start -i".cyan());

            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{}: {}", "Error".red().bold(), e);
        std::process::exit(1);
    }
}
