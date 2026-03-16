# memo × Claude Code

Claude Code reads `CLAUDE.md` automatically at the start of every session. `memo setup` installs **three hooks** and writes a context block into `CLAUDE.md` — the full memory loop runs with zero manual steps, ever.

---

## Setup

Run once in your project root:

```sh
memo setup
```

Then bootstrap from your git history so the agent has context from day one:

```sh
memo bootstrap
```

---

## What gets installed

**Three hooks in `.claude/settings.json`:**

| Hook | Trigger | What it does |
|---|---|---|
| `PostToolUse` | After every Write / Edit / MultiEdit | Runs `memo capture` — auto-logs the file with a code description |
| `UserPromptSubmit` | At the start of each session | Runs `memo inject --claude --once` — injects fresh context |
| `Stop` | When you close Claude Code | Runs `memo inject --claude` — saves context for next session |

**`.claude/settings.json`** (excerpt):

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [{ "type": "command", "command": "memo capture" }]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [{ "type": "command", "command": "memo inject --claude --once" }]
      }
    ],
    "Stop": [
      {
        "hooks": [{ "type": "command", "command": "memo inject --claude" }]
      }
    ]
  }
}
```

**`CLAUDE.md`** (excerpt):

```markdown
<!-- memo:instructions:start -->
## memo — persistent agent memory
- At session start: run `memo inject --claude` to load context from previous sessions
- After modifying any file: run `memo log "modified {filename}: {one-line reason}"`
- When you identify something to fix later: run `memo log "todo: {description}"`
- At session end: run `memo recap "{what was done} — next: {what comes next}"` then `memo inject --claude`
<!-- memo:instructions:end -->

<!-- memo:start -->
## memo context
last: (no entries yet)
<!-- memo:end -->
```

---

## The session loop

```
Open Claude Code
      │
      ▼
UserPromptSubmit hook → memo inject --claude --once
      │  (injects context only if new entries exist)
      ▼
Claude reads CLAUDE.md ←── recap + recent entries + open todos
      │
      ▼
You work — Claude edits files
      │
      ▼
PostToolUse hook → memo capture
      │  (logs "wrote src/auth.rs: added fn handle_login"
      │   or  "edited src/db/pool.rs: added fn connect_pool"
      │   or  "edited src/auth.rs (3 changes)" if no pattern matched)
      ▼
Claude logs semantic context:
  memo log "modified src/auth.rs: extracted JWT validation"
  memo log "todo: add refresh token endpoint"
      │
      ▼
At session end:
  memo recap "implemented JWT auth — next: refresh token endpoint"
      │
      ▼
You close Claude Code
      │
      ▼
Stop hook → memo inject --claude
      │
      ▼
CLAUDE.md updated silently — ready for next session
```

---

## What the context block looks like

```
## memo context
recap (2026-03-15): "implemented JWT auth — next: refresh token endpoint"
recent (2026-03-15): "wrote src/auth/jwt.rs: added fn validate_token"
recent (2026-03-15): "edited src/auth/jwt.rs: added fn refresh_token"
recent (2026-03-15): "modified src/auth/jwt.rs: extracted JWT validation"
todo: add refresh token endpoint
recent tags: auth · jwt · auto
```

---

## Example session

```
You: where did we leave off?

Claude: Based on memo — last session you implemented JWT auth.
        The recap says: "next: refresh token endpoint".
        There's an open todo for that. Should I start there?
```

---

## Key commands

```sh
memo recap "<summary>"    # log end-of-session summary (shown prominently next session)
memo todo list            # see all open todos
memo todo done <id>       # mark a todo as done
memo bootstrap            # import recent git commits as memory entries
memo inject --claude      # manually update CLAUDE.md
memo doctor               # check hooks, DB, and all agent config files
```

---

## Verify setup

```sh
memo doctor
```

Example output on a healthy project:

```
Core
  ✓ binary: /usr/local/bin/memo
  ✓ database: ~/.local/share/memo/abc12345.db (42 entries)

Claude Code
  ✓ CLAUDE.md: memo context block present
  ✓ hook Stop: memo inject --claude
  ✓ hook UserPromptSubmit: memo inject --claude --once
  ✓ hook PostToolUse: memo capture (Write|Edit|MultiEdit)

Cursor
  ✓ .cursor/rules/memo.mdc: alwaysApply: true
  ✓ .cursor/rules/memo.mdc: memo context block present

All checks passed.
```

If anything is missing, run `memo setup` again — it is idempotent.
