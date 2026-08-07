---
id: dc-3kzj
type: decision
title: per-node monotone versioning; every ref stamps its target
v: 1
status: in-force
provenance: user
created: 2026-08-07T11:04:40Z
actor: claude-fable-5
ratified:
  by: user
  date: 2026-08-07
edges:
- rel: about
  to: ar-bu4u
  at: 1
---

Git backs whole-artifact history; nodes carry monotone v; edges record the target version (or file blob) they were written against. Staleness is a query, not a sweep.
