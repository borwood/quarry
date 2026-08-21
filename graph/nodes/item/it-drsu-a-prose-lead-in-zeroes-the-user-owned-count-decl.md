---
id: it-drsu
type: item
title: 'a prose lead-in zeroes the user-owned count: declared calls read as none when a line stands between the head and the list'
v: 3
status: sketch
provenance: assistant
created: 2026-08-21T09:38:03Z
actor: claude-fable-5
kind: bug
acceptance:
- 'a report''s declared user-owned calls are counted whatever prose stands between the section head and its entries: a lead-in line never reads as a declaration of none, and the count still ends at the next heading'
witness:
- line: 'a report''s declared user-owned calls are counted whatever prose stands between the section head and its entries: a lead-in line never reads as a declaration of none, and the count still ends at the next heading'
  by: chat:b8214cb4-bf6e-42fa-9ceb-317d5660ca2d
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Witnessed at the it-p8rp harvest, 2026-08-21, from the judgment seat — the reconcile printed 'report declares 0, thread(s) filed under this badge: 1' and confronted a gap, while the registered report do-yeum plainly declared one call and named the thread it filed. The agent was honest; the parser misread it.

queries::declared_user_owned_calls finds the section head, then walks the lines under it counting list entries. Its loop breaks on the first non-blank, non-indented line that is neither a list entry nor the bare word none. do-yeum's section reads:

  ## user-owned calls
  (blank)
  **One.**
  (blank)
  1. **The ratified user_owned_await wording now carries a second meaning** ...

'**One.**' is a prose lead-in announcing the count. It is not an entry: the entry test wants '- ', '* ', '· ', '• ' or a digit followed by '.' or ')', and '**One.**' opens with two asterisks, so starts_with("* ") is false. It is not the none escape either: trimming '*' and '.' leaves 'One', not 'none'. So the loop breaks with n == 0 and the function returns Some(0) — the section is present and read as declaring nothing, and the numbered entry two lines below is never reached.

The direction of the failure is what makes it a defect rather than a wart. Here a thread existed, so the mismatch surfaced as a gap and a human looked. The dangerous case is the same report shape with the thread genuinely unfiled: declared 0 against filed 0 renders 'reconciled', a false all-clear over a call the report explicitly declared. cl-cjbb's whole point is that the section and the threads are the machine's only sight of a semantic judgment no hook can see; a parse that silently reads 'One.' as zero puts a hole exactly there.

Related but distinct from the shape it-p8rp just fixed: that one joined the wrong report, this one misreads the right one. Both end at the same surface with the same misleading sentence.

The fork for the builder: skip prose lines between the head and the first entry rather than breaking on them (the loop already has a continue arm for indented continuations, so the break is doing double duty as 'section ended'), or teach the head-line parse to read a spelled count. Skipping is the least-surprise reading — a lead-in sentence before a list is ordinary prose, and the section's real end is the next heading. Note the break must still fire on a genuine new section, so 'ends at a markdown heading or after entries have started' is the shape to aim at, not 'never break'.
