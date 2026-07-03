---
status: idea
type: feature
priority: low
area: backend
---

## Fully agentic feedback loop

### Problem

The current AI agent integration is one-directional — agents create tasks in .plan/, humans review manually. There's
no mechanism for agents to receive feedback, update task status based on CI results, or autonomously iterate on work.

### Current state

```
Agent ──creates──► .plan/task.md  ──human reads──►  Human
                                               (no return path)
```

### Target state

```
Agent ──creates──► .plan/task.md  ──human edits──►  .plan/task.md
  ▲                                                   │
  └─────────agent re-reads, acts on feedback──────────┘
```

### Loop levels

| Level | Description                                   | Autonomy | Trust required |
| ----- | --------------------------------------------- | -------- | -------------- |
| 0     | Agent creates tasks, human reviews manually   | None     | None           |
| 1     | Agent reads task status, acts on changes      | Low      | Low            |
| 2     | Agent updates task status, creates follow-ups | Medium   | Medium         |
| 3     | Agent opens PRs, reads CI, fixes failures     | High     | High           |
| 4     | Agent deletes/creates tasks autonomously      | Full     | Full           |

### Level 1: Read feedback (minimal)

Agent reads .plan/ on each invocation. If a human changed status from `open` to `blocked` and added a note, the agent
sees it next time:

```markdown
---
status: blocked
---

## Notes

Blocked on API endpoint design. Need to decide REST vs GraphQL first.
```

No new code needed — the agent just re-reads the file. The loop is:

```
plan list  ──►  agent sees blocked task  ──►  agent addresses blocker
```

### Level 2: Write updates (medium)

Agent can update task status and append notes:

```bash
# Agent workflow
plan list -s open          # discover work
# ... do the work ...
# Edit .plan/task.md: status → done, append results
plan format .plan/task.md  # clean up
```

Needs: agent-friendly CLI commands. Already works — `plan add`, `plan open`, `plan format` exist.

### Level 3: CI integration (high)

Full loop: agent creates task → opens PR → CI runs → agent reads results → fixes failures → re-pushes.

```
Agent ──► plan add fix-auth
  │
  ├─► git checkout -b fix-auth
  ├─► make changes
  ├─► git push, open PR
  │
  ◄── CI fails: test_xyz broken
  │
  ├─► read CI output
  ├─► fix test
  ├─► git push
  │
  ◄── CI passes
  │
  └─► edit .plan/fix-auth.md: status → done
```

### Level 4: MCP / API integration

Expose plan operations as MCP tools so agents can call them programmatically:

```json
{
  "tool": "plan_list",
  "arguments": { "status": "open", "area": "backend" }
}
```

```json
{
  "tool": "plan_update",
  "arguments": { "slug": "fix-auth", "status": "done", "note": "Fixed in commit abc123" }
}
```

The plan CLI already has the right interface for this — each subcommand maps to an MCP tool.

### Trust and policy

| Operation      | Default policy | Configurable?                 |
| -------------- | -------------- | ----------------------------- |
| `plan list`    | Allow          | —                             |
| `plan add`     | Allow          | Yes (area restrictions)       |
| `plan open`    | Allow          | —                             |
| `plan format`  | Allow          | —                             |
| `plan delete`  | Deny           | Yes                           |
| Status changes | Allow          | Yes (require human approval?) |

### Exploration needed

- How do agents discover task changes? Options:

  | Method | Latency | Portability | Complexity |
  | --- | --- | --- | --- |
  | Poll on each invocation | High | High | Low |
  | inotify/fswatch | Low | Linux/macOS | Medium |
  | File hash comparison | Medium | High | Low |

- What's the minimal MCP server? A thin wrapper around the plan CLI binary. Could be 100 lines of Rust using the `rmcp`
  crate.

- Should agent-created tasks be marked differently? E.g., `area: agent` or a frontmatter field `created_by: agent` for
  auditability.
