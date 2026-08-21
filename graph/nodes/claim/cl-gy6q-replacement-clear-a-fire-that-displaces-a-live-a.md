---
id: cl-gy6q
type: claim
title: '`replacement-clear`: a fire that displaces a live arc releases the replaced agent at the upsert — one clearing point, both arms, and a refused fire displaces nothing'
v: 8
status: ratified
provenance: assistant
created: 2026-08-21T14:38:03Z
actor: claude-opus-5
kind: vein
ratified:
  by: claude-opus-5
  date: 2026-08-21
aliases:
- eplacement-clear-a-fire-that-displaces-a-live-ar
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/ops.rs
  at: 2c2276685d23
- rel: source
  to: file:tests/basic.rs
  at: 75f4c2e53c44
- rel: supports
  to: cl-b2z2
  at: 3
- rel: supports
  to: cl-kggw
  at: 3
---

`replacement-clear`: a fire that displaces a live arc releases the replaced agent at the upsert — one clearing point, both arms, and a refused fire displaces nothing

THE HOLE, MEASURED. ops::dispatch has two arms that hand a LIVE arc to a new agent: the cross-chat steal and the same-chat re-dispatch (the documented recovery flow). The steal cleared the replaced agent's badge and said why on the clear — "so a stolen-from agent's later acts stop stamping into an arc it no longer works". The re-dispatch arm, with the same replacement semantics, cleared nothing: it minted a fresh token and reset `joined`, and left the acting map untouched. A re-dispatch OVERWRITES the item's held entry (dc-qyr5: entries key by item) rather than removing it, so the replaced agent's acting row still named a badge that is still held — and coord::live_acting_badge honors exactly that, so the row stayed a LIVE binding, never residue. Repro before the fix: dispatch X, join as agent A, re-dispatch X from the same chat, join as agent B; load_dispatches shows acting A→X and B→X both live.

STALE IN BOTH DIRECTIONS. A's stray acts kept stamping into X's trace, and under one-badge-per-identity (cl-kggw) A's identity stayed BLOCKED from joining any other work until someone harvested X — the join refusal reads the same live row. A replaced agent could therefore neither be re-dispatched elsewhere nor stop polluting the arc it had lost.

ONE CLEARING POINT, BOTH ARMS. The fix is not a second clear; it is one. Both arms now route through coord::clear_dispatch at the upsert, so they agree by construction instead of by discipline — they differ in who holds afterward and in nothing else. The FULL clear is deliberate: the pins and the badge-scoped attention belong to the replaced BINDING too. cl-b2z2 already stated that a re-dispatched item's next agent reads with its own eyes, and cl-aujk that the pin lives and dies with the badge it serves; both sentences were false for the same-chat arm until now. Whoever joins next re-plants both, and a same-agent recovery re-binds at its own re-join by construction (the pin-restore pattern).

DECIDED EARLY, COMMITTED LATE. Whether a fire replaces a live arc is decided where the held entry is read; the replacement COMMITS at the upsert, past the last refusal. The placement is load-bearing, not tidiness. Between the decision and the upsert sit the lease arms — a lease held by another session, a C7 overlap on a re-reserve, an empty write-set — and the steal's early clear meant a fire that refused there had already destroyed the victim's dispatch: the arc lost by a command that failed. The steal EVENT moved down with the clear for the same reason, so a refused steal now logs no take-over it never performed. This is the rule the --files shape floor keeps at the top of the same function (cl-74qt): a refused fire has no side effects.

Pinned by a_same_chat_re_dispatch_frees_the_replaced_agent_the_way_a_steal_does and a_refused_fire_leaves_the_live_dispatch_standing in tests/basic.rs. The first measures the replacement: the replaced agent stamping into no arc, no acting row surviving it, the badge's attention residue gone and re-delivered at the next join, the freed identity joining other work (pre-fix a "one agent, one badge" refusal), exactly one agent wearing the badge afterward, and the held entry surviving its own replacement with holder intact and joined reset. The second measures the placement: a steal whose re-reserve refuses on C7 leaves the held entry, its holder, its joined agent, that agent's binding and its live token all standing, and logs no steal event. Each half probed against the regression it guards — dropping the clear fails the new instrument AND dispatch_steal_takes_the_dispatch_whole; restoring the clear at the decision point fails the refusal instrument.
