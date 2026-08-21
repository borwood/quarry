# Dispatch report — it-rmqy, arc 2 (the spawn line misdirects the fork)

Arc 1 (report do-qcvy) met both acceptance outcomes in `src/ops.rs` but was
crippled by a dead lease glob: it could write neither a test nor `src/main.rs`.
This arc's lease (`src/ops.rs`, `src/main.rs`, `tests/**`) closed exactly those
two gaps. `src/ops.rs` needed no further change.

## Outcomes against the RETURN SPEC

**1. "the spawn line printed at dispatch names no working directory" — MET,
now pinned by an instrument.**

`tests/worktree_proof.rs::the_store_pin_carries_a_worktree_dispatch_end_to_end`
asserts it at the seat that matters (the real binary, cwd in a git worktree):
nothing stands between the announcement and `run: q join ` (no path, no
comma-clause), and stripping the single `--store <root>` occurrence leaves the
canonical root named nowhere else in the line. Probed against the regression it
guards — re-adding `in {root}, ` to `ops::dispatch` fails the suite with
`no directing clause stands between the announcement and the command`.

**2. "a join whose working checkout differs from the store root opens the brief
by saying so" — MET, and it now says it exactly once.**

The duplicate arc 1 flagged is gone: `src/main.rs` (Join arm, was lines
2077-2083) printed its own `store pinned: …` paragraph immediately before the
brief, so a fork join said the same thing twice in adjacent paragraphs.
Deleted; `ops::fork_banner` is the single statement, and it carries the file
half the old line omitted. Pinned by four asserts: the banner precedes the
brief header, it names both roots, it is the *very next non-empty line after
the bind* (nothing may stand between), and it appears once. Probed:
reinstating a second orientation paragraph fails the suite.

The negative half is a new test,
`a_canonical_join_says_nothing_about_where_it_stands` — a join from the store
root and an idempotent re-join from a *subdirectory* of it both carry no
banner, so the walk-up that finds the graph from a nested cwd never fakes a
fork.

**Verification** — full suite, read raw: `test result: ok. 129 passed; 0 failed`
(basic), `2 passed` (broken_pipe), `1 passed` (observed_set), `1 passed`
(store_lint), `1 passed` (surface_lint), `3 passed` (worktree_proof), 0
doc-tests. No compiler warnings. `cargo build --release` rebuilt afterwards, so
the on-PATH `q` matches the source.

*Dispatcher's note at landing: the suite was re-run from the judge seat before
landing and read raw — nine `test result: ok` lines, basic at 129 passed,
worktree_proof at 3 passed.*

## Graph writes under this badge

- **cl-kr7f** v3→v5 — body amended: the stale "the test … which this arc's
  lease denied" sentence replaced by what now pins it, both test names and both
  probes named; the join-arm deduplication recorded. New
  `source → file:tests/worktree_proof.rs` edge — the claim's grounding is now
  the instrument, not a by-hand run.
- **q affirm cl-nzjs --to file:tests/worktree_proof.rs** — the only claim this
  diff pushed behind. Re-read: `worktree-dispatch` still describes reality.
- **it-rzqv** (new, sketch) — see below.
- **Not affirmed, deliberately:** all twelve `file:src/main.rs` stamps were
  already behind before this arc touched the file. This edit pushed none of
  them behind; affirming would claim review of changes never read.

## Filed, not built

**it-rzqv** — *a re-rendered brief loses the fork banner: `q brief` orients no
one, and it is the command the agent is told to re-run.* The banner is composed
in `ops::join`, so a fork agent is oriented exactly once. Every re-render loses
it — and `q brief <item>` is precisely what the write-guard teaching line and
the PreToolUse dispatch reminder tell a working agent to run, i.e. what it
reaches for after a context loss, when orientation matters most. `fork_banner`
is `pub`; one call in main.rs's Brief arm closes it. Left unbuilt: it-rmqy's
acceptance names the join and the spawn line only.

## user-owned calls

**none.** One near-miss checked rather than assumed: the `store pinned: …` line
deleted here was a composed string flagged in the 2026-08-18 store-pin report
under the dc-vzvf channel. dc-vzvf ratified the register across *three*
landings — the renderer, sediment-and-rot, and the assay office — and the
store-pin landing is not among them, so the line carries no user ratification.
Removing it serves the user's 2026-08-19 ruling (B: the join opens the brief by
saying so) rather than contradicting it. Flagged here per the channel, not as a
thread.

**Composed register (dc-vzvf, ratify-or-amend):** one string *removed* (join's
`store pinned:` paragraph); no new strings composed this arc — the banner text
is arc 1's, unchanged.

## Reflections

- **Test-message discipline paid.** Both new asserts were probed against the
  exact regressions they guard before being trusted. The second probe is the one
  that would have been gotten wrong by feel: the first draft only counted
  `WHERE YOU STAND:` occurrences, which a *differently worded* duplicate
  paragraph — precisely the bug arc 1 left — would have sailed straight past.
  The assert that actually bites is positional: the banner must be the next
  thing said after the bind.
- **Arc 1's placement doubt, judged fresh as asked:** re-read the
  `ops.rs`-vs-`render.rs` argument without the lease pressure that produced it,
  and it holds. The brief's second line promises everything below it is derived
  from the graph at render time; where the process stands is not graph. The
  item's own `about → file:src/render.rs` edge records the shaping
  conversation's guess, and it is now wrong — worth a dispatcher's hand at
  harvest, either re-pointing it or leaving it as history.
- **The fork-path assert is case-folded and separator-normalized** rather than
  compared raw. A Windows path round-tripped through
  `SetCurrentDirectory`/`GetCurrentDirectory` can come back with its own idea of
  case; `store::store_relative` already takes that posture for the same reason,
  so the test is house-consistent rather than merely defensive.
- **Two arcs, one small fix, because the hand-off broke itself twice** — a dead
  lease glob the first time, a stale write-set the second (`render.rs` was
  dropped this round, which was right, but nothing told arc 1 that would
  happen). Arc 1's closing suggestion still looks correct from this seat:
  `q dispatch` should validate its own hand-off artifacts — globs that can
  match, a store that exists — at the firing station, where the dispatcher can
  still fix them for free. That is it-x4bb's neighbourhood, and it is the
  second time in a row this item has paid for it.
