---
id: cl-74qt
type: claim
title: '`glob-shape-floor`: a comma-joined --files value is refused where the globs enter — one predicate, both stations, ahead of every mutation'
v: 5
status: ratified
provenance: assistant
created: 2026-08-21T09:52:24Z
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
  to: file:src/coord.rs
  at: 1cb9b241aa5a
- rel: source
  to: file:src/ops.rs
  at: c282feb29612
- rel: supports
  to: it-x4bb
  at: 4
---

`glob-shape-floor`: a comma-joined --files value is refused where the globs enter — one predicate, both stations, ahead of every mutation

coord::check_glob_shapes is the one point that judges the SHAPE of a lease's globs (it-x4bb). --files is repeatable and carries no value delimiter, so a comma-joined value arrives as ONE glob; coord::globs_overlap compares STATIC PREFIXES by the prefix-of relation, so that glob matches the FIRST path in the value and nothing after it. Every later path is leased in name only: render::brief prints it as leased and the item's own file edges name it, and teach::lease_check's under-a-badge arm then denies the agent's write to it as "outside the leased write-set" — mid-arc, in a seat that cannot fix it, since extending a lease is release plus re-reserve and an agent may not release its own lease. It degraded quietly rather than failing: the first path kept working, so the arc started, spent context, and died at whichever write came second. The it-rmqy arc lost its src/render.rs and tests/** halves to exactly this.

REFUSED, never split — the fork it-x4bb left to build time, settled at the build. One meaning per flag, never a guess: a comma is legal inside a real filename (and inside a brace alternation this matcher does not read), so a silent split can mint exactly the dead globs the check exists to prevent, in the one direction nobody would look. The refusal instead costs one re-run at the station that CAN fix it, and teaches the repeatable flag once: it names the offending value, states the mechanism, derives the corrected command from the value in hand (--files "a" --files "b" --files "c"), and names the write-set authoring road the fallback comes from.

Two call sites, the stations it-x4bb named. coord::reserve calls it, so every lease-taking road inherits the floor — q reserve and the dispatch fallback to an item's recorded write-set alike, the latter refusing beside the foreign-lease refusal and like it. ops::dispatch calls it on the FLAG value ahead of everything that mutates: ahead of ops::acceptance_backstop, ownership, the brief event, the lease, and the in-flight flip — so a mistyped fire has nothing to undo (no gate-refusal event, C8 unsatisfied, status untouched). Deliberately NOT extended to q set write-set+=: the fire-time floor is the load-bearing one — no agent can be denied its own write-set — and a new refusal at a shaping station was not this item's to add. Both stations teach the repeatable flag in their own --help besides, so the shape is stated before it is refused.

Pinned by a_comma_joined_files_value_is_refused_where_the_globs_enter in tests/basic.rs: the defect measured on globs_overlap itself (the joined value matches its first path alone), both stations refusing with the teaching, the write-set fallback road, the pre-mutation property, the repeatable-form positive control, and end to end through the spawned binary — the value reaches the station unsplit, and no lease ever holds a comma-bearing glob. Each half probed against the regression it guards: removing the check fails the reserve refusal; removing only the ops::dispatch call site fails the no-brief-event assert while the reserve arm still passes.
