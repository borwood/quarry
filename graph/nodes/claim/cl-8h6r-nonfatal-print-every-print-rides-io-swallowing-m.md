---
id: cl-8h6r
type: claim
title: '`nonfatal-print`: every print rides io-swallowing macros - output failure never skips state work; a closed pipe exits quietly'
v: 3
status: ratified
provenance: assistant
created: 2026-08-20T09:08:02Z
actor: claude
kind: vein
ratified:
  by: claude
  date: 2026-08-20
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/emit.rs
  at: 3315e622a480
- rel: supports
  to: it-8tcy
  at: 6
---

The macros out!, outln!, errln! in src/emit.rs write via writeln! and swallow io errors: every print site in the binary rides them (all of main.rs plus the serve loop's stderr line in view.rs), so a verb completes all state work sequenced after prints - readings sweeps, area watermarks, ratification, log events, view regeneration - whatever stdout's fate, and a closed pipe (head -1, a dying terminal) ends output quietly with exit 0, never a panic. The sequencing audit of every verb path found no print-dependent mutation, so the helper alone carries the invariant, whatever order the code runs in. Pinned by a_closed_pipe_never_panics_and_state_after_the_print_still_lands in tests/broken_pipe.rs, which spawns the real binary with the pipe's read end dropped.
