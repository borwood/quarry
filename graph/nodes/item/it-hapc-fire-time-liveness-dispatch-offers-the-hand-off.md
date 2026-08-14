---
id: it-hapc
type: item
title: 'fire-time liveness: dispatch offers the hand-off when a dispatching session is live'
v: 7
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
- rel: depends-on
  to: it-skpa
  at: 5
- rel: depends-on
  to: dc-crea
  at: 2
---

Sketched 2026-08-12 from dc-ydvb; reshaped 2026-08-13 under dc-ad8b (kind is registry data) and dc-crea (routing): at q dispatch from a non-dispatch session, derive the live sessions with appropriate coverage for the item (registry kind + purview fit + last_seen heartbeat; the kind field landed with it-skpa). One appropriate, charter-certain match - prefer the hand-off, leave the item ready for it, inform the user of the routing. Multiple plausible matches or any ambiguity - defer to the user; never guess between dispatchers. None live - offer the launcher (dispatcher-session.cmd, minted under dc-wngq) or fire solo, both legitimate. A surface, never a gate.
