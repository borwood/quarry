---
id: dc-qeup
type: decision
title: sessions are purviews; leases are per-item, at dispatch, explicit-release
v: 2
status: in-force
provenance: user
created: 2026-08-08T13:46:31Z
actor: claude-fable-5
ratified:
  by: user
  date: 2026-08-08
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: settles
  to: th-j9uu
  at: 2
---

Sessions are purviews; leases are per-item plumbing. A session's durable identity is a named purview over areas (committed registry, QUARRY_SESSION attribution); sessions start leaseless and reserve AT DISPATCH, when an agent is about to touch files — the lease attaches to the item it serves, several may be held concurrently, and release is explicit only (wrap nags; steals are loud and logged). Globs are the truth of a write-set (** covers files the work will create); exclusive is the default and --shared marks co-write zones where presence-awareness replaces mutual exclusion. Cross-session requests need no new machinery: an item filed into the other purview with a depends-on from the blocked item, surfaced by purview-scoped orientation, closed by ordinary homework.
