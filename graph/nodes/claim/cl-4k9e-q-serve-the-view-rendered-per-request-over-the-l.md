---
id: cl-4k9e
type: claim
title: '`q-serve`: the view rendered per request over the live store'
v: 4
status: asserted
provenance: assistant
created: 2026-08-12T11:59:44Z
actor: claude-fable-5
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/view.rs
  at: 9f654b842a36
- rel: supports
  to: dc-f79h
  at: 1
---

each GET re-renders the whole view over the live store: std-only accept loop on 127.0.0.1:7171 (--port), no cache, no watcher, no new dependency; render errors answer 500; the baked write stays the offline courtesy and q view --open prefers the served URL when the loop answers a probe
