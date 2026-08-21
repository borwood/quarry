---
id: it-2eqk
type: item
title: 'the leased return silences the land-time vein prompt: a registered report is a doc inside the observed set'
v: 4
status: done
provenance: assistant
created: 2026-08-21T10:29:30Z
actor: claude-fable-5
kind: bug
acceptance:
- a dispatched landing whose agent registered its report still draws the landed-uncited prompt when no claim or doc cites the code it touched, and the arc report path alone never satisfies the check
witness:
- line: a dispatched landing whose agent registered its report still draws the landed-uncited prompt when no claim or doc cites the code it touched, and the arc report path alone never satisfies the check
  by: agent:ac5ef4f97bb5d7f51
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
- rel: depends-on
  to: it-3prx
  at: 5
---

Introduced by the it-3prx landing (cl-ue2e) and measured before it landed, not after: with the arc's report path leased, the agent's report is an in-repo write under the badge, so it accrues into the observed set like any other touch — correctly, by cl-up6s's no-silent-discard invariant. main.rs's vein_check then takes the observed set as its globs (observed wins over lease globs, deliberately: what the guard saw is truer than what the lease predicted), and queries::files_cited counts a DOC WHOSE OWN PATH falls inside those globs as a citation. The registered report is exactly such a doc. So every dispatched landing whose agent registered its report satisfies files_cited trivially, and the land-time landmark prompt of dc-grrb goes silent — on the one class of landing it was written for.

MEASURED, not reasoned: with observed = ["src/geo/pass.rs", "docs/reports/<arc>.md"] and no claim citing either, files_cited is false and the prompt fires; registering the report doc at that path — changing nothing else — flips it to true. (Throwaway instrument, run at the it-3prx build and deleted; two asserts, the second the failure.)

The fix is one line, at the consumer, and its predicate already exists: vein_check filters coord::is_arc_report out of the globs it checks before calling files_cited — the arc's return is accounting, never a landed capability, so it can neither cite nor be cited for one. render::harvest already makes the same exclusion for its leased-but-untouched enumeration. Both files sit outside the it-3prx write-set (src/main.rs, and src/queries.rs if the second call site at main.rs's release path is repaired the same way), which is why this is filed rather than fixed.

Worth deciding while here: files_cited's doc-path clause counts ANY registered doc inside the touched set as citation, which is loose by design (dc-grrb: presence of citation, never quality). The narrow fix restores the prompt for dispatched landings; whether the clause itself wants tightening is a separate question, not this item's.
