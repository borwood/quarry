---
id: cl-2kxr
type: claim
title: '`scoped-affirm`: --to acts and reports on one target alone — the zero names its scope and the node''s reach beyond it; the unscoped zero earns its all-clear'
v: 7
status: ratified
provenance: assistant
created: 2026-08-21T12:03:22Z
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
  to: file:src/main.rs
  at: da8ef57cd4da
- rel: source
  to: file:src/queries.rs
  at: 1fdd2b2cb752
- rel: source
  to: file:src/ops.rs
  at: 55d346308d3f
- rel: source
  to: file:tests/basic.rs
  at: a0b0aeaf24a1
- rel: supports
  to: it-awhz
  at: 5
---

`scoped-affirm`: --to acts and reports on one target alone — the zero names its scope and the node's reach beyond it; the unscoped zero earns its all-clear

The affirm surface printed ONE line for a zero count (it-awhz): "nothing behind — no restamp needed." That is the UNSCOPED statement — every ref on this node is current — and it also printed when the count was zero only because `--to` named a target the node has no behind edge toward. The two states read identically and the reader could not tell them apart from the output. Measured on the real graph before the fix: `q affirm cl-up6s --to it-dprv` printed it while cl-up6s was behind on two refs in the same second, and the same call shape had already printed it over cl-zj2c and cl-p4k2, both genuinely drifted against src/teach.rs. The scoped form is not an unusual call an agent had to invent: the behind lines and the dispatch homework compose `q affirm <node> --to <target>` themselves (main.rs print_homework, render::homework, the brief's behind-check), so the machine hands out the call whose zero it could not read back.

THE CURE IS THE MESSAGE, NOT THE COUNT. `--to` limits the restamp correctly and nothing about that changed. main.rs's print_affirm now says WHICH zero it is: this node carries no ref toward the target (the incident shape), or refs toward it are behind but not restampable, or that ref is current. And every scoped line — zero or not — is followed by the node's reach beyond the scope: how many OTHER refs are still behind, with the unscoped `q affirm <id>` in hand, or the explicit statement that nothing else is behind either. A scoped line never makes a statement about the whole node, which is the invariant the defect broke.

THE UNSCOPED ZERO NOW EARNS ITS ALL-CLEAR. One branch over from the reported defect, on the road that was supposed to be safe: a ref that cannot be restamped at all — a dangling target, a missing file (queries::behind severity 2) — leaves the count at zero, so "nothing behind" was a false all-clear there too. The unscoped zero keeps its line only when the node's behind set is empty; otherwise it says how many refs it could not clear, and a nonzero unscoped restamp says what it left behind.

DERIVED AFTER THE ACT, NEVER PREDICTED. queries::behind_node is the classifier's per-node core, extracted so that `behind` maps it over the store — cl-34ra's one classification point is undisturbed, surfaces still render what they find — and so the affirm surface can ask about the single node under review without walking and re-hashing the whole graph. queries::affirm_scope splits that set at the scope the caller gave: target_known (an edge toward it, or the doc's own registered path), toward, elsewhere. print_affirm reads it after the restamp, so every line reports the state the caller now holds rather than a prediction of it.

THE SCOPE BINDS THE ACT, NOT ONLY THE REPORT. ops::affirm's path-backed-doc self-blob restamp ran regardless of `--to`: a scoped affirm on a drifted registered doc restamped the doc's own file and returned a count the caller could only read as the named target having moved — the same misreport in the other direction, a one becoming a lie where the zero was. It is now gated on the scope naming that file, spelled `file:<path>` as the homework line advertises it. A scoped affirm acts on its scope alone, or the count it returns cannot be read at all.

THE FILE-REF AFFORDANCE IS NOW STATED. `--to file:src/teach.rs` is exactly the right call for an agent that reviewed one file of a claim sourced on several, and nothing in the verb's help or examples said the flag took one. `q affirm --help` now carries it with the three example shapes and the sentence that its result speaks for that target alone.

Pinned by a_scoped_affirm_zero_is_a_statement_about_its_scope_alone (the incident in shape on the derivation itself: target_known false with elsewhere 2, then the file-ref restamp leaving elsewhere 1, then the unrestampable ref), a_scoped_affirm_restamps_only_within_the_scope_it_was_given (the doc-blob leak measured on ops::affirm), and the_affirm_surface_says_which_zero_it_is end to end through the spawned binary (all three scoped shapes, the scoped restamp naming its scope, the unscoped all-clear earned, and the unrestampable ref never collapsing into it) — all in tests/basic.rs. Each half probed against the regression it guards: restoring the old two-line surface reproduces the incident reading verbatim, collapsing the scope to None fails the scope-naming assert, and removing the doc-blob gate fails the out-of-scope restamp assert.
