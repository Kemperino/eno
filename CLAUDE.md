# Eno - Agent Orchestration Tool

Like the composer Brian Eno, this tool is minimalist and simple.

## Overview

Eno orchestrates parallel AI coding agents (Claude, Codex) with isolated git worktrees, resource coordination, and tmux-based workflow.

## Architecture

```
src/
├── main.rs           # Entry point and command dispatch
├── cli.rs            # Clap CLI definitions (Tool enum: Claude, Codex)
├── config.rs         # Configuration types (YAML parsing, branch name generation)
├── error.rs          # Error types (thiserror)
├── git.rs            # Git worktree management
├── tmux.rs           # Tmux session/window management
├── session.rs        # Session state persistence (JSON)
├── coordinator.rs    # Resource allocation (ports, docker)
├── context.rs        # Context file generation (.eno-context.md)
└── commands/
    ├── mod.rs
    ├── start.rs      # Create new swarm session
    ├── status.rs     # Show session status
    ├── send.rs       # Send/broadcast messages
    ├── attach.rs     # Attach to tmux
    └── cleanup.rs    # Remove session resources
```

## Key Concepts

### Session State
- Stored in `/tmp/eno-sessions/<session-id>/state.json`
- Tracks agents, worktrees, branches, port allocations
- Persists across CLI invocations

### Resource Isolation
- Each agent gets a port range (100 ports: 9100-9199, 9200-9299, etc.)
- Docker prefixes prevent container naming collisions
- Environment variables injected: `ENO_AGENT_ID`, `ENO_PORT_BASE`, etc.

### Context Injection
- `.eno-context.md` file injected into each worktree (not CLAUDE.md to avoid conflicts)
- Contains task, resource allocations, coordination rules
- Added to `.git/info/exclude` to prevent commits

### Agent Launch
- Claude: `claude --dangerously-skip-permissions 'task'`
- Codex: `codex --dangerously-skip-permissions 'task'`

## Building

```bash
cargo build --release
cargo install --path .
```

## Testing

```bash
cargo test
```

## Commands

- `eno start -i` - Interactive session setup
- `eno start --agent claude:"task"` - Direct agent specification
- `eno status` - Show session status
- `eno attach` - Attach to tmux session
- `eno send <n> "msg"` - Send message to agent
- `eno broadcast "msg"` - Message all agents
- `eno cleanup` - Clean up session
