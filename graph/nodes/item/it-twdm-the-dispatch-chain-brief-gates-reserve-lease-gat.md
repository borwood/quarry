---
id: it-twdm
type: item
title: 'the dispatch chain: brief gates reserve, lease gates write'
v: 5
status: done
provenance: assistant
created: 2026-08-09T07:54:21Z
actor: claude
kind: feature
ratified:
  by: user
  date: 2026-08-09
acceptance:
- q reserve refuses unless a brief-render event for the item exists in this session's log
- 'PreToolUse hook checks code writes against live lease coverage: warn for main sessions, deny for dispatched agents'
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: it-hrbq
  at: 4
- rel: part-of
  to: it-wkcq
  at: 2
---

The structural guarantee for the code-write channel, settled 2026-08-09: delivery and acknowledgment at choke points the work cannot avoid. q brief renders and logs; q reserve refuses without a same-session brief render for the item (you cannot hold a lease on work you were not briefed for); a PreToolUse hook checks file writes against live lease coverage — warn-first for user-orchestrated main sessions, deny for dispatched agents. Hardens the deliberately-soft lease compromise; brickolage parallelism is what changes that calculus. Enforcement beats doctrine, measured (the build mutex lesson).
