---
status: done
type: feature
priority: medium
area: testing
---

## Test coverage for untested modules

Purpose: Core modules lib.rs, theme.rs, and gloss/filter.rs have zero tests. The
Config layering logic, color conversion, theme discovery, and filter pipeline are
all unverified. Integration tests directory is empty.

Planned tests:
- lib.rs: Config::load() with layered overrides, color parsing edge cases,
  ratatui_to_termcolor round-trip, workspace_root directory traversal,
  resolve_file search order
- theme.rs: load_themes() discovery across directories, fallback to default
- gloss/filter.rs: stdin→stdout pipeline (may need exploration on how to test
  interactive TUI code)
- Integration tests: plan add/open/delete lifecycle, gloss file viewing
