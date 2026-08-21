---
id: cl-8h8j
type: claim
title: '`landmark-globs`: the arc''s own return drops out of the land-time citation check — one filter point, both consumers'
v: 5
status: ratified
provenance: assistant
created: 2026-08-21T10:43:52Z
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
  to: file:src/queries.rs
  at: b4bf019f82ca
- rel: supports
  to: it-2eqk
  at: 4
- rel: source
  to: file:src/main.rs
  at: 1f5713b3303b
---

`landmark-globs`: the arc's own return drops out of the land-time citation check — one filter point, both consumers

The consumer-side correction under the leased return (it-2eqk, cl-ue2e). With the arc's report path leased, the agent's report is an in-repo write under the badge, so it accrues into the observed set like any other touch — correctly, by cl-up6s's no-silent-discard invariant. main.rs's vein_check takes the observed set as its globs (observed wins over lease globs, deliberately: what the guard saw is truer than what the lease predicted), and queries::files_cited counts a DOC WHOSE OWN PATH falls inside those globs as a citation. The registered report is exactly such a doc. So every dispatched landing whose agent registered its return satisfied files_cited trivially, and the land-time landmark prompt of dc-grrb went silent — on the one class of landing it was written for.

MEASURED, not reasoned. With observed = ["src/geo/pass.rs", "<the arc's report path>"] and no claim citing either, files_cited is false; registering the report doc at that path — no claim, no file edge, nothing said about the code — flips it to true. That assert stands in the instrument as the pre-fix half.

ONE FILTER POINT: queries::landmark_globs drops every glob coord::is_arc_report matches, and both consumers route through it — vein_check (the release and done-flip seats) and the wrap boundary backstop. The exclusion lands at the CONSUMER, never inside files_cited: the predicate answers literally about whatever it is handed, and a doc sitting at a checked path really is a citation of it. The arc's return is accounting, never a landed capability, so it can neither cite one nor be cited for one. render::harvest already made the same exclusion for its leased-but-untouched enumeration; this is the second reader of the same fact.

FILTERED BEFORE THE PROMPT, not only before the check: the line names the files it wants a vein for, and that list must carry the code that landed, never the paperwork about it.

THE SECOND CONSUMER IS REACHABLE, not defensive. q harvest clears the BADGE and leaves the lease (coord::clear_dispatch prunes held entries, acting rows, pins and badge attention — never globs), so a dispatcher who lands without releasing reaches the boundary with the arc's report path still in the held globs, and there the backstop reads the LEASE, not the observed set. Measured in the instrument end to end.

A dispatcher's own broader docs/reports/** glob is deliberately not this shape (coord::is_arc_report: concrete path, no wildcards) and survives the filter: it was authored as write-set, so it is the work, and a doc landing under it is a real citation.

Left standing, and not this item's: files_cited's doc-path clause counts ANY registered doc inside the touched set as citation. That looseness is by design (dc-grrb: presence of citation, never quality); whether the clause itself wants tightening is a separate question.

Pinned by the_arcs_own_return_never_satisfies_the_land_time_landmark_check in tests/basic.rs: the defect measured on files_cited itself (registration alone flipping it), the filtered set, the report path alone judging to empty, the dispatcher's own glob surviving, and both consumers end to end through the spawned binary — the done-flip prompt naming the code and not the return, and the wrap backstop the same after a harvest that left the lease. Each consumer probed against the regression it guards: removing the vein_check call silences the landing prompt entirely, removing the wrap call silences the backstop.
