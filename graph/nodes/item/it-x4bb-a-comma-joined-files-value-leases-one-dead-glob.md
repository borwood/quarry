---
id: it-x4bb
type: item
title: 'a comma-joined --files value leases one dead glob: the dispatched agent is denied its own write-set'
v: 4
status: done
provenance: assistant
created: 2026-08-21T08:34:57Z
actor: claude-fable-5
kind: bug
acceptance:
- 'a dispatched agent is never denied a write to a path its own brief prints as leased: a comma-bearing --files value is caught where the globs enter — split into its globs or refused with the repeatable flag taught, never leased whole — and q reserve inherits the same check'
witness:
- line: 'a dispatched agent is never denied a write to a path its own brief prints as leased: a comma-bearing --files value is caught where the globs enter — split into its globs or refused with the repeatable flag taught, never leased whole — and q reserve inherits the same check'
  by: chat:b8214cb4-bf6e-42fa-9ceb-317d5660ca2d
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Witnessed from the agent seat on it-rmqy, 2026-08-21. q dispatch --files is a repeatable flag (Vec<String>, no value delimiter). A dispatcher who passes one comma-joined value — here --files "src/ops.rs,src/render.rs,tests/**" — mints a lease holding ONE glob whose static prefix is the entire string. coord::globs_overlap compares static prefixes by the prefix-of relation, so that glob matches only the FIRST path in it: writes to src/ops.rs pass, and writes to src/render.rs and tests/** are DENIED by the badge's own write guard (teach::lease_check's under-a-badge arm) as "outside the leased write-set" — the very files the brief prints as leased and the item's own file edges name.

Three properties make it a defect rather than a typo. It is silent at the firing station: nothing at dispatch or reserve looks at the shape, though a comma can never appear in a path glob here. It surfaces only at the agent's first denied write, mid-arc, in a seat that cannot fix it — extending a lease is release plus re-reserve, and the actor rules forbid an agent releasing its lease, so the teaching line ("ask the dispatcher to extend the lease") is the only road and it costs a re-dispatch. And it degrades quietly rather than failing: the first path keeps working, so the arc starts, spends context, and dies at whichever write comes second.

Evidence: the it-rmqy arc lost its src/render.rs and tests/** halves to exactly this — the fix landed in src/ops.rs alone and the instrument that would pin it could not be written.

The fork to settle at build time: SPLIT comma-bearing --files values where the globs enter, or REFUSE the shape loudly and teach the repeatable flag. Splitting is the least-surprise reading of the dispatcher's intent; refusing keeps one meaning per flag and never guesses. Either check is cheap and belongs at the entry point both stations share (ops::dispatch and coord::reserve, so q reserve inherits it — the same value shape reaches both).
