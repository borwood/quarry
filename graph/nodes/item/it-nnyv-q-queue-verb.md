---
id: it-nnyv
type: item
title: q queue verb
v: 6
status: ready
provenance: assistant
created: 2026-08-07T11:05:00Z
actor: claude-fable-5
kind: feature
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: part-of
  to: it-wkcq
  at: 1
---

Single-thread topic queue: push/order/pop thread nodes (DESIGN.md section 9, need-order list). NOT q query queue (the answerable-thread listing) — this verb mechanizes conversation pacing: the user works one thread at a time; the queue holds what is next and in what order. Directly serves the single-thread reply convention (CLAUDE.md, user ruling 2026-08-09). Dropped in error 2026-08-09 from its bodyless title; restored the same day on reading the design doc.
