---
id: it-vkxh
type: item
title: 'join-as-fetch: the token hand-off, agent identity injection, and the join gate'
v: 3
status: shaped
provenance: assistant
created: 2026-08-13T10:35:42Z
actor: claude-fable-5
kind: debt
acceptance:
- 'lands `q-join`: dispatch mints a single-use token into a one-line spawn prompt; q join consumes it, binds the acting agent identity to the badge, and renders the brief fresh'
- 'lands `agent-identity-injection`: the session hook injects QUARRY_AGENT from the hook-provided agent id alongside CHAT/SESSION/ACTOR; q resolves badges from the association map; per-shell export dies from the payload'
- 'lands `join-gate`: file writes into a dispatched lease globs from a context resolving no badge are denied with the teaching line - the C8 move applied to the hand-off'
- 'lands `work-only-stamping`: held entries resolve refusal and boundary only; stamping resolves env and association, never the holding chat'
write_set:
- src/**
- tests/**
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-zbxj
  at: 1
---

Sketched 2026-08-13 from dc-zbxj: replaces the paste-whole hand-off and the per-shell export discipline that dc-zbxj rules out. Landing this closes it-pdnq (ordering-dependent observation - the gate and the constructed association remove the learned-association window) and retires the manual-export instruction from the dispatch payload. Held entries keep refusal and boundary only; multi-badge holding and the double-dispatch refusal stay deliberately out of scope (deferred on th-6upm).
