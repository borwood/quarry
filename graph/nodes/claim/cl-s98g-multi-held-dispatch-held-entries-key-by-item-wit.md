---
id: cl-s98g
type: claim
title: '`multi-held-dispatch`: held entries key by item with a holder field - a chat holds many, an item belongs to one chat, steal takes it whole'
v: 5
status: asserted
provenance: assistant
created: 2026-08-13T13:41:30Z
actor: claude-fable-5
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 8f43237f98b6
- rel: supersedes
  to: cl-amza
  at: 5
- rel: supports
  to: dc-qyr5
  at: 2
---

held dispatch state keys by item with the dispatching chat riding each entry as holder; a chat holds any number of live dispatches (the same-chat second-dispatch refusal is deleted); dispatching an item another chat holds live refuses naming the holding chat and session; same-chat re-dispatch stays free with a fresh token; q dispatch --steal with a required --reason takes the dispatch whole - held entry, lease, fresh token, old agent's associations cleared - loud and logged. Legacy single-slot and per-chat-keyed state files migrate in place.
