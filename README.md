# tablethat

**plan** + **gloss** — task management and markdown viewing.

## Two tools

### plan

Task management with kanban TUI. Tasks live in `.plan/*.md` as markdown files with YAML frontmatter.

```
plan                    # list tasks
plan -k                 # kanban view
plan tui                # interactive TUI
plan --lint             # validate frontmatter
plan --init             # scaffold .plan/ directory
```

### gloss

Markdown viewer — filter mode for vim/pagers, TUI for browsing.

```
gloss README.md         # TUI single file viewer
gloss docs/             # TUI directory browser
cat file.md | gloss     # filter mode (stdin → stdout)
:%!gloss                # vim filter
```

## Install

```
cargo install --path .
```

## Task format

Each task is a markdown file in `.plan/`:

```markdown
---
status: open
type: bug
priority: high
area: backend
---

## Notes

Description and context.
```

**Status:** `idea` · `backlog` · `open` · `in-progress` · `blocked` · `done`
**Type:** `bug` · `feature` · `chore` · `decision` · `perf`
**Priority:** `high` · `medium` · `low`

Validation uses `.plan/.schema.json` (project-local), or falls back to `~/.config/plan/schema.json`.

## Configuration

Both tools use layered configuration: **defaults < config file < env vars < CLI flags**.

### plan

| Source | Path |
|---|---|
| Config file | `plan.toml` (project) / `~/.config/plan/config.toml` (user) |
| Env prefix | `PLAN_` |
| Env vars | `PLAN_ROOT`, `PLAN_EDITOR`, `PLAN_CONFIG`, `PLAN_THEMES_DIR` |

### gloss

| Source | Path |
|---|---|
| Config file | `gloss.toml` (project) / `~/.config/gloss/config.toml` (user) |
| Env prefix | `GLOSS_` |
| Env vars | `GLOSS_CONFIG`, `GLOSS_THEMES_DIR` |

### Themes

Theme TOML files in `themes/` directory. Cycle with `c` in preview mode.

```toml
name = "my-theme"

[theme]
h1_color = "green"
code_color = "magenta"
```

Color values: `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `gray`, `darkgray`, `white`, or ANSI256 decimal.

## Integration with AI agents

`plan` works naturally with AI coding agents. The `.plan/` directory convention gives agents a structured way to record discoveries:

1. Agent encounters something tangential during a task
2. Agent creates a task file in `.plan/`
3. Human reviews with `plan -k` or `plan tui`

## License

MIT OR Apache-2.0
