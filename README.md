# memo — Persistent memory for AI coding agents

**Stop starting every AI session from zero.**

`memo` gives AI agents like Claude Code, Cursor, and Aider a persistent memory across sessions. One binary, zero dependencies, works in any project.

```
$ memo inject

## memo context
last: 2026-03-15 — "refactored auth middleware, JWT now stateless"
todo: fix token refresh in utils.rs:42
recent tags: refactor · auth · bug
```

---

## The problem

Every new AI session starts from scratch. The agent re-reads files it already read, re-discovers conventions it already learned, asks questions it already asked. On a large codebase this costs hundreds of tokens and minutes of context-building — every single time.

`memo` fixes this with a compact context block (~80 tokens) injected at session start. Written by the agent, read next time.

---

## Install

**cargo** (recommended):
```sh
cargo install memo-agent
```

**curl** (Linux / macOS):
```sh
curl -fsSL https://github.com/rustkit-ai/memo/releases/latest/download/install.sh | sh
```

**brew**:
```sh
brew install rustkit-ai/tap/memo
```

---

## Quickstart

### With Claude Code (automatic)

Run once in your project:
```sh
memo setup
```

That's it. `memo setup`:
- Writes agent instructions into `CLAUDE.md`
- Installs a Stop hook in `.claude/settings.json` so the context block refreshes automatically at the end of every session

From then on, Claude remembers what happened last time without you doing anything.

### With any other agent

Add this to your `CLAUDE.md` / `AGENTS.md` / system prompt:

```markdown
## Memory
- Run `memo inject` at the start of every session
- Run `memo log "<what you did>"` after each significant task
- Run `memo log "todo: <next step>"` before ending the session
```

---

## How it works

```
Session 1                          Session 2
─────────────────────────────────  ─────────────────────────────────
agent runs `memo inject`    →      reads last context (~80 tokens)
agent works on the project
agent runs `memo log "..."`  →     stored in SQLite
session ends, hook fires    →      CLAUDE.md updated automatically
```

Storage: SQLite at `~/.local/share/memo/<project-hash>.db`.
Project identity: git remote URL hash (fallback: absolute path hash).
No config files. No daemons. No background processes.

---

## Commands

| Command | Description |
|---|---|
| `memo setup` | One-time Claude Code integration (CLAUDE.md + Stop hook) |
| `memo init` | Initialize project memory |
| `memo log "<message>"` | Save a memory entry |
| `memo log "<message>" --tag refactor` | Save with one or more tags |
| `memo log -` | Read message from stdin |
| `memo inject` | Print context block to stdout |
| `memo inject --claude` | Write context block into CLAUDE.md |
| `memo inject --since 7d` | Limit context to last 7 days |
| `memo inject --format json` | JSON output for programmatic use |
| `memo list` | Show last 10 entries |
| `memo list --all` | Show all entries |
| `memo list --tag bug` | Filter by tag |
| `memo search <query>` | Full-text search entries |
| `memo delete <id>` | Delete a specific entry |
| `memo tags` | List all tags with usage counts |
| `memo stats` | Entry count + token savings estimate |
| `memo clear` | Clear all memory for current project |

---

## Real-world example

```sh
# Agent starts a session
$ memo inject
## memo context
last: 2026-03-14 — "added Stripe webhook handler"
todo: write tests for webhook signature verification
recent tags: stripe · payments · todo

# Agent works, then logs what it did
$ memo log "wrote tests for webhook handler, all passing" --tag stripe
logged: wrote tests for webhook handler, all passing

$ memo log "todo: deploy to staging and verify with Stripe dashboard"
logged: todo: deploy to staging and verify with Stripe dashboard

# Next session — agent knows exactly where to pick up
$ memo inject
## memo context
last: 2026-03-15 — "wrote tests for webhook handler, all passing"
todo: deploy to staging and verify with Stripe dashboard
recent tags: stripe · payments · todo
```

---

## Why not just use CLAUDE.md?

You can write to `CLAUDE.md` manually — but that means you do the work. `memo` lets the **agent** maintain its own memory, automatically, without human intervention between sessions.

---

## License

MIT — [rustkit-ai/memo](https://github.com/rustkit-ai/memo)
