## Task Management (.plan)

Every task lives as a markdown file in `.plan/<slug>.md` with YAML frontmatter. The filename (without `.md`) is the unique key — no numeric ID needed.

### Status lifecycle

| status        | meaning                   | when to use                                   |
| ------------- | ------------------------- | --------------------------------------------- |
| `idea`        | aspirational, not settled | explore later, not ready to start             |
| `backlog`     | accepted, deferred        | consider when stepping back to plan next work |
| `open`        | ready                     | actionable, waiting to be picked up           |
| `in-progress` | active                    | currently being worked on                     |
| `blocked`     | stuck                     | note the blocker in the body                  |
| `done`        | complete                  | finished                                      |

### Task Management Workflow

- **Discover work:** Use `plan list` lists all tasks and prefer items with priority => high and status => in-progress or open
- **Step back / plan:** Consider items with status => `backlog` or `idea` when considering next big steps
- **Create:** copy `.plan/.TEMPLATE.md` to `.plan/<slug>.md` and consider contained instructions to build the task.
- **Update:** update 'status' to reflect status. append progress notes at the bottom (newest first). Do not rewrite history.
- **Complete:** set `status: done` when finished. Do not delete the file.
- **Block:** set `status: blocked` and note the blocker in the body.
- **Validate:** run `plan lint` to check; `plan format` to auto-format.
