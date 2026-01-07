use std::path::PathBuf;

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};

use crate::cli::Tool;
use crate::config::{task_to_branch_name, AgentSpec, EnoConfig};
use crate::context::{generate_context_file, inject_context};
use crate::coordinator::ResourceCoordinator;
use crate::error::{EnoError, Result};
use crate::git::GitManager;
use crate::session::{AgentState, SessionState};
use crate::tmux::{TmuxManager, WindowConfig};

pub fn run_start(
    repo: Option<PathBuf>,
    base_ref: Option<String>,
    agents: Option<u8>,
    config: Option<PathBuf>,
    interactive: bool,
    agent_specs: Vec<String>,
    no_attach: bool,
) -> Result<()> {
    println!("\n{}", "🎵 Eno Agent Orchestrator".bold());
    println!("{}\n", "   Like the composer, minimalist and simple.".dimmed());

    // Determine repository path
    let repo_path = repo.unwrap_or_else(|| std::env::current_dir().unwrap());
    let repo_path = repo_path.canonicalize().map_err(|_| {
        EnoError::RepoNotFound(repo_path.display().to_string())
    })?;

    println!("Repository: {}", repo_path.display().to_string().cyan());

    // Initialize git manager
    let git = GitManager::new(repo_path.clone())?;

    // Collect agent specifications (and possibly get base_ref from config)
    let (specs, config_base_ref) = if let Some(config_path) = config {
        // Load from config file
        let config = EnoConfig::from_file(&config_path)?;
        let specs = config
            .agents
            .into_iter()
            .map(|a| {
                let tool: Tool = a.tool.parse().map_err(|e: String| EnoError::Config(e))?;
                let branch = a.branch.unwrap_or_else(|| task_to_branch_name(&a.task));
                Ok(AgentSpec {
                    tool,
                    task: a.task,
                    branch,
                    command: a.command,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        (specs, config.base_ref)
    } else if !agent_specs.is_empty() {
        // Parse from command line
        (agent_specs
            .into_iter()
            .map(|s| parse_agent_spec(&s))
            .collect::<Result<Vec<_>>>()?, None)
    } else if interactive {
        // Interactive mode
        (collect_agents_interactive(agents)?, None)
    } else {
        return Err(EnoError::Config(
            "No agents specified. Use --interactive, --config, or --agent".to_string(),
        ));
    };

    // Determine base ref: command line > config > auto-detect
    let base_ref = match base_ref.or(config_base_ref) {
        Some(r) => {
            println!("Base ref:   {}", r.cyan());
            r
        }
        None => {
            let detected = git.detect_base_ref()?;
            println!("Base ref:   {} (auto-detected)", detected.cyan());
            detected
        }
    };

    if specs.is_empty() {
        return Err(EnoError::Config("At least one agent is required".to_string()));
    }

    if specs.len() > 4 {
        return Err(EnoError::MaxAgentsExceeded(4));
    }

    // Check that all tools are installed
    println!("\nChecking tools...");
    for spec in &specs {
        if spec.tool.is_installed() {
            println!("  {} {} found", "✓".green(), spec.tool);
        } else {
            return Err(EnoError::ToolNotInstalled {
                tool: spec.tool.to_string(),
                hint: spec.tool.install_hint().to_string(),
            });
        }
    }

    println!("\nCreating session with {} agent(s)...\n", specs.len());

    // Create session
    let session_id = SessionState::generate_id();
    let coordinator = ResourceCoordinator::new();

    // Create worktree directory
    let worktree_base = repo_path.join(".eno-worktrees");
    std::fs::create_dir_all(&worktree_base)?;

    // Create agents
    let mut agent_states = Vec::new();
    for (i, spec) in specs.iter().enumerate() {
        let agent_id = i + 1;
        let worktree_path = worktree_base.join(format!("agent-{}-{}", agent_id, spec.tool));
        let port_base = coordinator.port_base_for_agent(agent_id);

        // Check if branch already exists
        if git.branch_exists(&spec.branch)? {
            println!(
                "  {} Branch '{}' already exists, will use it",
                "⚠".yellow(),
                spec.branch
            );
        }

        // Create worktree
        print!("  Creating worktree: {} ", spec.branch.cyan());
        git.create_worktree(&spec.branch, &worktree_path, &base_ref)?;
        println!("{}", "✓".green());

        agent_states.push(AgentState::new(
            agent_id,
            spec.tool,
            spec.task.clone(),
            spec.branch.clone(),
            worktree_path,
            port_base,
        ));
    }

    // Create session state
    let session = SessionState::new(session_id.clone(), repo_path, base_ref, agent_states)?;

    // Inject context files
    print!("  Injecting context files (CLAUDE.md) ");
    for agent in &session.agents {
        let content = generate_context_file(agent, &session);
        inject_context(&agent.worktree, &content, "CLAUDE.md")?;
    }
    println!("{}", "✓".green());

    // Create tmux session
    let tmux = TmuxManager::new(session.tmux_session.clone())?;

    print!("  Creating tmux session ");
    let first_agent = &session.agents[0];
    let first_spec = &specs[0];
    let first_env = coordinator.env_for_agent(1, &session);

    let first_window = WindowConfig::new(
        first_agent.display_name(),
        first_agent.worktree.clone(),
    )
    .with_env_map(first_env)
    .with_command(first_spec.command.clone().unwrap_or_else(|| first_agent.tool.command().to_string()));

    tmux.create_session(&first_window)?;

    // Add remaining windows
    for (i, agent) in session.agents.iter().skip(1).enumerate() {
        let spec = &specs[i + 1];
        let env = coordinator.env_for_agent(agent.id, &session);

        let window = WindowConfig::new(agent.display_name(), agent.worktree.clone())
            .with_env_map(env)
            .with_command(spec.command.clone().unwrap_or_else(|| agent.tool.command().to_string()));

        tmux.add_window(&window)?;
    }
    println!("{}", "✓".green());

    // Save session
    session.save()?;

    println!("\n{}", "Session created successfully!".green().bold());
    println!("\n{}", "Agents:".bold());
    for agent in &session.agents {
        println!(
            "  {} Agent {}: {} ({})",
            "•".cyan(),
            agent.id,
            agent.task,
            agent.branch.dimmed()
        );
    }

    println!("\n{}", "Commands:".bold());
    println!("  {} - view status", "eno status".cyan());
    println!("  {} - attach to session", "eno attach".cyan());
    println!("  {} - send message", "eno send <agent> <msg>".cyan());
    println!("  {} - cleanup session", "eno cleanup".cyan());

    if !no_attach {
        println!("\n{}", "Attaching to tmux session...".dimmed());
        println!(
            "{}",
            "(Use Ctrl-b n/p to switch agents, Ctrl-b d to detach)".dimmed()
        );
        tmux.attach()?;
    } else {
        println!(
            "\n{}",
            format!("Tmux session: {}", session.tmux_session).dimmed()
        );
        println!("{}", "Run 'eno attach' to attach to the session".dimmed());
    }

    Ok(())
}

fn parse_agent_spec(spec: &str) -> Result<AgentSpec> {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(EnoError::Config(format!(
            "Invalid agent spec '{}'. Format: tool:task",
            spec
        )));
    }

    let tool: Tool = parts[0].parse().map_err(|e: String| EnoError::Config(e))?;
    let task = parts[1].to_string();
    let branch = task_to_branch_name(&task);

    Ok(AgentSpec {
        tool,
        task,
        branch,
        command: None,
    })
}

fn collect_agents_interactive(count: Option<u8>) -> Result<Vec<AgentSpec>> {
    let theme = ColorfulTheme::default();

    // Find installed tools
    let all_tools = [Tool::Claude, Tool::Codex, Tool::Aider, Tool::Cursor, Tool::Custom];
    let available: Vec<_> = all_tools.iter().filter(|t| t.is_installed()).collect();

    if available.is_empty() {
        return Err(EnoError::Config(
            "No AI coding tools found. Install claude, codex, or aider first.".to_string(),
        ));
    }

    println!("\n{}", "Available tools:".dimmed());
    for tool in &available {
        println!("  {} {}", "✓".green(), tool);
    }

    let count = match count {
        Some(c) => c as usize,
        None => {
            Input::with_theme(&theme)
                .with_prompt("How many agents?")
                .default(2)
                .validate_with(|input: &usize| {
                    if *input >= 1 && *input <= 4 {
                        Ok(())
                    } else {
                        Err("Must be between 1 and 4")
                    }
                })
                .interact()?
        }
    };

    let tool_names: Vec<_> = available.iter().map(|t| t.to_string()).collect();
    let mut specs = Vec::new();

    for i in 1..=count {
        println!("\n{}", format!("Agent {}:", i).bold());

        let tool_index = Select::with_theme(&theme)
            .with_prompt("Tool")
            .items(&tool_names)
            .default(0)
            .interact()?;

        let tool = *available[tool_index];

        let task: String = Input::with_theme(&theme)
            .with_prompt("Task")
            .interact_text()?;

        let branch = task_to_branch_name(&task);

        let command = if tool == Tool::Custom {
            let cmd: String = Input::with_theme(&theme)
                .with_prompt("Command")
                .default("bash".to_string())
                .interact_text()?;
            Some(cmd)
        } else {
            None
        };

        println!("  Branch: {}", branch.dimmed());

        specs.push(AgentSpec {
            tool,
            task,
            branch,
            command,
        });
    }

    Ok(specs)
}
