---
id: it-xpt7
type: item
title: 'multi-held dispatch: many badges per chat, per-item ownership, boundary harvests all'
v: 7
status: done
provenance: assistant
created: 2026-08-13T13:20:20Z
actor: claude-fable-5
kind: debt
acceptance:
- 'lands `multi-held`: a chat holds any number of live dispatches; the same-chat second-dispatch refusal is deleted'
- 'lands `item-one-chat`: dispatching a live-dispatched item refuses naming the holding chat and session; same-chat re-dispatch stays free with a fresh token; steal with a required reason takes the dispatch over, loud and logged'
- 'lands `boundary-harvests-all`: wrap, resume, and retire refuse while the chat holds any live dispatch, enumerating each with its q harvest command'
write_set:
- src/**
- tests/**
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-qyr5
  at: 2
---

Sketched 2026-08-13 from dc-qyr5, the last piece of the th-6upm triage: completes the parallel-dispatch repair arc that it-u8uf opened and it-vkxh advanced. Deletes the same-chat second-dispatch refusal; replaces it with per-item ownership (refuse naming the holder, same-chat re-dispatch free with a fresh token, steal with a required reason takes the dispatch whole - the lease steal pattern, loud and logged); the boundary guard enumerates every live held dispatch with its q harvest command. Landing this makes fire-them-all-off literal from one chat. Seams for the builder: coord.rs holds DispatchMap.held (keyed per dispatching chat - multi-held means several entries under one chat key wants a keying rethink: key by item with a holder field, or value becomes a list), boundary_badge and boundary_refusal (enumerate, not first-found), clear_dispatch (per item, must stay exact under multi-held); ops.rs dispatch holds the same-chat refusal to delete, the cross-chat double-hold to refuse, the steal path to add, and the token mint; teach.rs lease_check reads dispatched state (verify it stays per-item correct); tests/basic.rs dispatch_one_act_then_harvest, boundary tests, and the legacy migration test all encode the old semantics and need the flip, commented at each flip site. Never edit graph/ by hand; the surfacing atom owns all title rendering.
