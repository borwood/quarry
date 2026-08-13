---
id: it-ds6b
type: item
title: 'acceptance lines are append-only: no replace or remove verb'
v: 1
status: sketch
provenance: assistant
created: 2026-08-11T10:29:40Z
actor: claude
kind: bug
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/ops.rs
  at: 01323571de4e
---

q set supports acceptance+= only; a corrupted or stale acceptance line cannot be repaired or retired through any verb. Discovered 2026-08-11 when shell-mangled backtick names forced re-minting it-u7dp as it-ygw7. Wants acceptance removal or replacement as a logged act, in the spirit of q unlink.
