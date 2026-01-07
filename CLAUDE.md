# Eno - Agent Orchestration Tool

Like the composer Brian Eno, this tool is minimalist and simple.

## Overview

Eno orchestrates parallel AI coding agents with isolated worktrees, resource coordination, and tmux-based workflow.

## Architecture

```
src/
├── main.rs           # Entry point and command dispatch
├── cli.rs            # Clap CLI definitions
├── config.rs         # Configuration types (YAML parsing)
├── error.rs          # Error types (thiserror)
├── git.rs            # Git worktree management
├── tmux.rs           # Tmux session/window management
├── session.rs        # Session state persistence (JSON)
├── coordinator.rs    # Resource allocation (ports, docker)
├── context.rs        # Context file generation (CLAUDE.md injection)
└── commands/
    ├── mod.rs
    ├── start.rs      # Create new swarm session
    ├── status.rs     # Show session status
    ├── send.rs       # Send/broadcast messages
    ├── lock.rs       # Resource locking
    ├── attach.rs     # Attach to tmux
    └── cleanup.rs    # Remove session resources
```

## Key Concepts

### Session State
- Stored in `/tmp/eno-sessions/<session-id>/state.json`
- Tracks agents, worktrees, branches, port allocations
- Persists across CLI invocations

### Resource Isolation
- Each agent gets a port range (default: 100 ports starting at 9100)
- Docker prefixes prevent container naming collisions
- Environment variables injected: `ENO_AGENT_ID`, `ENO_PORT_BASE`, etc.

### Context Injection
- `CLAUDE.md` file injected into each worktree
- Contains task, resource allocations, coordination rules
- Added to `.git/info/exclude` to prevent commits

### Locking
- File-based locks in `<state_dir>/locks/`
- Uses `fs2::FileExt::try_lock_exclusive()`
- Lock info stored in JSON for display

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## Common Modifications

### Adding a new command
1. Add variant to `Commands` enum in `cli.rs`
2. Create `commands/<name>.rs`
3. Add to `commands/mod.rs`
4. Handle in `main.rs` match

### Adding environment variables
Modify `ResourceCoordinator::env_for_agent()` in `coordinator.rs`

### Changing context file format
Modify `generate_context_file()` in `context.rs`

## Dependencies

- `clap` - CLI parsing
- `serde` / `serde_json` / `serde_yaml` - Serialization
- `chrono` - Date/time
- `colored` - Terminal colors
- `dialoguer` - Interactive prompts
- `tabled` - Table formatting
- `fs2` - File locking
- `which` - Command detection
