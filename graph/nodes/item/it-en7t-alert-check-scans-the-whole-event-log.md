---
id: it-en7t
type: item
title: alert check scans the whole event log
v: 1
status: sketch
provenance: assistant
created: 2026-08-11T05:25:36Z
actor: claude
kind: watch
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

The cross-session alert check reads the full log each pass - fine below ~10k events; shard-aware scanning is the fix the day it shows in hook latency. Migrated from CLAUDE.md known compromises 2026-08-11.
