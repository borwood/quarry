---
id: it-hapc
type: item
title: 'fire-time liveness: dispatch offers the hand-off when a dispatching session is live'
v: 12
status: done
provenance: assistant
created: 2026-08-12T21:11:34Z
actor: claude
acceptance:
- '`fire-routing`: q dispatch from a non-dispatch session derives dispatch-kind coverage (kind + purview fit + last_seen heartbeat) and stops with the leave offer when a covering dispatcher is live - nothing dispatched, nothing written, the live session(s) named'
- 'the wake offer: none awake enumerates the registered dispatch-kind sessions covering the item, each with its launcher (script when present, inline command otherwise); one candidate is offered directly, several defer to the user'
- --solo fires from anywhere, no reason demanded; dispatch-kind sessions, continuations of a live dispatch (re-dispatch, steal), and items no dispatcher covers never route
- 'routing is advisory and stateless: no routed-waiting state, exit clean, the item stays honestly ready for anyone'
archived: true
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
  at: 4
---

Reshaped 2026-08-14 under dc-crea (pull semantics): at q dispatch from a non-dispatch session, derive the live sessions with appropriate coverage for the item (kind field from it-skpa + purview fit + last_seen heartbeat). Any live - offer the leave: do not fire solo, keep the item ready (ready IS the dispatcher feed), print which live session(s) cover it; no choosing among them - the first to claim dispatches it, and the claim point guards the race (dc-qyr5). None awake - offer the wake choice: enumerate registered dispatch-kind sessions covering the item; one charter-certain candidate - offer its launcher (dispatcher-session.cmd today); several plausible - the user picks. Fire solo stays legitimate throughout. A surface, never a gate.
