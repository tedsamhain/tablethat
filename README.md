# tablethat

Markdown-native task tracker with a kanban TUI.

**Table that.** You're deep in task X. The AI agent finds something — a memory leak, a design smell, an interesting pattern. It's worth tracking, but not right now. `tablethat` captures it as a deferred task and gets out of your way.

```
tablethat "found auth token leak in retry logic" --type bug
```

Tasks live in `.plan/tasks/*.md` as markdown files with YAML frontmatter. The filesystem is the database, git is the audit trail.

## Install

```
cargo install --path .
```

## Quick start

```bash
# List all tasks
tablethat

# Kanban view grouped by status
tablethat -k

# Filter
tablethat -s open -t bug
tablethat -q "auth"

# Interactive TUI
tablethat tui

# Validate task frontmatter
tablethat --lint

# Normalize task files (canonical field ordering)
tablethat --format
```

## Task format

Each task is a markdown file in `.plan/tasks/`:

```markdown
---
status: open
type: bug
priority: high
area: backend
---

## Notes

Description and context. Agents append progress notes below.
```

**Status:** `idea` · `backlog` · `open` · `in-progress` · `blocked` · `done`
**Type:** `bug` · `feature` · `chore` · `decision` · `perf`
**Priority:** `high` · `medium` · `low`

Validation is driven by `.plan/schema.json` — edit it to add fields or constrain values.

## TUI

```
tablethat tui
```

Interactive kanban browser with keyboard navigation:

| Key              | Action                    |
| ---------------- | ------------------------- |
| `↑`/`↓` `k`/`j` | Navigate tasks            |
| `←`/`→` `h`/`l` | Move between fields       |
| `Enter`          | Preview task (markdown)   |
| `f`              | Filter by selected field  |
| `e`              | Open task in `$EDITOR`    |
| `q`              | Clear filters / quit      |
| `Ctrl-c`         | Quit                      |

## Integration with AI agents

`tablethat` works naturally with AI coding agents. The `.plan/` directory convention gives agents a structured way to record discoveries:

1. Agent encounters something tangential during a task
2. Agent runs `tablethat "summary" --status idea` to record it
3. Human reviews with `tablethat -k` or `tablethat tui`

The schema is human-readable markdown. No APIs, no databases, no coordination servers.

## Configuration

`tablethat` uses layered configuration: **defaults < config file < env vars < CLI flags**.

### Config file search paths

Checked in order (first found wins):

1. `--config <path>` (explicit CLI flag)
2. `$T2_CONFIG` env var
3. `./tablethat.toml` (project-local)
4. `~/.config/tablethat/config.toml` (Linux) / platform equivalent

### Environment variables

Prefix: `T2_`

| Variable | Equivalent |
|---|---|
| `T2_ROOT` | `--root` |
| `T2_EDITOR` | Editor fallback (overrides `$EDITOR`) |
| `T2_CONFIG` | Config file path |

### Config keys

All keys are optional. Example `tablethat.toml`:

```toml
root = "/path/to/project"
editor = "hx"
default_sort = ["priority", "area", "slug"]
kanban_order = ["idea", "backlog", "open", "in-progress", "blocked", "done"]

[theme]
h1_color = "green"
h2_color = "cyan"
h3_color = "cyan"
code_color = "magenta"
bold_style = "bold"        # bold, dim, italic, underlined
emphasis_style = "underlined"

[colors.status]
in_progress = "magenta"
open = "yellow"
blocked = "red"
backlog = "blue"
idea = "cyan"
done = "green"

[colors.priority]
high = "red"
medium = "yellow"
low = "darkgray"
```

Color values: `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `gray`, `darkgray`, `white`, or an ANSI256 decimal (e.g. `"8"`).

## License

MIT OR Apache-2.0
