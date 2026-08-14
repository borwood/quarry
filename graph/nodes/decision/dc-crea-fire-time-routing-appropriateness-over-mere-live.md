---
id: dc-crea
type: decision
title: 'fire-time routing: a pull, never a send; the only choice is which dispatcher to wake'
v: 4
status: in-force
provenance: user
created: 2026-08-14T00:53:18Z
actor: claude-fable-5
ratified:
  by: user
  date: 2026-08-14
aliases:
- fire-time-routing-appropriateness-over-mere-live
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: ar-c7f5
  at: 1
- rel: builds-on
  to: dc-ydvb
  at: 2
---

Refines the dc-ydvb hand-off; tightened 2026-08-14 in discussion, replacing this decision's earlier "send" phrasing: routing is a PULL, never a send. At q dispatch from a non-dispatch session, derive the live sessions with appropriate coverage for the item (kind + purview fit + last_seen). Any live and appropriate - do not fire solo: leave the item in ready (ready IS the dispatcher feed; the kind-shaped wake and the purview-change surfaces deliver it) and inform the user which live session(s) cover it. There is never a choice between live dispatchers: the first to claim dispatches it, and the claim point already guards the race (dc-qyr5: an item belongs to one chat once dispatched; a second chat needs --steal --reason). Routing is advisory, never binding - no routed-waiting state; an unclaimed item stays honestly ready for anyone. The ONLY who-choice arises when no appropriate dispatcher is awake and design must choose which to WAKE: pick by charter certainty and inform the user; any ambiguity defers to the user. Fire solo stays legitimate throughout. Remains a surface, never a gate.
