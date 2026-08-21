---
id: cl-hct3
type: claim
title: '`arc-bounded-return`: the harvest reconcile joins only reports registered after this dispatch; a carried prior report is evidence, never a return'
v: 5
status: ratified
provenance: assistant
created: 2026-08-21T09:32:38Z
actor: claude-fable-5
kind: vein
ratified:
  by: claude-fable-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/queries.rs
  at: dc4d3311848a
- rel: supports
  to: it-p8rp
  at: 4
---

`arc-bounded-return`: the harvest reconcile joins only reports registered after this dispatch; a carried prior report is evidence, never a return

The time bound under the report join of cl-cjbb (it-p8rp). queries::latest_report_doc takes the arc's dispatch stamp and filters the supports-linked report set by it before newest-created wins; queries::arc_dispatched_at derives that stamp from the log — the newest `op: dispatch` event on the item — and render::harvest passes it at the one call site. This amends cl-cjbb's join sentence: the key is kind report PLUS a supports edge to the item PLUS registration at or after this arc's dispatch, and newest-created wins among those.

Why time and not authorship: a report is honestly registered by the agent under its badge or by the dispatcher afterward, so who wrote it is no join key — but what a prior arc's report can never be is younger than the dispatch it precedes. Why the log and not the held entry's `since`: both stamp the same instant, and the held entry dies at the clear, so a harvest re-run after landing would lose the bound; the log is the durable seat. The bound is inclusive because the clock is second-resolution (Store::now zeroes nanoseconds) — a report registered inside the dispatch's own second can only be the arc's own. An item that never flew (arc_dispatched_at → None) filters nothing: the degradation is to the old unfiltered pick, never to empty.

The carry itself is deliberate and stays. A dispatcher links a prior arc's report to an item with supports so the brief's evidence section shows it ("a prior dispatch's report — read it before repeating its ground", render::brief) — that is the edge doing its job. The defect was only that the harvest reconcile read the same edge as this arc's RETURN: before the arc had registered anything, newest-wins handed a stranger's prose to declared_user_owned_calls. At the it-tanf harvest that printed the false flag "the registered report carries no user-owned calls section — absence is itself the flag" about do-4y49, the it-csm3 attention report carried in for its caveat. The same unbounded pick would have let a re-dispatch reconcile against its predecessor's report, which cl-cjbb's own body claimed it did not. Until this arc registers, the await arm speaks — framings::user_owned_await, whose "no report file to parse yet" now means no report of THIS arc.

Pinned by the_reconcile_never_parses_a_report_that_predates_the_arcs_dispatch in tests/basic.rs: the pure pick bounded and unbounded, the harvest surface's await arm with the carried report's declaration absent from it, the arc's own report displacing the carried one once registered, and the re-dispatch direction measured on explicit stamps (the end-to-end half backdates the carried report on disk rather than racing a second-resolution clock). Probed against the regression it guards — restoring the unbounded call fails the suite, printing the false reconcile.
