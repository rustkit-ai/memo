# memo-agent × Cursor

Cursor reads rules from `.cursor/rules/` automatically. `memo setup` writes a rule file with `alwaysApply: true` that instructs Cursor to maintain the memory loop: inject context at session start, log after file edits, recap at session end.

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

**`.cursor/rules/memo.mdc`**:

```markdown
---
description: memo persistent memory instructions
alwaysApply: true
---

## memo — persistent agent memory
- At session start: run `memo inject --cursor` to load context from previous sessions
- After modifying any file: run `memo log "modified {filename}: {one-line reason}"`
- When you identify something to fix later: run `memo log "todo: {description}"`
- At session end: run `memo recap "{what was done} — next: {what comes next}"` then `memo inject --cursor`

<!-- memo:start -->
## memo context
last: (no entries yet)
<!-- memo:end -->
```

Because `alwaysApply: true` is set, Cursor loads this rule file in every chat and Composer session automatically.

---

## The session loop

```
Open Cursor
      │
      ▼
Cursor loads .cursor/rules/memo.mdc  (alwaysApply: true)
      │
      ▼
Agent runs: memo inject --cursor
      │  (updates the context block in memo.mdc)
      ▼
Agent reads the context — knows where it left off
      │
      ▼
You work — agent logs after each file edit:
  memo log "modified src/payments/service.rs: added idempotency keys"
  memo log "todo: write integration test for duplicate charge case"
      │
      ▼
At session end:
  memo recap "added idempotency to payment service — next: integration tests"
  memo inject --cursor
      │
      ▼
Next session starts with full context
```

---

## What the context block looks like

```
## memo context
recap (2026-03-15): "added idempotency to payment service — next: integration tests"
recent (2026-03-15): "modified src/payments/service.rs: added idempotency keys"
todo: write integration test for duplicate charge case
recent tags: payments · idempotency
```

---

## Example session

```
You: [opens Cursor, starts a new chat]

Cursor: Based on memo — last session you added idempotency keys to the payment
        service. Open todo: write an integration test for the duplicate charge
        case. Should I start there?
```

---

## Key commands

```sh
memo recap "<summary>"    # log end-of-session summary
memo todo list            # see all open todos
memo todo done <id>       # mark a todo as done
memo bootstrap            # import recent git commits as memory entries
memo inject --cursor      # manually update .cursor/rules/memo.mdc
memo inject --all         # update all configured agent files at once
```

---

## Verify setup

```sh
cat .cursor/rules/memo.mdc
```

You should see the `alwaysApply: true` frontmatter and the `<!-- memo:start -->` block.
