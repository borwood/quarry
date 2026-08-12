---
id: it-hapc
type: item
title: 'fire-time liveness: dispatch offers the hand-off when a dispatching session is live'
v: 3
status: sketch
provenance: assistant
created: 2026-08-12T21:11:34Z
actor: claude
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: it-u8uf
  at: 7
- rel: depends-on
  to: dc-ydvb
  at: 2
---

Sketched 2026-08-12 from dc-ydvb: at q dispatch from a non-dispatching session, derive whether a dispatching-kind session is live (registry + last_seen heartbeat, both existing) and shape the offer: live - prefer the hand-off, leave the item ready for the steward loop; none - offer the launcher or fire solo, both legitimate. A surface, never a gate. Depends on session kind being readable (charter convention today) and collides with the single-slot badge until it-u8uf lands.
