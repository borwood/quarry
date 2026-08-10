---
id: th-fu73
type: thread
title: 'user presence: when should a fork block?'
v: 3
status: parked
provenance: assistant
created: 2026-08-10T06:53:10Z
actor: claude
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: depends-on
  to: dc-kpqg
  at: 2
---

Seeded by the user 2026-08-10, parked as future work — return when the capability push settles. The concern: dc-kpqg's file-and-continue rule (queue the thread, take the least-committal provisional path, keep building) is right when the user is away, but sometimes a fork SHOULD block — when the user is present, an interrupt is cheap and a provisional path may be waste. Presence varies within a day; the rule is currently presence-blind. The idea: user presence as a graph/session feature. Sketch-space to explore when taken up: presence as a session-level signal (declared by the user, or derived — recency of user-provenance events); dispatch semantics that read it (present: surface the fork now and pause the affected arc; away: file-and-continue); which forks qualify even when present (C3-class only, or any high-cost fork); how presence interacts with the single-thread reply convention and the queue. Relation to standing rulings: refines HOW dc-kpqg's thread-filing lands, never whether the thread is filed — the queue stays the record either way. Old-process contrast, worth keeping: in deepcraft this idea would be a roadmap flag maintained by hand; here the parked thread IS the flag, surfaced by the thread's own status when the queue drains.
