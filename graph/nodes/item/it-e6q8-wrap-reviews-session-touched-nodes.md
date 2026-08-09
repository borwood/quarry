---
id: it-e6q8
type: item
title: wrap reviews session-touched nodes
v: 1
status: sketch
provenance: user
created: 2026-08-09T07:14:53Z
actor: claude
kind: feature
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

At wrap, list every node created or adjusted during the session for a final review pass. Derived from the event log — bounded by the session, no corpus sweep, no matching heuristics. Catches same-session stragglers (a ruling minted mid-session gets a second look while its obsolescence homework is still cheap); cross-session obsolescence remains rule-time judgment. Replaces the rejected rule-time text-match tripwire: naive text match WILL mislink, and a full body+title sweep grows expensive under many parallel lines of work (brickolage will have them). Noted for the future, not load-bearing: a meme of quoting concept names in backticks would strengthen phrase matching if lexical tooling is ever wanted — but per the autopsy, mechanisms must not depend on remembered conventions.
