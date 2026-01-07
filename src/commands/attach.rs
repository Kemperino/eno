use colored::Colorize;

use crate::error::{EnoError, Result};
use crate::session::SessionState;
use crate::tmux::TmuxManager;

pub fn run_attach() -> Result<()> {
    let session = SessionState::find_active()?.ok_or(EnoError::NoActiveSession)?;

    println!(
        "Attaching to session: {}",
        session.tmux_session.cyan()
    );
    println!(
        "{}",
        "(Use Ctrl-b n/p to switch agents, Ctrl-b d to detach)".dimmed()
    );

    let tmux = TmuxManager::new(session.tmux_session)?;
    tmux.attach()?;

    Ok(())
}
