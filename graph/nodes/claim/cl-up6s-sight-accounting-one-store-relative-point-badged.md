---
id: cl-up6s
type: claim
title: '`sight-accounting`: one store-relative point; badged resolution failure…'
v: 9
status: ratified
provenance: assistant
created: 2026-08-20T09:32:21Z
actor: claude
kind: vein
ratified:
  by: claude
  date: 2026-08-20
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/teach.rs
  at: d15f00a82b95
- rel: source
  to: file:src/store.rs
  at: bcd701ef3890
- rel: source
  to: file:src/render.rs
  at: ceb6cf060a06
- rel: source
  to: file:src/coord.rs
  at: 24df8ce05680
- rel: source
  to: file:tests/observed_set.rs
  at: 845c755fae73
- rel: supports
  to: it-bj3b
  at: 9
- rel: supersedes
  to: cl-h2gx
  at: 5
---

The observation layer's accounting invariant (it-bj3b): a badged write lands in the observed set or is recorded failing to — never silently discarded. Four parts, one system.

ONE RESOLUTION POINT: store::store_relative (with Store::relative in hand) is the single store-relative resolver for observation and lease judgment — case-folded and separator-normalized on both sides (Windows tool hands deliver mixed case and mixed separators), work root stripped before store root (dc-g5x5: a fork mirrors the store's layout), component boundary held. Both hook arms (the write guard and the session hook's shell observer) resolve through it; a divergent second strip was the class of silent discard the defect named.

NO SILENT DISCARD UNDER A BADGE: teach::observe_write records a badged write whose path resolves nowhere — raw path, marked unresolved in the touched JSONL (coord::Touch / accrue_touch_ext) — and render::harvest renders the unresolved set at the judgment seat. Leaseless out-of-repo writes stay unrecorded (scratchpads are not this graph's arc); only a badge makes the failure accounting.

THE SHELL CHANNEL: teach::write_shapes parses a command string for the common write shapes — redirects (glued forms included), tee, cp/mv/touch targets, git checkout -- and git restore, the PowerShell content writers — and teach::observe_shell (riding the Bash|PowerShell session hook beside the staging guard) accrues every target that resolves store-relative, marked via:shell. Best-effort in the commit-sweep guard's posture: observation only, never a denial, never a spoken line; a parse is a guess, so unresolvable parsed targets drop — only the tool-write channel records unresolved paths, because there the write is a certainty.

THE BOUNDARY STATED: framings::SIGHT_BOUNDARY renders wherever an observed set faces a reader — harvest's OBSERVED vs LEASED (empty set included), the dispatch trace, wrap's leaseless pickup — naming the three strata: tool writes exact, shell writes parsed best-effort, the residue beyond both. A partial observed set can no longer read as complete.

Pinned by store_relative_survives_windows_case_and_separator_mixing, write_shapes_parses_the_common_write_shapes, observe_shell_accrues_only_resolvable_targets_marked_shell, and a_badged_write_failing_resolution_is_recorded_and_harvest_states_the_boundary in tests/basic.rs, and end-to-end through the spawned binary by badged_writes_land_or_are_recorded_never_dropped in tests/observed_set.rs. Diagnosis of the incident class: cl-qat8.
