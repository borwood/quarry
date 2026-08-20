# Dispatch report — it-bj3b: the observed set stops lying

Fourth worktree dispatch (serial run). Agent report condensed faithfully;
dispatcher's landing addendum follows.

## Outcomes against the RETURN SPEC (one acceptance line, five clauses)

**1. Diagnosis claim naming the mechanism, evidence as source — MET.** cl-qat8
(`observed-set-miss`, kind reading, supports it-bj3b) pins **matcher-scoped
observation**: the observation layer rode only the Write|Edit|NotebookEdit hook; a
badged shell-made write (heredoc, redirect, tee, Add-Content) fired only the
session hook, which observed nothing. Sourced to all three incident reports
(do-hmac, do-gkpp, do-h3z4 — three instances, not the item body's two). The
Windows-relativization suspect is excluded for the recorded instances by the
alternation discriminator in do-h3z4 (same machine, runs 3 and 9 missed while 4–6
observed — a deterministic path failure misses all or none; what varied was the
write instrument), incident-day code state, and the join-gate sequence.

**2. No silent discard under a badge — MET.** `teach::observe_write` records a
badged write whose path resolves nowhere — raw path, `unresolved: true` in the
touched JSONL — and the guard arm routes the previously-skipped rel-None case into
it. Leaseless out-of-repo writes stay unrecorded (only a badge makes scratch
accounting). Harvest renders the unresolved set raw at the judgment seat; the
dispatch trace marks it. Pinned in tests/basic.rs and end-to-end in
tests/observed_set.rs.

**3. Regression instrument — MET.** New single resolution point
`store::store_relative` (case-folded, separator-normalized, work-root-first per
dc-g5x5, component-boundary) replaces the inline strip; both hook arms ride it.
Pinned pure (`store_relative_survives_windows_case_and_separator_mixing`) and
through the real hook seam with a case-mangled absolute path
(`badged_writes_land_or_are_recorded_never_dropped`).

**4. Shell hook parses common write shapes — MET.** `teach::write_shapes`
(quote-aware tokenizer: redirects incl. glued `2>file`, tee, cp/mv/touch,
`git checkout --`/`git restore`, Set-Content/Add-Content/Out-File/Copy-Item/
Move-Item) plus `teach::observe_shell` beside the staging guard: resolvable targets
accrue marked `via: shell`; observation only — never a denial, never a spoken line;
unresolvable parsed targets drop (a parse is a guess; only the certain tool-write
channel records unresolved). False-negative chosen over false-positive throughout.

**5. Sight boundary stated — MET.** `framings::SIGHT_BOUNDARY` (composed by the
agent, owed ratify-or-amend per dc-vzvf) renders at harvest's OBSERVED vs LEASED
(empty set included), the dispatch trace, and wrap's leaseless pickup; harvest
names the channel per file (`(shell-parsed, best-effort)`) and counts the
unresolved set.

## Test results (verbatim, read raw)

```
test result: ok. 116 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.10s   (tests/basic.rs)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s    (tests/broken_pipe.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s    (tests/observed_set.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s    (tests/store_lint.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s    (tests/surface_lint.rs)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s    (tests/worktree_proof.rs)
```

## Mechanics

- Branch `worktree-agent-a465241ed10ed0450`, commit `69014fc` ("The observed set
  stops lying: badged writes land or record raw, the shell parses, the boundary
  speaks"). Fast-forwarded at landing. teach.rs changed but no wiring/matcher
  changes — init confirmed no-op at landing.
- Graph writes under the badge: cl-qat8 (reading, diagnosis) and cl-up6s (vein
  `sight-accounting`), both supports it-bj3b. A claim→claim depends-on was refused
  by the matrix; the mention channel carries it, per the refusal's teaching.

## Agent reflections (verbatim highlights)

- **The defect reproduced itself on me, mid-fix.** I appended the new tests to
  tests/basic.rs via PowerShell Add-Content (a shell write). My own dispatch trace
  shows 7 observed files — tests/basic.rs absent, the exact incident shape, third
  file in three incidents to be tests/basic.rs, because appending to a 5000-line
  test file is precisely when agents reach for the shell. Corroborates the
  diagnosis better than anything I wrote in the claim.
- The alternation clue in do-h3z4 did the heavy diagnostic lifting — a
  dispatcher's one throwaway sentence at harvest was worth more than the code
  archaeology. Harvest-time observations are cheap to write and irreplaceable
  later.
- Doubt: `write_shapes` is a hand tokenizer over two shell dialects; the
  false-negative space is large by design. If the parser grows, it should grow
  from observed misses, not speculation.
- Design friction: badged scratchpad writes now render at harvest as
  "unresolved" — honest but possibly noisy for agents staging bodies in temp
  files; folding temp-dir paths behind a count is a one-line render change if it
  grates.

## Dispatcher's landing addendum

Fourth consecutive worktree arc, fast-forward again, dc-g5x5 clean again. The
harvest surface was itself the fix's best witness: observed-vs-leased showed
tests/basic.rs missing while the branch diff carried +195 lines in it — the
agent's Add-Content escaped the old observation layer one last time. Judgment
acts at landing: cl-up6s superseded cl-h2gx (the agent's C3-correct flag — that
claim's "observes every in-repo write" is exactly what the diagnosis refuted;
post-fix truth is tool-writes-exact, shell-best-effort, unresolved-recorded).
Owed to the user on the flag channel: `SIGHT_BOUNDARY` and the harvest/trace
strings.
