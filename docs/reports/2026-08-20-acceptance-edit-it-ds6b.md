# Dispatch report — it-ds6b: the contract repairs by line

Sixth worktree dispatch (serial run). Agent report condensed faithfully;
dispatcher's landing addendum follows.

## Outcomes against the RETURN SPEC (one acceptance line, six clauses — all MET)

- **`acceptance-=` lands with no new verb**: one arm in `ops::set`'s field match.
  Replace is both fields in one call — `"acceptance-=<old>" "acceptance+=<new>"` —
  one act, one log event, one bump, resolved in field order.
- **Exact-or-unique-substring resolution**: the exact line wins outright (never
  poisoned by substringing a sibling); else a substring matching exactly one line
  resolves; zero or multiple matches refuse listing what stands / the candidates,
  stating "Nothing was removed" — never a silent no-op, nothing saved on refusal.
- **The log and echo carry the full resolved line**: the set event's fields are
  rewritten to carry the resolved line (never the typed fragment) plus a
  `removed_acceptance` array whole; the verb echoes `acceptance removed: "<full
  line>"` per removal. Who and why ride actor/session/badge stamps and `--note`.
- **Stripping a readied item's last line loudly demotes to shaped** in the same
  act: status flips, the event carries `demoted {from, to, cause}`, the verb
  prints the UN-READIED line with the dc-p6z4 teach. Replace never demotes.
- **Non-design-seat mutations ride the witness review channel**:
  authoring-by-subtraction is authoring — a witness seat's removal lands a
  `WitnessMark { removed: true }` storing the full resolved line;
  `queries::witness_flags` surfaces unratified removal marks regardless of line
  presence; both design wake surfaces count them; only `--ratify --by user`
  clears. A replacement's `+=` arm rides the existing authoring machinery. Design
  seats mutate freely, unmarked.
- **The item bumps — the contract is content**: v increments exactly once per
  act, including replace. Pinned.

## Test results (verbatim, read raw)

```
test result: ok. 121 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.27s (tests\basic.rs)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s   (tests\broken_pipe.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s   (tests\observed_set.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (tests\store_lint.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (tests\surface_lint.rs)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.79s   (tests\worktree_proof.rs)
```

Three new instruments, including a real-binary demotion read and an end-to-end
witness-removal ride through ratification.

## Mechanics

- Branch `worktree-agent-afb0fa80ed38b871e`, commit `5f2a801` ("The contract
  repairs by line: acceptance-= resolves forgiving, records faithful, demotes on
  strip"), 5 files, +498/−20. Fast-forwarded at landing. teach.rs untouched.
- Graph acts under the badge: vein cl-pq2t (`acceptance-edit`, source
  file:src/ops.rs), supports it-ds6b; about edges added for the work's real
  footprint (src/queries.rs, src/model.rs, src/main.rs); affirms on cl-t7nx,
  cl-psau, cl-88ma (bodies re-read, instruments re-run green) and cl-pq2t.

## Interpretive calls the agent flagged (C3 discipline)

1. **In-flight strips demote too** — dc-p6z4 says "ready to shaped"; the agent
   demotes in-flight as well, mirroring `acceptance_backstop`, since
   in-flight+empty would manufacture the breach state the tripwire refuses.
   Pinned in test.
2. **Removal skips the witness line check and sequence check** — the line check
   guards names entering the register; removal only takes them out, and the
   review ride covers abuse. A witness removal author bars itself from executing
   the item (`witness_execution_check` walks removal marks too).
3. **Exact duplicates**: an exact `-=` on two byte-identical lines removes the
   first — resolved content identical, no ambiguity refusal.

## Deferred to the design seat (deliberately not amended by agent or dispatcher)

- cl-78yz (`witness-flags`) and cl-5n6m (`witness-pen`) read slightly dated:
  the removal arm now sits beside "marks whose line still sits in acceptance",
  and "a future acceptance-change verb" is no longer future. Neither is
  contradicted; body amendment is the design seat's call.
- The design wake count line still says "line(s) authored from a non-design
  seat" — removal acts count there too; a pinned test constrains the string.
- Schema growth: `WitnessMark.removed` — a stored bit the brief didn't
  anticipate, forced by line-presence having been the channel's whole predicate.
  Dispatcher looked squarely as asked: judged necessary and minimal.

## Agent reflections (verbatim highlights)

- The forcing incident's trap nearly re-armed itself: minting the vein claim
  from PowerShell meant backtick-laden strings through the exact quoting
  gauntlet the incident named. Inputting register prose from a shell remains the
  sharpest edge in the workflow.
- `q link` bumps the source node; three about-links moved it-ds6b v8→v11 and put
  the agent's minutes-old supports edge behind. Homework caught it and named the
  clearing affirm — but "bookkeeping edges bump the item while unlink doesn't"
  is an asymmetry it couldn't find stated anywhere.
- The fork's `q query behind` is ~93 entries of mostly fork-geometry noise:
  claims stamped against newer canon blobs drift against the older checkout for
  files the agent never touched. A "behind, scoped to files this badge touched"
  cut would make the homework actionable from a fork. (Third sighting of
  behind-noise friction in the run's evidence pile.)
- The brief was excellent — five settled points mapping one-to-one onto code
  seams; the one gap was the witness-removal surfacing contract (hence the
  schema bit above).

## Dispatcher's landing addendum

Sixth consecutive worktree arc, fast-forward again, dc-g5x5 clean again ("zero
friction beyond the behind-noise and the absolute-path habit"). The sight
boundary rendered clean this arc — five files observed, five in the diff, no
parser debris. Evidence pile grows by one design-shaped cut: badge-scoped behind
for fork agents.
