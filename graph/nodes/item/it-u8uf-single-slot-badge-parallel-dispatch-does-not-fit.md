---
id: it-u8uf
type: item
title: 'single-slot badge: parallel dispatch does not fit the machine-local state'
v: 7
status: done
provenance: assistant
created: 2026-08-11T05:25:18Z
actor: claude
kind: debt
acceptance:
- 'lands `per-chat-badge`: dispatch state holds one entry per dispatching chat, keyed by the session-hook binding; a second dispatch refuses only from the chat that already holds one'
- 'lands `badge-chat-resolution`: the write guard, event stamping, trace, and harvest resolve the badge for the acting chat, not the machine - parallel agents land under their own items'
write_set:
- src/**
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-ydvb
  at: 2
---

One dispatch state at a time (do-xt6f): q dispatch refuses while any dispatch is active on this machine, serializing hand-offs across every chat. Promoted from watch to debt 2026-08-12 by dc-ydvb: this is a regression, not a future concern — the user reports the single slot was introduced and first noticed in a session that had been running multiple parallel agents until that point, because parallel is the normal shape ("are these parallelizable items? fire them all off"). The ruling makes per-chat badge state a precondition of the settled dispatch pattern: a decisions session firing hot while a dispatching session stewards in-flight work collides at this slot today. Likely shape, unchanged from the watch: dispatch state keyed by the chat binding the session hook already knows; refusal only guards a second dispatch from the same chat; the write guard, event stamping, trace, and harvest resolve the badge for the acting chat, not the machine. Adjacent: it-9p3v (the trace is machine-scoped — the same key fixes both).
