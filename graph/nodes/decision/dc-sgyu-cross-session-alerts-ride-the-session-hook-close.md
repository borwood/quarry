---
id: dc-sgyu
type: decision
title: cross-session alerts ride the session hook, closed list, silence default
v: 2
status: in-force
provenance: user
created: 2026-08-08T22:00:27Z
actor: claude-fable-5
ratified:
  by: user
  date: 2026-08-08
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: settles
  to: th-v9tg
  at: 2
---

The session hook is the message bus: the same PreToolUse hook that injects identity emits throttled additionalContext deltas — no new hook, no polling process, delivery lands at the next tool call. The alert list is CLOSED, each naming an act: (a) something filed into your purview by another session, (b) your lease stolen, with the logged reason, (c) your work unblocked by another session's landing. Per-session cursor, 180s minimum check interval, first check plants the cursor without dumping history, silence is the default state. Citation staleness stays pull-only: write-time presence notes cover collisions; review-paced work does not interrupt.
