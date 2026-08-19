---
id: it-rmqy
type: item
title: 'the spawn line misdirects the fork: in-root phrasing sends a worktree agent out of its isolation'
v: 4
status: ready
provenance: assistant
created: 2026-08-19T10:17:30Z
actor: claude
kind: bug
acceptance:
- 'the spawn line printed at dispatch names no working directory: followed verbatim it is the join command with the store pin, correct from the canonical tree and from a worktree fork alike'
- 'a join whose working checkout differs from the store root opens the brief by saying so: the agent works where it stands, and acts, stamps, and observed writes land at the canonical graph'
witness:
- line: 'the spawn line printed at dispatch names no working directory: followed verbatim it is the join command with the store pin, correct from the canonical tree and from a worktree fork alike'
  by: chat:f2fea7f8-a89a-4143-860e-d3196969d549
  session: dispatcher
  kind: dispatch
  date: 2026-08-19
- line: 'a join whose working checkout differs from the store root opens the brief by saying so: the agent works where it stands, and acts, stamps, and observed writes land at the canonical graph'
  by: chat:f2fea7f8-a89a-4143-860e-d3196969d549
  session: dispatcher
  kind: dispatch
  date: 2026-08-19
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/ops.rs
  at: 2f23adcb5114
- rel: about
  to: file:src/render.rs
  at: 4fb197f5a445
- rel: depends-on
  to: dc-g5x5
  at: 3
- rel: depends-on
  to: dc-zbxj
  at: 1
---

Witnessed 2026-08-19 from the dispatcher seat during the worktree surface walk. The spawn line composed at dispatch (src/ops.rs) reads "in {root}, run: q join <token> --store {root}" with {root} the canonical store. A worktree agent obeying "in {root}" leaves its fork: file work lands in the canonical tree and the isolation is defeated — or the agent edits in the fork but runs verbs from the canonical root, and blob stamps hash canonical content it never wrote (the work root follows cwd). The clause predates the worktree case dc-g5x5 landed; under the pin, cwd is irrelevant to where graph acts land, so directing the agent anywhere serves nothing and only misdirects.

Fix shape ratified by the user 2026-08-19, as presented from this seat: (A) the spawn line goes location-neutral — the "in {root}" clause drops; what remains is the join command with the store pin, correct from any cwd. (B) join, which already resolves the working checkout separately from the store root, detects the fork case and opens the brief by saying so — work where you stand; acts, stamps, and observed writes land at the canonical graph. Rejected: a worktree flag on dispatch — dispatch cannot know where the agent will come to sit; isolation is chosen at spawn time in the harness, and detection at join reads the real cwd. Builds on dc-zbxj (the one-line hand-off this line is part of).
