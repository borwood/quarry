---
id: cl-3qwu
type: claim
title: '`surface-atom`: one struct in one module — `atom-line`, `atom-ref`, `atom-unpack` own every register'
v: 3
status: asserted
provenance: assistant
created: 2026-08-12T10:35:34Z
actor: claude
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/surface.rs
  at: a7f6c9502a6c
---

One Atom struct resolved from the graph carries id, title, type, kind, status, v, areas as titles, provenance with grounding presence, and archived; atom_line, atom_ref, and atom_unpack are the only renderers, all owned by src/surface.rs.
