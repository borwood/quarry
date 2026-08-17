---
id: cl-jdg3
type: claim
title: '`render-once`: a node renders once at its highest earned fidelity; every other position is a one-line ref'
v: 3
status: asserted
provenance: assistant
created: 2026-08-17T08:31:27Z
actor: claude-fable-5
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/render.rs
  at: 5762e22f59c0
---

Render-once bookkeeping maps each node id to the section that rendered its body — READ-FIRST and THE MAP seed it, backdrop sections consume it: full > truncated > atom > count, and every later position renders the atom line with a body-in-section-above pointer. The dedup failure the payload spike measured (do-2g8m) closes by construction.
