---
id: th-mbbb
type: thread
title: 'worktree dispatch, the dispatcher''s side: when to isolate, and how the code comes home'
v: 1
status: queued
provenance: assistant
created: 2026-08-19T10:17:52Z
actor: claude
edges:
- rel: about
  to: ar-xa38
  at: 1
---

Plea from the dispatcher seat, 2026-08-19 surface walk, ahead of the first live worktree dispatch. dc-g5x5 lands the graph side whole: a worktree dispatch's acts, stamps, and observed writes reach the canonical store while cwd sits in the fork, and the fork's own graph copy stays inert. What no surface encodes is the dispatcher's side of the arc. (1) When does a dispatch warrant worktree isolation? Candidate reasons seen from this seat: parallel dispatches whose write-sets or build resources would collide, and the exe-lock dance do-799m records — a fork build never locks the canonical q.exe the hooks invoke. (2) How does the code come home? The fork branch's merge into the canonical tree is on no surface: by whose hand, and ordered where against harvest, status=done, and release. dispatch --help, harvest --help, and the guide's dispatch block are silent; the return-is-a-report doctrine covers judgment, never collection. (3) The regeneration seam: a slice touching src/teach.rs owes rebuild plus q init --claude — in a fork, whose tree does init write under the store pin, and when does the canonical side regenerate? Untested; the first live run avoids it in the fork and owes the answer here. Evidence from the it-gwj7 worktree run lands on this thread.
