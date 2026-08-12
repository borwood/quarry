---
id: cl-amza
type: claim
title: '`per-chat-badge`: dispatch state holds one entry per dispatching chat (…'
v: 3
status: asserted
provenance: assistant
created: 2026-08-12T21:34:37Z
actor: claude
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 73738d6f07cd
- rel: supports
  to: it-u8uf
  at: 7
---

`per-chat-badge`: dispatch state holds one entry per dispatching chat (chat:<id> via hook-injected QUARRY_CHAT, session:<name> where no chat id reaches), legacy single-slot files migrating under their session key; a second dispatch refuses only from the chat already holding one
