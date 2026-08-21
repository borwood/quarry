---
id: cl-2jas
type: claim
title: '`commit-set`: the log owns the uncommitted graph - last writer since the prior commit; sweeps deny, explicit paths pass'
v: 3
status: ratified
provenance: assistant
created: 2026-08-20T08:51:54Z
actor: claude
kind: vein
ratified:
  by: claude
  date: 2026-08-20
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
- rel: source
  to: file:src/teach.rs
  at: d15f00a82b95
- rel: supports
  to: it-4q6t
  at: 9
---

queries::pending_graph interrogates git (status --porcelain -uall over graph/) and derives ownership at read: a node file belongs to the session with the LAST stamped event on the node since the file's prior commit (queries::commit_sets, the pure core); earlier editors in the same window are carried, annotated, so a mixed-authorship file lands in exactly one commit-set. teach::sweep_shape string-matches bulk staging best-effort by design (git add graph / graph dirs / -A / . / -u; git commit -a as a tracked-only capture); teach::staging_guard rides the Bash|PowerShell PreToolUse hook on the C6 channel and denies a sweep only when it would capture another session's node files - the asking session gets its own git add line in hand, foreign sets list with the owner's last-seen age, and an owner quiet past OWNER_STALE_SECS (an hour; never-seen counts stale) flips to an explicit-path adoption offer naming the origin, which passes by construction. Explicit-path staging always passes; the monthly log shard is exempt ledger (internally attributed, never split, rides along with any commit); unstamped files block no sweep; a worktree fork is skipped (work_root differs from root - the fork's graph checkout is inert). q query commit-set is the pull handle both refusal and wrap advertise; wrap echoes the ownership split whenever graph changes are pending; q session retire reads the retiree's set before the heartbeat clears and offers the leftovers their own labeled commit. Landed under it-4q6t. Pinned by sweep_shapes_match_bulk_staging_and_explicit_paths_pass, commit_set_ownership_derives_last_writer_and_annotates_carried_edits, and staging_guard_denies_the_sweep_with_the_askers_line_and_offers_adoption_when_stale in tests/basic.rs.
