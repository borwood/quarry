---
id: it-3wef
type: item
title: 'protocol layer: verb-riding instructions and declared fields'
v: 3
status: done
provenance: assistant
created: 2026-08-08T20:28:03Z
actor: claude-fable-5
kind: feature
acceptance:
- project-authored instructions ride trigger points (verb x type x kind), emitted programmatically
- unknown frontmatter fields preserved; protocol-declared fields settable and rendered
- 'per-rule delivery tier: inline vs gate (intent-token), design pending user reaction'
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-9hgd
  at: 1
---

The pack half of the engine/protocol split. Instructions keyed to trigger points live in the host graph and fire in verb output. Delivery tiers under discussion (2026-08-08): inline (rides the act, default, frequent verbs) vs gate (clobber-precedent intent-token: intercept, inject context, mint arg-bound one-time token, re-invoke executes) — with session-scoped once-per-rule firing as the proposed improvement so gates degrade to inline within a session. Gate candidates: content-shaping charters (journal), rare consequential verbs (steal). User's clobber project is the precedent.
