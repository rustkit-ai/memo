# Changelog

All notable changes to this project will be documented in this file.

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
