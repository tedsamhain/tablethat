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

## License

MIT OR Apache-2.0
