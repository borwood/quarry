---
id: cl-z6gc
type: claim
title: '`agent-identity-injection`: the session hook injects QUARRY_AGENT from the hook-provided agent_id alongside CHAT/SESSION/ACTOR; badges resolve from the association map'
v: 2
status: asserted
provenance: assistant
created: 2026-08-13T11:04:31Z
actor: claude-fable-5
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/teach.rs
  at: 70f6bcb0f76f
---

The session hook reads the agent id the harness provides to hooks run in a subagent (top-level agent_id, verified live 2026-08-13; read defensively across key spellings) and injects QUARRY_AGENT alongside QUARRY_CHAT, QUARRY_SESSION, and QUARRY_ACTOR. q resolves badges from the association map by agent, chat, then session key; the per-shell badge export died from the dispatch payload and QUARRY_DISPATCH env survives only as the out-of-hook-coverage override.
