---
id: it-ds6b
type: item
title: 'acceptance lines are append-only: no replace or remove verb'
v: 2
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

Requirement from the th-wxr9 settlement (user-ruled 2026-08-17): when replace/remove lands, the change verbs must by construction loudly unflip a readied item that the edit leaves acceptance-less — ready back to shaped, stated in the verb's own output. The ready-implies-acceptance invariant is then held at every mutation point: flip (q ready) refuses, fire (q reserve) backstops, the brief trips on breach, and these verbs demote on strip. The fix inherits this as a requirement, not a suggestion.
