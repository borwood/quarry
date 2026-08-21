---
id: cl-h9d6
type: claim
title: '`slug-follows-title`: a retitle carries the node file with it; the slug it leaves is an alias that still resolves'
v: 6
status: ratified
provenance: assistant
created: 2026-08-21T15:09:05Z
actor: claude-opus-5
kind: vein
ratified:
  by: claude-opus-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/ops.rs
  at: 2c2276685d23
- rel: source
  to: file:src/store.rs
  at: 44d71b72d0f5
- rel: source
  to: file:src/main.rs
  at: 114a318cb5ef
- rel: source
  to: file:tests/basic.rs
  at: 75f4c2e53c44
---

`slug-follows-title`: a retitle carries the node file with it; the slug it leaves is an alias that still resolves

THE SECOND FAULT OF it-8k3p, and the one that made the first permanent. A node filename is <id>-<slugify(title)>.md, derived once at mint in ops::new_node and — until this landing — never re-derived. q set title= repaired the TITLE and never the file, so a corrected title left the wrong name on disk forever and a mint-time typo could not be undone at all. Defensible on its own terms (the id is the immutable anchor, render unpacks ids to current titles, nothing downstream reads the slug) and still wrong in one respect that matters: the slug is the name the next reader greps for, and cl-gy6q wore a name nobody typed for as long as the store stood.

THE SLUG NOW FOLLOWS THE TITLE. ops::set derives the wanted path in the title arm and moves the file to it. The old slug was ALREADY recorded in front.aliases by every retitle — written by that one line and read by nobody, dead since it was added — and moving the file is what makes that record load-bearing: Store::find gained an alias rung between the live-slug rung and the title-fragment rung, so a name the node has left still answers while a live slug outranks it. Re-setting a title to the value it already carries is therefore the repair road for a file left behind by a pre-fix retitle, which is how cl-gy6q was repaired at this landing.

THE PLACEMENT IS THE RULE cl-gy6q STATES FOR ITSELF: a refused act has no side effects. The rename is decided in the field loop and COMMITTED past the last refusal — past the acceptance gate and the it-ds6b demotion — so a q set that refuses moves no file. It runs BEFORE the save rather than after it, and that ordering is the load-bearing half: rename-then-save leaves exactly one file wearing the node's id whichever step fails, where save-then-delete can leave two, and two files with one id is the one state Store::load_all cannot read straight. Store::move_node_file refuses outright if something already stands at the target.

NEVER SILENT. The move rides SetOutcome.moved to the surface (main.rs echoes "file moved: … — the slug follows the title, and <old> is kept as an alias that still resolves") and onto the set event as a renamed{from,to} field, so the log holds the name the file left. q set --help states the behaviour and the repair road, which is the half of the fork it-8k3p left open — the verb either carries the filename or says plainly that it will not; it now carries it AND says so.

MEASURED AT THE LANDING. Sweeping all 429 node files, comparing the file slug against slugify of the current title: 12 mismatches, of which cl-gy6q was the only corruption — the other 11 are honest historical retitles (the spine→vein rename, decision-time-bones, and their kin) whose files name titles that no longer exist. They are repairable one q set at a time by the road above and were deliberately left alone: each repair bumps its node and sends its citers behind for a name nothing downstream reads. Note the sweep the brief cites compared FIRST WORDS only and found one; a full-slug compare finds twelve, so a corruption deeper inside a title would have escaped the earlier method.

Pinned by a_retitle_carries_the_nodes_file_with_it in tests/basic.rs: the move and the vacated name, exactly one file per id, the alias rung answering the old slug while the live slug outranks it, the pre-fix state rebuilt on disk and repaired by re-setting the title it already carries, a refused act moving nothing, and the echo end to end through the spawned binary. Each half probed against the regression it guards: dropping the move fails the move assert, dropping the alias rung fails the old-slug resolve (and NO other test in the suite notices, which is why the rung is pinned here), and committing the rename at the decision point instead of past the gates leaves the refused act having destroyed the file name.
