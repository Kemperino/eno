# Eno - AI Agent Coordination Guide

This document explains how AI agents should interact with the eno orchestration system.

## For AI Agents Working in an Eno Session

When you're spawned as part of an eno swarm, you'll find a `CLAUDE.md` file in your worktree root. This file contains:

1. **Your task** - What you're supposed to work on
2. **Your resource allocation** - Ports, docker prefixes you can use
3. **Coordination rules** - How to avoid conflicts with other agents
4. **Other agents** - What your teammates are working on

## Environment Variables

Eno injects these environment variables into your shell:

| Variable | Description | Example |
|----------|-------------|---------|
| `ENO_AGENT_ID` | Your agent number (1-4) | `1` |
| `ENO_AGENT_COUNT` | Total agents in session | `3` |
| `ENO_SESSION_ID` | Session identifier | `20250107-143022` |
| `ENO_PORT_BASE` | Start of your port range | `9100` |
| `ENO_PORT_RANGE` | Size of your port range | `100` |
| `ENO_DOCKER_PREFIX` | Docker container prefix | `eno-1-` |
| `COMPOSE_PROJECT_NAME` | Docker Compose project | `eno-agent-1` |
| `ENO_STATE_DIR` | Session state directory | `/tmp/eno-sessions/...` |
| `ENO_LOCK_DIR` | Lock files directory | `/tmp/eno-sessions/.../locks` |
| `ENO_HTTP_PORT` | Suggested HTTP port | `9100` |
| `ENO_HTTPS_PORT` | Suggested HTTPS port | `9101` |
| `ENO_DB_PORT` | Suggested database port | `9132` |
| `ENO_REDIS_PORT` | Suggested Redis port | `9179` |

## Resource Isolation Rules

### Ports
- **Only use ports in your assigned range** (e.g., 9100-9199 for Agent 1)
- Use `$ENO_PORT_BASE` as your base and add offsets
- Common pattern: HTTP on base, HTTPS on base+1, DB on base+32

### Docker
- **Prefix all container names** with `$ENO_DOCKER_PREFIX` (e.g., `eno-1-myapp`)
- Docker Compose automatically uses `$COMPOSE_PROJECT_NAME`
- Networks are isolated per compose project

### Shared Resources
For resources that can't be parallelized (like running the full integration test suite):

```bash
# Acquire a lock before using shared resource
eno lock acquire integration-tests

# ... run your tests ...

# Release when done
eno lock release integration-tests
```

Lock names are arbitrary strings. Common ones:
- `integration-tests` - Full test suite
- `database-migrations` - Schema changes
- `docker-compose` - Starting/stopping compose
- `build-cache` - Shared build artifacts

## Communication

### Receiving Messages
Messages from other agents or the operator appear in your terminal prefixed with:
```
# [eno] Message: <content>
# [eno] Broadcast: <content>
```

### Checking Status
```bash
eno status  # See all agents and their tasks
```

## Best Practices

1. **Read CLAUDE.md first** - Understand your task and constraints
2. **Respect port ranges** - Don't bind to ports outside your range
3. **Use locks for shared resources** - Prevent race conditions
4. **Keep changes focused** - Stay in your lane, don't modify files other agents are working on
5. **Commit frequently** - Small, focused commits on your branch
6. **Check other agents' tasks** - Avoid duplicating work

## Example Workflow

```bash
# 1. Check your context
cat CLAUDE.md

# 2. See what other agents are doing
eno status

# 3. Start your dev server on your assigned port
PORT=$ENO_HTTP_PORT npm run dev

# 4. When you need shared resources
eno lock acquire integration-tests
npm test
eno lock release integration-tests

# 5. Commit your work
git add .
git commit -m "Implement feature X"
```

## Troubleshooting

### Port already in use
Check if another agent is using your port by mistake:
```bash
lsof -i :$ENO_HTTP_PORT
```

### Lock stuck
If a lock is held by a dead process:
```bash
eno lock list        # See who holds it
eno lock steal <resource>  # Force release (use carefully)
```

### Can't find session
```bash
eno status  # Will show error if no session
# Session might have been cleaned up, ask operator to restart
```
