---
id: it-bj3b
type: item
title: 'observed-set miss: a badged tests write escaped observation'
v: 1
status: sketch
provenance: assistant
created: 2026-08-14T06:42:48Z
actor: claude-fable-5
kind: watch
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Instance 2026-08-14, it-hapc harvest: OBSERVED vs LEASED reported tests/** leased-but-untouched while tests/basic.rs was genuinely modified under the badge - the new test exists, the tree diff against the prior commit confirms it, the suite counts it. The write happened; the observed set missed it. Sibling of it-pdnq (ordering-dependent observation, done) and it-z2sy (observed set ignores live leases) - mechanism here unidentified; the agent also stopped and relaunched q serve mid-dispatch, which may or may not relate. Watch: on a second instance, promote to bug and hunt the mechanism.
