---
id: cl-9v7c
type: claim
title: '`fire-routing`: dispatch offers the pull — the leave when a covering dispatcher is live, the wake when none; --solo fires regardless'
v: 3
status: asserted
provenance: assistant
created: 2026-08-14T06:32:40Z
actor: claude-fable-5
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 24df8ce05680
---

q dispatch from a non-dispatch session derives the dispatch-kind sessions covering the item (kind + purview fit against the item about-areas, vacuous for area-less items + the last_seen heartbeat under coord::DISPATCH_LIVE_SECS) in coord::fire_routing, the one derivation point; any live covering session stops the fire with the leave offer naming every live session unranked, none awake stops it with the wake offer enumerating candidates each beside coord::wake_command (launcher script when present, inline launch otherwise); the offer is stateless (no held entry, no status touch, exit 0) and --solo, a dispatch-kind session, a continuation of a live dispatch, or an uncovered item fires without a surface
