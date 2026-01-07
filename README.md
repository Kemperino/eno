```
                        ███████╗███╗   ██╗ ██████╗
                        ██╔════╝████╗  ██║██╔═══██╗
                        █████╗  ██╔██╗ ██║██║   ██║
                        ██╔══╝  ██║╚██╗██║██║   ██║
                        ███████╗██║ ╚████║╚██████╔╝
                        ╚══════╝╚═╝  ╚═══╝ ╚═════╝

              "Honor thy error as a hidden intention"
                                         - Brian Eno


                            ██████████████████
                        ████░░░░░░░░░░░░░░░░░░████
                      ██░░░░░░░░░░░░░░░░░░░░░░░░░░██
                    ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██
                  ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██
                ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██
                ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██
              ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██
              ██░░░░░░████░░░░░░░░░░░░░░░░░░████░░░░░░░░░░██
              ██░░░░██    ██░░░░░░░░░░░░░░██    ██░░░░░░░░██
              ██░░░░██    ██░░░░░░░░░░░░░░██    ██░░░░░░░░██
              ██░░░░░░████░░░░░░░░░░░░░░░░░░████░░░░░░░░░░██
              ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██
              ██░░░░░░░░░░░░░░░░████░░░░░░░░░░░░░░░░░░░░░░██
                ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██
                ██░░░░░░░░░░████████████████░░░░░░░░░░░░██
                  ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██
                    ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░██
                      ████░░░░░░░░░░░░░░░░░░░░░░████
                          ████████████████████████
```

# eno

Like the composer, minimalist and simple.

Orchestrate parallel AI coding agents with isolated git worktrees, resource coordination, and tmux-based workflow.

## Install

```bash
cargo install --path .
```

## Quick Start

```bash
# Interactive setup
eno start -i

# Direct specification
eno start --agent claude:"Refactor auth to OAuth2" --agent codex:"Add auth tests"

# From config file
eno start --config eno.yaml
```

## Commands

| Command | Description |
|---------|-------------|
| `eno start` | Start a new agent swarm session |
| `eno status` | Show status of current session |
| `eno attach` | Attach to the tmux session |
| `eno send <n> <msg>` | Send message to agent n |
| `eno broadcast <msg>` | Message all agents |
| `eno cleanup` | Clean up the session |

## How It Works

1. **Isolated Worktrees** - Each agent gets its own git worktree with a task-based branch
2. **Resource Coordination** - Agents get isolated port ranges (9100-9199, 9200-9299, etc.)
3. **Context Injection** - Each worktree gets a `CLAUDE.md` with task and coordination rules
4. **Tmux Integration** - All agents run in named tmux windows

## Example Session

```
$ eno start -i

🎵 Eno Agent Orchestrator
   Like the composer, minimalist and simple.

Repository: /home/user/myapp
Base ref:   origin/main (auto-detected)

Available tools:
  ✓ claude
  ✓ codex

How many agents? 2

Agent 1:
  Tool: claude
  Task: Refactor authentication to use JWT
  Branch: refactor/refactor-authentication-use-jwt

Agent 2:
  Tool: codex
  Task: Add comprehensive test coverage
  Branch: test/add-comprehensive-test-coverage

Checking tools...
  ✓ claude found
  ✓ codex found

Creating session with 2 agent(s)...

  Creating worktree: refactor/refactor-authentication-use-jwt ✓
  Creating worktree: test/add-comprehensive-test-coverage ✓
  Injecting context files (CLAUDE.md) ✓
  Creating tmux session ✓

Session created successfully!
```

## Configuration File

```yaml
# eno.yaml
agents:
  - tool: claude
    task: Implement OAuth2 authentication
    branch: feature/oauth2  # optional, auto-generated if omitted

  - tool: codex
    task: Add API tests
```

## Supported Tools

| Tool | Command | Install |
|------|---------|---------|
| Claude | `claude` | `npm install -g @anthropic-ai/claude-code` |
| Codex | `codex` | `npm install -g @openai/codex` |

## License

MIT
