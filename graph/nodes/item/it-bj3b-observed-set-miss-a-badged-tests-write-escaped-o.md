---
id: it-bj3b
type: item
title: 'observed-set miss: a badged tests write escaped observation'
v: 4
status: sketch
provenance: assistant
created: 2026-08-14T06:42:48Z
actor: claude-fable-5
kind: bug
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Instance 2026-08-14, it-hapc harvest: OBSERVED vs LEASED reported tests/** leased-but-untouched while tests/basic.rs was genuinely modified under the badge - the new test exists, the tree diff against the prior commit confirms it, the suite counts it. The write happened; the observed set missed it. Sibling of it-pdnq (ordering-dependent observation, done) and it-z2sy (observed set ignores live leases); the agent also stopped and relaunched q serve mid-dispatch, which may or may not relate. Reclassified bug 2026-08-14 by user ruling in the followups session: a failure to observe a file change is a defect on its face - no second-instance threshold, no watch. Diagnose and fix with urgency; observation accounting must be trustworthy before the successor repo adopts the tool.
