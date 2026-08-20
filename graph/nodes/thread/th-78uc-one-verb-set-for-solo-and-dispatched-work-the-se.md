---
id: th-78uc
type: thread
title: 'one verb set for solo and dispatched work: the seat coincidence is the special case'
v: 1
status: queued
provenance: user
created: 2026-08-20T03:29:44Z
actor: claude
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
---

The user's direction (2026-08-19, during the defect sweep): solo and subagent work should share the same verb set. The two-seat machinery (dispatcher seat, agent seat) should allow the same identity to occupy both seats — loudly reporting that it joined its own dispatch — and both-seats-same-identity becomes the special case for otherwise-blocking guards (dispatcher writing into a leased zone, and any others). The instinct: separate paths where, from the agentic user's perspective, it could be one system.

The scouting finding that reframes the scope: self-join already works. ops::join refuses only a missing identity and the witness-author key (dc-mpg8) — nothing stops the dispatching chat from consuming its own token. Once bound, its writes resolve to the badge via the acting map, accrue per-item, draw the first-write contract echo, and pass the dispatched-zone deny (which keys on joined-vs-unjoined, not dispatcher-vs-agent). The model is latent in the construction; it is untaught, unreported, and its seat-collapse consequences are unhandled.

Change inventory (scoped 2026-08-19; ~5 files — main, ops, coord, teach, render — plus regen and tests):
1. One-act self-dispatch (q dispatch <item> --files --self = dispatch + immediate self-join, no token dance), loudly reporting both seats are yours; the same loud detection when a token path self-joins.
2. Guard carve-outs: near zero — verification and tests more than code.
3. Honesty marking in harvest and dispatch-trace: a self-occupied arc's observed set is hook-observed but self-reported in role — one identity held both seats, chores indistinguishable from arc writes. The output says so. Precedent: the assay office's solo self-ratified-on-the-record.
4. Boundary interactions: wrap/retire refuse while a self-dispatch flies (the refusal teaches self-harvest); a second self-join refuses while one is live.
5. The solo path's public face retires: brief+reserve become internals; C8 is satisfied by construction (dispatch logs the brief); guide/skill teaching collapses to one verb set.

Dissolves: it-z2sy whole (no more unbadged declared arcs); the two unfiled holes found in the same scouting (reserve permits same-session overlapping solo leases; the parent session can overlap-reserve its own dispatched zone unrefused — both die with reserve leaving the public path); it-csm3's self-arc case (acting identity IS the session; the real-dispatch case stands). Prerequisite: it-tanf (multi-join stamping) — self-join leans on the acting map's one-badge-per-identity being right.

Two accepted constraints from the pushback, both design requirements:
1. The seat split is where dispatch accounting gets its trustworthiness — observed-vs-leased means something because writer and judge differ. A self-occupied arc keeps mechanical observation but loses the adversarial reading; the record must never masquerade as seat-separated accounting.
2. Ceremony parity is a hard requirement: solo today is 4 acts (brief, reserve, done, release). If unification exceeds that (dispatch, join, harvest, done, release), the declared path gets heavier and agents drift toward leaseless drive-bys. One-act self-dispatch and prompt-never-gate self-harvest hold it at 4. If parity cannot hold, the design does not ship.
