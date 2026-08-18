---
id: cl-6ctr
type: claim
title: '`q-join`: dispatch mints a single-use token into a one-line spawn prompt; q join consumes it, binds the acting identity to the badge, and renders the brief fresh'
v: 3
status: asserted
provenance: assistant
created: 2026-08-13T11:04:24Z
actor: claude-fable-5
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/ops.rs
  at: c4e91d7fe548
---

q dispatch mints a single-use join token into a one-line spawn prompt; q join consumes the token, binds the acting identity (agent, chat, or session key) to the badge in the association map, logs the join, and renders the brief fresh from the graph. Re-join by the same identity is idempotent; a second identity refuses; a re-dispatch mints a fresh token and kills the old one.
