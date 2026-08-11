---
id: it-z2sy
type: item
title: 'solo-lease writes accrue as leaseless: the observed set ignores live leases'
v: 2
status: sketch
provenance: assistant
created: 2026-08-11T06:13:01Z
actor: claude
kind: bug
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-cc76
  at: 3
---

Noticed at this session's wrap: the it-j2rk solo arc held a lease on src/** while editing src/main.rs, yet wrap's leaseless pickup listed main.rs - accrual keys per-item only under a dispatch badge, so a leased solo arc records session-keyed and reads as undeclared. Fix: the guard's accrual (and wrap's pickup) should key per-item when a live lease of the writing session covers the path - the lease IS the declaration (dc-cc76); the badge is only the delegated case. Cheap: one reservations read on the observed path.
