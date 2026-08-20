---
id: cl-t7nx
type: claim
title: '`reserve-backstop`: fire hard-refuses an acceptance-less item on both stations and loudly un-readies it to shaped'
v: 3
status: ratified
provenance: assistant
created: 2026-08-18T01:24:30Z
actor: claude-fable-5
kind: vein
ratified:
  by: claude-fable-5
  date: 2026-08-18
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/ops.rs
  at: 289ccba3cb9e
- rel: supports
  to: cl-88ma
  at: 2
---

ops::acceptance_backstop is the one fire-time check: q dispatch calls it ahead of ownership, brief-logging, and lease logic (covering the re-dispatch path a reserve-side check would miss), and the q reserve handler calls it ahead of C8 on the solo path. A ready or in-flight item demotes to shaped through ops::set with the gate named in the logged note, so the ready feed stays true; nothing else mutates on the refusal — no brief event, no lease, no in-flight flip. Teaching is station-shaped: the dispatcher is taught return-to-design and never the authoring command (the pen stays with design, dc-p6z4); the solo station, design-capable, is taught acceptance+= directly. No inline authoring flags exist on either station. The it-ds6b rider inherits the invariant at mutation time: an edit stripping the last acceptance line demotes the same way.
