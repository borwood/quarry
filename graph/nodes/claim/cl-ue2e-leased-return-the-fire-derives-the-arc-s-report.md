---
id: cl-ue2e
type: claim
title: '`leased-return`: the fire derives the arc''s report path and leases it — one path per arc, never the zone; the agent registers its own return'
v: 4
status: ratified
provenance: assistant
created: 2026-08-21T10:28:30Z
actor: claude-fable-5
kind: vein
ratified:
  by: claude-fable-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 1cb9b241aa5a
- rel: supports
  to: it-3prx
  at: 5
---

`leased-return`: the fire derives the arc's report path and leases it — one path per arc, never the zone; the agent registers its own return

The contract's own artifact is now inside the contract (it-3prx). ops::dispatch derives coord::arc_report_path once per fire and set_arc_report points the lease at it, so the report the RETURN spec demands is a write the guard admits. Before this, a lease came from --files or the item's recorded write-set — both of which name the CODE the work touches — and teach::lease_check's badge arm denied the one artifact the brief mandated, mid-arc, in a seat that cannot extend a lease. The whole user-owned accounting of cl-cjbb degraded with it: unregistered, the agent's declaration lived only in a returned message, harvest rendered the await arm, and the reconcile fell back to the dispatcher's good faith on every dispatch.

ONE PATH PER ARC, NEVER THE ZONE. The path is docs/reports/<date>-<slug>-<item>.md, the reports register's own shape, the item id carrying the uniqueness: coord::globs_overlap compares static prefixes, so two arcs' concrete paths are never prefixes of one another and concurrent dispatches never collide. A docs/reports/** glob would have done the opposite — made the reports directory a co-write zone nobody asked for, with each agent holding write access to its neighbours' returns, and (from a second session) refused the fire outright at C7. A re-dispatch takes the next free -arcN name and the lease re-points at it (coord::set_lease_globs, the firing station's own hand — an agent may not extend its lease), so arc two can never overwrite the file arc one registered.

TWO DELIBERATE EXEMPTIONS FROM CONTENTION. The report path joins AFTER coord::reserve rather than inside it, and coord::lease_overlaps skips arc report paths on both sides: counted, every write-set covering docs/** would refuse against every live arc's return, and docs-touching work would be unfireable while any dispatch flies. There is nothing to contend for — the path is unique to the arc and names a file that does not exist yet — and the write guard never offered that protection anyway: a badged write is judged against the BADGE's own lease, foreign exclusivity unconsulted.

READ BACK, NEVER RE-DERIVED. coord::arc_report_in picks the concrete path out of the lease (a pattern never stands in for a path), and both seats read it: render::brief's RETURN spec names the path and hands over the filled-in q new doc registration command, and render::harvest prints the exact --path — or, when queries::latest_report_doc already finds this arc's report, says it is registered from the agent's own seat and asks for nothing, so one report never mints two doc nodes. The arc's report path also drops out of harvest's leased-but-untouched enumeration: an unwritten report is a missing return, not dead weight in the lease.

Pinned by the_arcs_report_path_is_leased_at_the_fire_and_registers_from_the_agents_seat in tests/basic.rs: the pre-fix denial measured on lease_check itself, the fire, the guard admitting exactly the leased path and refusing a neighbour's, two arcs' returns not overlapping, a docs/** write-set still firing beside a live arc, the brief's command, the homework's exact path, the agent-registered end state reconciling instead of awaiting, the re-dispatch re-point, and the pure moves. Each half probed against the regression it guards: dropping the lease re-point, restoring the unfiltered contention test, restoring the always-ask homework, dropping the brief line, and dropping the untouched filter each fail the suite.
