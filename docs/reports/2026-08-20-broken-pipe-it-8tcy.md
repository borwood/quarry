# Dispatch report — it-8tcy: the pipe dies quietly

Third worktree dispatch (serial run). Agent report condensed faithfully; dispatcher's
landing addendum follows.

## Outcomes against the RETURN SPEC

**1. Verbs complete every state mutation regardless of stdout's fate; a closed pipe
exits quietly without panic — MET.** New src/emit.rs: `outln!`/`out!`/`errln!`
macros writing via `writeln!`/`write!` and swallowing io errors. Every print site
rides them: 290 `println!`, 8 `print!`, 4 `eprintln!` in src/main.rs (mechanical
rename, zero leftovers, zero string-literal collisions), plus the serve loop's
stderr line in src/view.rs. A refusal path's `exit(2)` now reaches its exit code
even with dead stderr (before: panic exit 101 — a deny the harness might not
honor). Proven by spawning the real binary with the pipe's read end dropped (os
error 232 class): exit 0, no panic residue.

**2. Verb paths audited for mutations sequenced after prints; each survives output
failure — MET.** Full inventory of post-print state work in src/main.rs (mint-time
links, watermarks, `consume_readings`, `ratify_landing`, `clear_dispatch`,
`view::write`, wrap cursor, brief's C8 event — the harvest block being the largest).
**Nothing needed reordering**: the helper alone carries the invariant. Confirmed
live by `a_closed_pipe_never_panics_and_state_after_the_print_still_lands`
(tests/broken_pipe.rs): with stdout dead, a done-flip still ratifies, sweeps its
consumed reading, and regenerates the view. Two delivery-loss (not state) cases: a
dead pipe can lose the spawn prompt or a gate's resume token — both re-mintable.

**3. Homework re-derives on demand — MET.** Derivation moved out of
`print_homework` into `render::homework` (one derivation point); the act-time print
is a thin delivery whose header teaches the pull; new `q query homework <node>`
pulls per node, honest silence when empty. Pinned at library level and end-to-end
in the incident's shape (edit lands into a dead pipe; the query recovers the exact
behind line with its affirm command).

## Test results (verbatim, read raw)

```
test result: ok. 112 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.54s   (tests\basic.rs)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s    (tests\broken_pipe.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s    (tests\store_lint.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s    (tests\surface_lint.rs)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s    (tests\worktree_proof.rs)
```

No compiler warnings.

## Mechanics

- Branch `worktree-agent-aa6f1014952c7a5be`, commit `5e48341` ("The pipe dies
  quietly: prints swallow io errors, homework pulls on demand"). Fast-forwarded
  into main at landing. teach.rs untouched — no regeneration owed.
- Files: src/emit.rs (new), src/lib.rs, src/main.rs, src/render.rs, src/view.rs,
  tests/basic.rs, tests/broken_pipe.rs (new).
- Graph writes under the badge: vein cl-8h6r (`nonfatal-print`, source
  file:src/emit.rs) and feature cl-ugqu (`homework-query`, source
  file:src/render.rs), both supports it-8tcy.

## Agent reflections (verbatim highlights)

- The rename was 302 sites in one file, safe only via counting each macro family
  and checking string literals first; a print seam from day one would have made
  this a two-line fix. main.rs at 3,200 lines is th-yzmj's file-size infection
  wearing another coat.
- The audit's honest conclusion surprised me: nothing needed reordering. The
  hazard was never the order — it was that the print channel could throw at all.
- Doubt worth recording: the broken-pipe test's drop-the-read-end races the
  child's first write; assertions hold either way, but the failure path is
  exercised probabilistically. A deterministic harness needs a pre-closed handle
  at spawn, which std doesn't offer portably.
- Worktree friction: the shell sandbox refused live smokes in a scratch store
  outside the fork — a dispatched agent has no sanctioned shell path to a
  throwaway store from inside a fork (second sighting of scratch-store friction,
  different cause). Canonical q verbs from the fork ran fine throughout.

## Dispatcher's landing addendum

Third consecutive worktree arc, same collection order, fast-forward again, dc-g5x5
clean again. Post-merge `q query behind` shows 84 sev-4 file-drift entries — the
settled-strata phenomenon th-ybv9 describes, accumulated across this serial run's
three merges plus older stamps. Not mass-affirmed at this landing (affirm is a
recorded act of review); the house precedent is a dedicated source-drift pass
(it-4p7m's shape) once the run settles. Only this landing's own flip-behinds were
affirmed.
