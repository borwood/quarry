---
id: cl-kr7f
type: claim
title: '`location-neutral-handoff`: the spawn line names no working directory; join reads the real cwd and opens the brief with where-you-stand'
v: 7
status: ratified
provenance: assistant
created: 2026-08-21T08:34:03Z
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
  to: file:src/ops.rs
  at: 7785f1965da3
- rel: supports
  to: it-rmqy
  at: 8
- rel: source
  to: file:tests/worktree_proof.rs
  at: a6c6cd238f8d
---

The hand-off carries no working directory (it-rmqy). ops::dispatch composes the one-line spawn prompt as the join command plus the --store pin alone; the old "in <root>," clause is gone. Dispatch cannot know where the harness will sit the agent, and under the pin (dc-g5x5) cwd decides nothing about where acts land, so any directing clause can only misdirect: a worktree-isolated agent obeying it leaves its fork and lands file work in the canonical tree — or edits in the fork while running verbs from canon, where the work root follows cwd and its blob stamps hash content it never wrote. Followed verbatim the line is now correct from the canonical tree and from a fork alike.

Where the agent actually stands is read at join, from the real cwd. ops::fork_banner returns the WHERE-YOU-STAND line when store.work_root differs from store.root, and ops::join prepends it to the rendered brief, so a fork join OPENS with it: work where you stand (read, edit, build, and test in the fork; never cd to the canonical tree to run a verb), while q acts, blob stamps, and every hook-observed write land at the canonical graph whatever the cwd, the fork's own graph/ copy staying inert. Composed at the join rather than inside render::brief deliberately: the brief's second line promises everything below it is derived from the graph at render time, and this fact is read from the process, not the graph. The banner is the ONLY place the split is stated: main.rs's Join arm carried its own pinned-store paragraph, printed immediately before the brief, so a fork join said the same thing twice in adjacent paragraphs — that block is gone, subsumed by a banner that states the graph half and the file half together.

Rejected at ruling time (user, 2026-08-19): a worktree flag on q dispatch — isolation is chosen at spawn time in the harness, so only the join sees the truth. Extends cl-aujk (the pin lifecycle: dispatch still stamps --store into the line) and cl-nzjs (the worktree-dispatch receipt); the one-line shape of dc-zbxj, cl-p6aj, and cl-6ctr is untouched — the line lost a clause, not its shape.

Pinned end to end through the spawned binary in tests/worktree_proof.rs, the instrument the first arc's lease denied it. the_store_pin_carries_a_worktree_dispatch_end_to_end now carries both halves at the seat a worktree agent actually sits in: on the dispatch side no directing clause stands between the announcement and the command, and stripping the single "--store <root>" occurrence leaves the root named nowhere else in the line; on the join side the banner is the very next thing the join says after the bind (nothing stands between them), it names both roots, and it is said once. a_canonical_join_says_nothing_about_where_it_stands carries the negative half — a join from the store root and a re-join from a subdirectory of it both say nothing about where they stand, so the walk-up that finds the graph from a nested cwd never fakes a fork. Both asserts were probed against the regression they guard: re-adding the "in <root>," clause and re-adding a second orientation paragraph each fail the suite.
