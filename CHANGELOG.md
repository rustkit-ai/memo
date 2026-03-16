# Changelog

All notable changes to this project will be documented in this file.

## [0.1.6] - 2026-03-16

### Added
- `memo capture` now extracts a code description from the diff payload: `"wrote src/auth.rs: added fn handle_login"` instead of just `"wrote src/auth.rs"`. Detects fn/struct/enum/trait/impl/class/interface/route patterns across Rust, TypeScript, JavaScript, and Python.
- `memo doctor` now checks all three Claude Code hooks (Stop, UserPromptSubmit, PostToolUse) individually, and checks agent config files for Cursor, Windsurf, and Copilot if detected in the project.
- 31 new tests (26 unit tests for capture helpers + 5 integration tests for `capture` and `doctor`).

### Changed
- `memo capture` canonicalizes file paths before stripping the project prefix, fixing relative path display on macOS (symlinked `/var` → `/private/var`).

## [0.1.5] - 2026-03-16

### Added
- `memo inject --cursor` — write context block into `.cursor/rules/memo.mdc`
- `memo inject --windsurf` — write context block into `.windsurfrules`
- `memo inject --copilot` — write context block into `.github/copilot-instructions.md`
- `memo setup` now writes an initial empty context block into all agent files at setup time
- Agent-specific inject commands in setup instructions (`--claude`, `--cursor`, `--windsurf`, `--copilot`)
- Agent guides in `docs/agents/` for Claude Code, Cursor, Windsurf, and GitHub Copilot

### Changed
- Each agent's rules file now instructs it to run its dedicated inject command at session start

## [0.1.4] - 2026-03-16

### Added
- Multi-agent support: `memo setup` now configures Cursor, Windsurf, and GitHub Copilot in addition to Claude Code
- Cursor: writes `.cursor/rules/memo.mdc` with `alwaysApply: true`
- Windsurf: writes `.windsurfrules`
- GitHub Copilot: writes `.github/copilot-instructions.md` (appends if file exists)

### Changed
- README updated to document all four supported agents

## [0.1.3] - 2026-03-16

### Changed
- Refactored `memo-core`: `map_row()` helper eliminates repeated tuple mapping, `limit_val()` + `LIMIT -1` removes duplicated match/format patterns, `fetch()` helper centralises query execution
- Moved `open_in_memory()` to `#[cfg(test)]` block
- `OutputFormat` is now a proper `ValueEnum` instead of a free string
- `print_entry()` helper eliminates duplication between `list` and `search` commands
- `parse_duration()` simplified with `split_at` + `match`
- Fixed `save()` to return `last_insert_rowid()` instead of `rows_affected()`

## [0.1.2] - 2026-03-15

### Changed
- Migrate to Rust edition 2024
- Fix `install.sh` macOS compatibility (remove unused sha256sum call)
- Rewrite README with full session lifecycle explanation

### Fixed
- Release workflow: use `llvm-strip` for cross-compiled Linux targets (fixes aarch64-linux binary)
- `memo setup`: idempotent Stop hook installation
- CLAUDE.md replace off-by-one when `<!-- memo:end -->` marker is absent

## [0.1.1] - 2026-03-15

### Added
- `memo setup` — one-command Claude Code integration (CLAUDE.md + Stop hook)
- `memo search <query>` — full-text search entries
- `memo delete <id>` — delete a specific entry
- `memo tags` — list all tags with usage counts
- `memo list --tag <tag>` — filter entries by tag
- `memo inject --since <duration>` — limit context to recent entries (1d, 7d, 24h, 1w)
- `memo log -` — read message from stdin
- Multiple `--tag` flags support
- Integration tests

### Fixed
- SQL injection in `list()` — now uses parameterized queries
- Windows compatibility — use `dirs::data_local_dir()` instead of `$HOME`
- `sessions.ended_at` dead column removed from schema
- Token estimate in `stats` now uses actual inject block size

### Changed
- `memo init` now prints suggested CLAUDE.md snippet
- `Cargo.lock` committed (binary crate best practice)

## [0.1.0] - 2026-03-15

Initial release.

### Added
- `memo init`, `memo log`, `memo inject`, `memo list`, `memo clear`, `memo stats`
- SQLite storage at `~/.local/share/memo/<project-hash>.db`
- Project detection via git remote URL hash (fallback: path hash)
- `memo inject --claude` to write context into CLAUDE.md
- `memo inject --format json` for programmatic use
- CI workflow (ubuntu / macos / windows)
- Release workflow with cross-platform binaries
