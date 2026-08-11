---
id: th-ejn6
type: thread
title: 'leaseless code writes: where does the lease boundary actually sit?'
v: 4
status: resolved
provenance: user
created: 2026-08-09T16:57:21Z
actor: claude
archived: true
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: depends-on
  to: dc-qeup
  at: 2
---

Raised by the user 2026-08-09 during the landmark ratification: dispatching work that touches files CAN happen without a lease today, and that deserves careful reflection. The facts: dispatch has no verb — it happens in the harness, invisible to q; QUARRY_DISPATCH is an opt-in marker; with no leases on file the write hook allows everything; and unbound chats CANNOT lease at all (no session identity) — the very session that built the chain wrote code all night leaseless. The chain binds only once a lease exists; nothing gates dispatch on reserving. Candidate lines to weigh: a write-hook warning on any code write when the repo has registered sessions but no covering lease (noise risk for casual main-session edits); a q dispatch verb that renders the brief, reserves, and sets in-flight in one act — making the chain unavoidable by making it the convenient path; provisional leases for unbound chats. USER LEAN (2026-08-09): follow the dispatch-verb trail. Convenience should be leveraged, and it only can be when agents automatically KNOW it is the convenient option — the verb must be advertised where dispatch decisions happen (ready listings, briefs, the guide). Sibling thread: the dispatched agent as quarry citizen.
