---
id: it-e6wq
type: item
title: session-retire overbreadth under an unrelated badge
v: 6
status: ready
provenance: assistant
created: 2026-08-11T05:25:26Z
actor: claude
kind: bug
acceptance:
- 'lands nothing new: q session retire refuses only when the retiree is implicated - the chat''s own session under a live badge, or a retiree with a dispatch in flight - and a third session with no live dispatch retires clean while unrelated badges fly'
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: ar-c7f5
  at: 1
---

do-jn4s doubt, reclassified a bug under the dc-ygzz boundary: boundary_refusal blocks q session retire even when the active badge belongs to an unrelated arc and the retiree is a third session. With one badge on the machine, "any live dispatch" and "mid-dispatch" are the same fact and the refusal is right; once sessions and dispatches multiply it is overbroad — retiring an ephemeral third session touches nothing about a flying dispatch, yet every retire is treated as the chat closing its own arc.

The settled fix (user-agreed 2026-08-19) — scope the refusal to the retiree:

1. Retiring the chat's own session while it holds a live dispatch: refuse, as today — that IS closing out with harvest owed.
2. Retiring a session that itself has a dispatch in flight: refuse and point at that dispatch's q harvest — retire would rip the lease out from under a working agent.
3. Retiring a third session with no live dispatch: proceeds while unrelated badges fly; its idle leases release with last rites (retire's existing semantics).
