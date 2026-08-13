---
id: dc-qyr5
type: decision
title: 'multi-held dispatch: a chat holds many, an item belongs to one chat, the boundary harvests all'
v: 2
status: in-force
provenance: user
created: 2026-08-13T13:20:11Z
actor: claude-fable-5
ratified:
  by: user
  date: 2026-08-13
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: settles
  to: th-6upm
  at: 3
---

Multi-held: a chat may hold any number of live dispatches; the same-chat second-dispatch refusal is deleted - stamping follows the work (dc-zbxj), so holding resolves nothing and the ambiguity that justified the refusal is gone. An item belongs to one chat: dispatching an already-live-dispatched item refuses, naming the holding chat and session; re-dispatch from the same chat stays free (the documented recovery flow - fresh token minted, old token dies); from anywhere else, steal with a required reason takes the whole dispatch over, loud and logged - stealable, never shareable, the lease pattern applied to dispatches. Boundary: wrap, resume, and retire refuse while the chat holds ANY live dispatch, enumerating each with its q harvest command - no parking across a boundary; subagents cannot outlive their host session, so the long-runner exception is a non-case. Settles the remaining topics of the findings triage: multi-badge holding and holes 4 and 5.
