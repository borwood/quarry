---
id: it-3prx
type: item
title: 'the brief demands a report the lease forbids: docs/reports sits outside every dispatch write-set'
v: 3
status: sketch
provenance: assistant
created: 2026-08-21T09:20:53Z
actor: claude-fable-5
kind: bug
acceptance:
- 'a dispatched agent can register its own report from its own seat: the lease a dispatch takes covers the report path the RETURN spec demands, and harvest reconciles the declared user-owned calls without a dispatcher writing the doc node by hand'
witness:
- line: 'a dispatched agent can register its own report from its own seat: the lease a dispatch takes covers the report path the RETURN spec demands, and harvest reconciles the declared user-owned calls without a dispatcher writing the doc node by hand'
  by: chat:b8214cb4-bf6e-42fa-9ceb-317d5660ca2d
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
---

Witnessed from the dispatcher seat across three arcs on 2026-08-21 (it-rmqy twice, it-ap3x once): every one returned a report it could not register, and the dispatcher wrote the doc node by hand at landing.

The contradiction is in construction, not discipline. render::brief's RETURN spec instructs the agent to register its report, and harvest's HOMEWORK line prints the exact q new doc command with --path <report.md>. But the lease is derived from --files or the item's recorded write-set, and neither names docs/reports/ — an item's write-set describes the code the work touches, which is the whole point of it. So the write guard denies the one artifact the brief mandates. The agent seat cannot repair it either: extending a lease is release plus re-reserve, and the actor rules forbid an agent releasing its lease.

The cost is not just the manual step. latest_report_doc is how the harvest seat joins to the report, and declared_user_owned_calls parses the user-owned section out of the registered file. An unregistered report means harvest renders the await arm and the agent's declaration — 'none', in all three arcs — lives only in the returned message, where nothing can parse it. The accounting built at it-f6c2 silently degrades to the dispatcher's good faith on every dispatch, which is precisely the discipline-not-construction shape this system rejects elsewhere (dc-zbxj).

Two fixes were named from the agent seat. Either the dispatch station adds the report path to every lease it takes (cheap, one line where the globs are assembled, and it needs nothing from the item), or the reconciliation grows a second source that can read a report the agent never registered. The first is the least-surprise reading: the report is part of the arc's contract, so the arc should be able to write it. Note that a shared lease is wrong here — two arcs writing distinct report files need no co-write zone, only their own paths.

Neighbourhood: it-x4bb, the other place the firing station hands out a lease that cannot serve the work it was taken for.
