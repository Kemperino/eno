use colored::Colorize;

use crate::error::{EnoError, Result};
use crate::session::SessionState;
use crate::tmux::TmuxManager;

pub fn run_send(agent: u8, message: String) -> Result<()> {
    let session = SessionState::find_active()?.ok_or(EnoError::NoActiveSession)?;

    let agent_state = session
        .get_agent(agent as usize)
        .ok_or(EnoError::AgentNotFound(agent as usize))?;

    let tmux = TmuxManager::new(session.tmux_session.clone())?;

    // Format the message with a visual indicator
    let formatted = format!("# [eno] Message: {}", message);

    // Send to the agent's window
    tmux.send_keys_to_window(&agent_state.display_name(), &formatted)?;

    println!(
        "{} Sent to Agent {}: {}",
        "✓".green(),
        agent,
        message.dimmed()
    );

    Ok(())
}

pub fn run_broadcast(message: String) -> Result<()> {
    let session = SessionState::find_active()?.ok_or(EnoError::NoActiveSession)?;

    let tmux = TmuxManager::new(session.tmux_session.clone())?;

    // Format the message
    let formatted = format!("# [eno] Broadcast: {}", message);

    // Send to all agents
    for agent in &session.agents {
        tmux.send_keys_to_window(&agent.display_name(), &formatted)?;
    }

    println!(
        "{} Broadcast to {} agents: {}",
        "✓".green(),
        session.agents.len(),
        message.dimmed()
    );

    Ok(())
}
