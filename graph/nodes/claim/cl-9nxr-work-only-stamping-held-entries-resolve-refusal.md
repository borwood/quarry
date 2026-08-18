---
id: cl-9nxr
type: claim
title: '`work-only-stamping`: held entries resolve refusal and boundary only; stamping resolves env and association, never the holding chat'
v: 4
status: asserted
provenance: assistant
created: 2026-08-13T11:04:46Z
actor: claude-fable-5
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 5f2e8b4afc49
- rel: supersedes
  to: cl-vsew
  at: 6
---

Held dispatch entries resolve refusal and boundary only (boundary_badge reads env, associations, then held); badge resolution for stamping and the write guard (badge_for) reads env and the association map by agent, chat, then session key — never the holding chat or session. The dispatching chat mid-flight acts stamp nothing; no-attribution beats mis-attribution.
