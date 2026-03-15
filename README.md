# memo

Persistent memory for AI coding agents. Single Rust binary, zero runtime dependencies.

Works with Claude Code, Cursor, Aider, and any agent that can run shell commands.

## The problem

Every new AI session starts from zero. The agent re-explores project structure, re-discovers conventions, re-learns what was done last time. `memo` fixes this by injecting a compact context block at session start — written by the agent, read next time.

## Quickstart with Claude Code

```sh
memo setup
```

That's it. This writes instructions into `CLAUDE.md` and installs a Stop hook in `.claude/settings.json` so `memo inject --claude` runs automatically at the end of every session.

## Manual workflow

```sh
# Start of session
memo inject

# During session — log important decisions
memo log "switched to WAL mode for SQLite, fixes concurrent write issue"
memo log "refactored auth middleware" --tag refactor

# End of session
memo log "todo: fix token refresh in utils.rs:42"
```

**inject output (~80 tokens):**
```
## memo context
last: 2026-03-14 — "refactored auth, broke token refresh"
todo: fix utils.rs:42 — token refresh logic
recent tags: bug · refactor · auth
```

## Install

**cargo:**
```sh
cargo install memo-agent
```

**curl (Linux/macOS):**
```sh
curl -fsSL https://github.com/rustkit-ai/memo/releases/latest/download/install.sh | sh
```

**brew:**
```sh
brew install rustkit-ai/tap/memo
```

## Commands

```
memo setup                         # configure Claude Code integration (one-time)
memo init                          # initialize project memory
memo log <message>                 # save a memory entry
memo log <message> --tag X         # with one or more tags
memo log -                         # read message from stdin
memo inject                        # print compact context block (stdout)
memo inject --claude               # write block directly into CLAUDE.md
memo inject --format json          # JSON output for programmatic use
memo inject --since 7d             # limit context to last 7 days
memo list                          # show last 10 entries
memo list --all                    # show all entries
memo list --tag bug                # filter by tag
memo search <query>                # full-text search entries
memo delete <id>                   # delete a specific entry
memo tags                          # list all tags with usage counts
memo clear                         # clear all memory for current project
memo stats                         # entry count, estimated tokens saved
```

## Storage

SQLite at `~/.local/share/memo/<project-hash>.db`. Project identified by git remote URL (fallback: absolute path). No config files, no daemons, no background processes.

## Add to CLAUDE.md manually

```markdown
## memo — persistent agent memory
- Run `memo log "<what you did>"` after each significant task
- Run `memo log "todo: <next step>"` before ending the session
- Run `memo inject` at the start of a session to recall context
```

Or run `memo setup` to write this automatically.

## License

MIT
