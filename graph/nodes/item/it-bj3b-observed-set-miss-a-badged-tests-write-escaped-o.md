---
id: it-bj3b
type: item
title: 'observed-set miss: a badged tests write escaped observation'
v: 9
status: done
provenance: assistant
created: 2026-08-14T06:42:48Z
actor: claude-fable-5
kind: bug
acceptance:
- 'lands nothing new: the two in-tree misses carry a recorded diagnosis claim naming their mechanism with evidence as source; a badged write whose path fails store-relative resolution is recorded and rendered at harvest, never silently discarded; a regression instrument proves Windows absolute tool-write paths land store-relative in the observed set; the shell hook parses common write shapes best-effort and accrues resolvable targets; and every observed-vs-leased rendering states the sight boundary so a partial observed set cannot read as complete'
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Two in-tree instances: 2026-08-14 (it-hapc harvest: tests/** reported leased-but-untouched while tests/basic.rs was genuinely modified under the badge — the test exists, the diff confirms it, the suite counts it) and again during the 2026-08-17 inaugural in-tree dispatches (th-xeqw evidence note: "twice now"). The th-xeqw settled diagnosis (2026-08-18, user-accepted) confirms this item SEPARATE from the worktree store-fork class dc-g5x5 fixed: it reproduces in the main working tree. User ruling on record: a failure to observe a file change is a defect on its face; observation accounting must be trustworthy before the successor repo adopts the tool.

The in-tree suspects (code-read 2026-08-20):
- Shell-made writes are invisible by design: the guard hook matcher covers Write|Edit|NotebookEdit only; a file written via shell (heredoc, redirect, git apply) never reaches observe_write. Harvest admits this only for the EMPTY observed set ("the agent's shells ran outside the guard's sight"); a partial observed set renders with no caveat and reads as complete.
- Windows relativization fragility: tool hooks carry absolute paths; relativization against the store root precedes observe_write, and any failure (drive-letter case, separator mixing) falls through to the contains(':') filter (teach.rs lease_check/observe_write), which on win32 silently discards every absolute path. The drop leaves zero trace — the heart of the defect: accounting data under a badge silently discarded, so a miss cannot even be diagnosed later.

The settled shape (user-agreed 2026-08-20) — verify and instrument:
1. DIAGNOSE AGAINST THE RECORD: pin the two misses to their mechanism (incident-day state, or reproduction by shape) and land the diagnosis as a claim with the evidence as source.
2. NO SILENT DISCARD UNDER A BADGE: a badged write whose path fails to resolve store-relative is recorded (counted with its raw path) and rendered at harvest — resolution failure becomes visible accounting, never a dropped fact.
3. INSTRUMENT the resolution path: a regression proving Windows absolute tool-write paths (mixed case, mixed separators) land store-relative in the observed set.
4. PARSE COMMANDS FOR COMMON WRITE SHAPES (user addition 2026-08-20): the shell hook already receives every command string — parse the common write shapes (redirect >/>>, tee, cp/mv targets, touch, git apply/checkout --, PowerShell Set-Content/Out-File/Add-Content) and accrue targets that resolve store-relative. Best-effort by nature, same posture as the commit-sweep guard: parsing narrows the blind zone, the boundary statement covers what it cannot see.
5. STATE THE SIGHT BOUNDARY wherever observed-vs-leased renders: tool writes observed, shell writes parsed best-effort, the residue outside the guard's sight — a partial observed set must never read as complete.
