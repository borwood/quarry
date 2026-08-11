---
id: it-rzh7
type: item
title: 'best-effort locking: one orchestrator, not real concurrency'
v: 1
status: sketch
provenance: assistant
created: 2026-08-11T05:25:36Z
actor: claude
kind: watch
edges:
- rel: about
  to: ar-xa38
  at: 1
---

Lease, intent, heartbeat, and dispatch state files lock best-effort - adequate for one human orchestrating a few sessions on one machine, not for real concurrency. Migrated from CLAUDE.md known compromises 2026-08-11. Related: it-u8uf.
