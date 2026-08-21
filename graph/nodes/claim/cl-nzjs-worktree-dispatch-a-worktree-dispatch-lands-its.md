---
id: cl-nzjs
type: claim
title: '`worktree-dispatch`: a worktree dispatch lands its whole arc at the canonical store while cwd sits in the fork'
v: 3
status: ratified
provenance: assistant
created: 2026-08-18T08:51:53Z
actor: claude-fable-5
kind: feature
ratified:
  by: claude-fable-5
  date: 2026-08-18
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:tests/worktree_proof.rs
  at: a6c6cd238f8d
- rel: supports
  to: it-s789
  at: 7
---

The capability dc-g5x5 ruled, landed under it-s789: a worktree dispatch runs its whole arc — join, badged mints, hook-observed writes, harvest — landing at the canonical store while cwd sits in the fork. Blob stamps hash the fork content the agent actually touched against store-relative paths (the mint stamped the diverged fork blob, not the canon copy); observed writes accrue canonically under the badge; the fork own graph/ checkout is byte-identical across the whole arc — never written — and freehand fork-graph edits deny exactly as in the canon (C6). The instrument: the_store_pin_carries_a_worktree_dispatch_end_to_end in tests/worktree_proof.rs spawns the real binary with cwd in a git worktree and pins the chain end to end; a_dead_pin_refuses_instead_of_scattering pins the loud-refusal arm — a pin naming a moved graph errors, never silently falls back to a fork cwd.
