---
id: it-awhz
type: item
title: 'a scoped affirm prints the unscoped all-clear: --to a target with no behind edge says nothing behind'
v: 3
status: sketch
provenance: assistant
created: 2026-08-21T11:44:38Z
actor: claude-fable-5
kind: bug
acceptance:
- 'a zero-count affirm says which zero it is: a scoped affirm that restamped nothing names the scope it was given and never reads as a statement about the whole node, while an unscoped zero keeps meaning every ref is current'
witness:
- line: 'a zero-count affirm says which zero it is: a scoped affirm that restamped nothing names the scope it was given and never reads as a statement about the whole node, while an unscoped zero keeps meaning every ref is current'
  by: chat:b8214cb4-bf6e-42fa-9ceb-317d5660ca2d
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Noticed at the it-dprv landing, following the it-dt68 report's recipe verbatim (`q affirm cl-zj2c --to it-dt68`) and getting an all-clear that was not one.

main.rs's Affirm arm prints one message for a zero count: `ops::affirm(&store, &node, to)` returns the number of refs restamped, and `if count == 0` prints "nothing behind — no restamp needed." That line is the UNSCOPED statement — every ref on this node is current — but it also prints when the count is zero only because `--to` named a target this node has no behind edge toward. The two states read identically, and the reader cannot tell them apart from the output.

MEASURED, 2026-08-21. `q affirm cl-up6s --to it-dprv` printed "nothing behind — no restamp needed." while `q query behind` showed cl-up6s behind on two refs in the same second (src/render.rs and src/coord.rs, both sev 4). The same call shape had already produced the same line for cl-zj2c and cl-p4k2 at this arc's join, both of which were genuinely drifted against src/teach.rs — I only found the real drift by reading `q query behind` and grepping for the ids, and affirmed them with the unscoped `q affirm <id>`.

WHY IT MATTERS HERE. The behind backlog is 110 entries, so a silently skipped affirm is invisible; and the recipe an agent copies from a landed report is exactly the scoped form. An agent that ran the scoped call, read the all-clear, and reported the affirm done would have left a ratified claim behind on the file its own diff drifted — the affirm accounting reporting the opposite of the truth, which is the same species of sight failure as it-bj3b's silent discard, one surface over.

THE CURE IS THE MESSAGE, NOT THE COUNT. A zero under a `--to` wants a line that names the scope ("nothing behind toward <target>") and, since the node's real state is one derivation away, ideally says whether the node is behind anywhere else with the unscoped command in hand. The unscoped zero keeps the line it has. Nothing about the restamp behaviour is wrong — `--to` correctly limits to one target.

Also worth settling in the same pass: `--to` accepts a file ref (`q affirm cl-up6s --to file:src/teach.rs` restamped 1, which is exactly the right affordance for an agent that reviewed one file of a multi-file claim), and nothing in the verb's help or examples says so.
