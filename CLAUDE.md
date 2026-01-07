# Eno Agent Context

You are **Agent 1** of **2** in a coordinated eno swarm session.

## Your Task

add mit license

## Git Branch

You are working on branch: `feature/add-mit-license`

Base ref: `origin/main`

## Resource Allocation

| Resource | Your Assignment |
|----------|----------------|
| Port range | 9100-9199 |
| Docker prefix | `eno-1-` |
| Compose project | `eno-agent-1` |
| Test DB | `test_agent_1` |

### Common Port Mappings

| Service | Port | Env Var |
|---------|------|--------|
| HTTP | 9100 | `$ENO_HTTP_PORT` |
| HTTPS | 9101 | `$ENO_HTTPS_PORT` |
| Database | 9132 | `$ENO_DB_PORT` |
| Redis | 9179 | `$ENO_REDIS_PORT` |

## Coordination Rules

1. **Ports**: Only bind to ports in your range (9100-9199)
2. **Docker**: Prefix all container names with `eno-1-`
3. **Shared resources**: Use locks before accessing:
   ```bash
   eno lock acquire integration-tests
   # ... run tests ...
   eno lock release integration-tests
   ```

## Other Agents (for awareness)

| Agent | Tool | Task | Branch |
|-------|------|------|--------|
| 2 | cursor | change brian eno quote | `feature/change-brian-eno-quote` |

## Session Commands

```bash
eno status          # See all agents and their status
eno send 2 "msg"    # Message another agent
eno broadcast "msg" # Message all agents
eno lock list       # View active locks
eno lock acquire <resource>  # Acquire a lock
eno lock release <resource>  # Release a lock
```

## Environment Variables

The following environment variables are available:

```bash
ENO_AGENT_ID=1          # Your agent ID
ENO_AGENT_COUNT=2       # Total number of agents
ENO_SESSION_ID=20260107-174855  # Session identifier
ENO_PORT_BASE=9100       # Start of your port range
ENO_PORT_RANGE=100       # Size of your port range
```
