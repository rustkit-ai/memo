# memo × Windsurf

Windsurf reads `.windsurfrules` automatically in every session. `memo setup` writes instructions into that file telling Windsurf to maintain the memory loop: inject context at session start, log after file edits, recap at session end.

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

## What gets written

**`.windsurfrules`**:

```markdown
# memo — persistent agent memory
- At session start: run `memo inject --windsurf` to load context from previous sessions
- After modifying any file: run `memo log "modified {filename}: {one-line reason}"`
- When you identify something to fix later: run `memo log "todo: {description}"`
- At session end: run `memo recap "{what was done} — next: {what comes next}"` then `memo inject --windsurf`

<!-- memo:start -->
## memo context
last: (no entries yet)
<!-- memo:end -->
```

Windsurf loads `.windsurfrules` automatically — no additional configuration needed.

---

## The session loop

```
Open Windsurf
      │
      ▼
Windsurf reads .windsurfrules
      │
      ▼
Agent runs: memo inject --windsurf
      │  (updates the context block in .windsurfrules)
      ▼
Agent reads the context — knows where it left off
      │
      ▼
You work — agent logs after each file edit:
  memo log "modified src/db/migrate.rs: added pg16 migration"
  memo log "todo: update connection pool config for pg16 defaults"
      │
      ▼
At session end:
  memo recap "migrated DB to PostgreSQL 16 — next: update connection pool config"
  memo inject --windsurf
      │
      ▼
Next session starts with full context
```

---

## What the context block looks like

```
## memo context
recap (2026-03-15): "migrated DB to PostgreSQL 16 — next: update connection pool config"
recent (2026-03-15): "modified src/db/migrate.rs: added pg16 migration"
recent (2026-03-15): "modified src/db/pool.rs: extracted pool config"
todo: update connection pool config for pg16 defaults
recent tags: db · migration
```

---

## Example session

```
You: [opens Windsurf, starts a new session]

Windsurf: Based on memo — last session you migrated the database to PostgreSQL 16.
          Open todo: update the connection pool config for pg16 defaults.
          Want to tackle that now?
```

---

## Key commands

```sh
memo recap "<summary>"    # log end-of-session summary
memo todo list            # see all open todos
memo todo done <id>       # mark a todo as done
memo bootstrap            # import recent git commits as memory entries
memo inject --windsurf    # manually update .windsurfrules
memo inject --all         # update all configured agent files at once
```

---

## Verify setup

```sh
cat .windsurfrules
```

You should see the instructions and the `<!-- memo:start -->` block.
