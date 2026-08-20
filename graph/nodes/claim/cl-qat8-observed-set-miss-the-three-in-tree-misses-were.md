---
id: cl-qat8
type: claim
title: '`observed-set-miss`: the three in-tree misses were shell-made writes -…'
v: 6
status: asserted
provenance: assistant
created: 2026-08-20T09:31:57Z
actor: claude
kind: reading
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:docs/reports/2026-08-17-vein-prompt-it-pgn9.md
  at: ebdbbc678e8b
- rel: source
  to: file:docs/reports/2026-08-14-fire-time-liveness-it-hapc.md
  at: 29c99936f51b
- rel: source
  to: file:docs/reports/2026-08-17-assay-office-it-swsy.md
  at: e1e018f2a946
- rel: supports
  to: it-bj3b
  at: 8
---

Diagnosis of the it-bj3b observed-set misses, pinned by reproduction by shape against the incident record (2026-08-20). MECHANISM: matcher-scoped observation — the observation layer rode only the Write|Edit|NotebookEdit PreToolUse hook, so a badged write made through a Bash/PowerShell command (heredoc, redirect, tee) fired only the session hook, which observed nothing; the write escaped the observed set with zero trace. The second suspect, Windows relativization fragility, is EXCLUDED for the recorded instances and now pinned green.

THE RECORD — three instances, all tests/basic.rs under a badge in the main working tree: do-hmac (it-hapc, 2026-08-14: harvest showed tests/** leased-but-untouched while the tree diff confirmed the modification); do-gkpp (it-swsy, 2026-08-17: the observed list omitted tests/basic.rs while the arc's diff showed +240 lines there); do-h3z4 (it-pgn9, 2026-08-17: same miss, and the discriminating observation — "instances now alternate (runs 3 and 9 missed, runs 4-6 observed)").

WHY THE ALTERNATION DECIDES IT: all runs shared one machine, one main working tree (root == work_root — the worktree store-fork class th-xeqw settled is separate by its own diagnosis), one harness, one store root. A deterministic environment failure (drive-letter case, separator mixing in the strip) would have missed every run or none; what varied per run was the agent's choice of write instrument. Incident-day code state confirms no live resolution seam: at a29c8d6 (2026-08-14, instance 1's build) the guard already case-folded and separator-normalized both sides before the prefix strip (git show a29c8d6:src/main.rs). And the sequence evidence closes the tool-write road: after the join gate (dc-zbxj, landed 2026-08-13, before all three instances) an unjoined tool write into a dispatched zone denies loudly and a joined one accrues — for a badged agent in the main tree, the only SILENT road past observation for a leased file was a shell-made write.

REPRODUCTION BY SHAPE: by construction the hook wiring routes only Write|Edit|NotebookEdit through the observation path, so a heredoc write reaches no observing code — the shape reproduces without a live incident. The regression instrument (tests/observed_set.rs) now pins the excluded suspect green: mangled Windows absolute tool-write paths (uppercased, mixed separators) land store-relative in the observed set, and a genuinely unresolvable badged path is recorded raw, never dropped. The repair narrows the diagnosed blind zone: teach::write_shapes/observe_shell parse the common shell write shapes into the observed set marked shell-parsed, and framings::SIGHT_BOUNDARY states the residue wherever observed-vs-leased renders.
