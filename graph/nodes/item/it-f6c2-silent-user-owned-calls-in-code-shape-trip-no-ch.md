---
id: it-f6c2
type: item
title: silent user-owned calls in code shape trip no choke point
v: 7
status: ready
provenance: assistant
created: 2026-08-11T05:25:07Z
actor: claude
kind: bug
acceptance:
- 'lands nothing new: the dispatch RETURN spec carries a user-owned-calls slot, harvest homework reconciles declared calls against threads filed under the badge, and the first-interception echo names thread-filing as the only landing'
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: depends-on
  to: dc-kpqg
  at: 4
- rel: about
  to: ar-c7f5
  at: 1
---

dc-kpqg residue, reclassified a bug under the dc-ygzz boundary: an agent that makes a user-owned call purely in code shape — writing the code that embodies the decision, attempting no graph write — is invisible to both choke points. No graph write, so the write hook never fires; harvest's mechanical compare sees files touched but cannot see a judgment buried in the diff. The whole guard is priming plus the dispatcher happening to notice while reading the diff — the doctrine-not-enforcement shape the measured enforcement-beats-doctrine claim says fails.

The settled fix (user-agreed 2026-08-19): the call itself is semantic — no hook can see it — so construction lands at the two ends we control, plus the channel that already fires:

1. RETURN spec grows a user-owned-calls slot: the brief's report template carries an explicit "user-owned calls encountered" section; the agent declares each or writes "none". Absence of the section is itself a harvest flag.
2. Harvest reconciles mechanically: harvest homework prints report-declared calls (N) against threads filed under this badge (M) and confronts the dispatcher with any gap, alongside the diff.
3. The first-interception contract echo names the rule: user-owned calls land as threads; code is not a landing.
