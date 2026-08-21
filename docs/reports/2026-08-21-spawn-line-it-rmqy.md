# Dispatch report — it-rmqy: the spawn line misdirects the fork

*Partial / stop-report. Registered by the dispatcher on the agent's behalf:
the arc stopped at its write-set boundary before registering its own return.
The words below are the agent's.*

## Outcomes against the RETURN SPEC

**1. "the spawn line printed at dispatch names no working directory" — MET.**
`ops::dispatch` now composes:

```
You are dispatched: run: q join {token} --store {root} — then follow what it prints.
```

The `in {root},` clause is gone; the `--store` pin stays (dc-g5x5, cl-aujk
untouched). The line is the join command and nothing else, correct followed
verbatim from the canonical tree and from a fork alike. `ops.rs` is the only
composition point in the repo — grep for `You are dispatched` / `in {root}`
finds nothing else outside two prior report docs and the item's own body. The
existing marker `"run: q join "` that `tests/worktree_proof.rs:177`,
`tests/observed_set.rs:111` and `tests/basic.rs:1646` key on survives
unchanged.

**2. "a join whose working checkout differs from the store root opens the
brief by saying so" — MET, but landed in `src/ops.rs`, not `src/render.rs`.**

New `ops::fork_banner(store) -> Option<String>` returns the WHERE-YOU-STAND
line when `store.work_root != store.root`; `ops::join` prepends it to the
rendered brief, so the brief a fork join prints opens with it. It states both
halves the acceptance names — work where you stand (read, edit, build, test in
the fork; never cd to canon to run a verb, which would take file work out of
isolation and hash content the agent never wrote) and the graph half landing
canonically (q acts, blob stamps, hook-observed writes, fork `graph/` inert).

Why `ops.rs` and not `render.rs`: the lease denied `render.rs` (see below),
and the placement is defensible on its own — the brief's second line promises
everything below it is *derived from the graph at render time*, and the fork
fact is read from the process's cwd, not the graph. Cost of the choice:
`q brief <item>` re-read from a fork does not carry the banner. If you want it
there, the function is `pub` and one call in `render::brief` picks it up.

**Verification** (the instrument I could not write, run by hand): a throwaway
canon store + real `git worktree` fork, dispatch then join from each side,
through `target/release/q.exe`. Dispatch printed the location-neutral line;
join with cwd in the fork opened with WHERE YOU STAND; join from the canonical
root and from a *subdirectory* of it printed no banner (no false positive from
walk-up discovery). Full suite green after the change:
`test result: ok. 129 passed; 0 failed` (basic), `2 passed` (broken_pipe),
`1 passed` (observed_set), `1 passed` (store_lint), `1 passed` (surface_lint),
`2 passed` (worktree_proof), 0 doc-tests. `cargo check --release --all-targets`
clean, no warnings.

## What blocked the rest

**The lease for this dispatch held one dead glob.** `--files` was passed as a
single comma-joined value, so the lease was `["src/ops.rs,src/render.rs,tests/**"]`
— one glob, not three. `coord::globs_overlap` compares static prefixes, so it
matched only the first path. Probed against the real guard:

| path | verdict |
|---|---|
| `src/ops.rs` | allow |
| `src/render.rs` | **deny** — "outside the leased write-set" |
| `tests/basic.rs` | **deny** |
| `src/main.rs` | deny (correct at the time) |

So this arc could not write `src/render.rs` or any test. The guard was not
routed around. Consequences carried forward:

- **No instrument pins either outcome.** The test belongs in
  `tests/worktree_proof.rs`, which already spawns the real binary with cwd in a
  fork — two asserts (spawn line names no directory; the fork join's brief
  carries the banner, the canonical join's does not) drop straight into
  `the_store_pin_carries_a_worktree_dispatch_end_to_end` at its existing join
  step, ~line 204.
- **`src/main.rs:2077-2083` is now redundant** and prints immediately *before*
  the banner, so a fork join says the same thing twice in adjacent paragraphs.
  It fires only on the Join arm, which always renders the brief, so the whole
  `if store.work_root != store.root { ... }` block is subsumed by the banner
  and should be deleted.

**Filed as a defect: it-x4bb** — a comma-joined `--files` value leases one dead
glob: the dispatched agent is denied its own write-set. Silent at the firing
station, surfaces mid-arc at the agent's first denied write, and unfixable from
the agent seat (extending a lease is release + re-reserve, which the actor
rules forbid).

## Graph writes under this badge

- **cl-kr7f** — vein, `` `location-neutral-handoff` ``: the spawn line names no
  working directory; join reads the real cwd and opens the brief with
  where-you-stand. Sourced `file:src/ops.rs`, about `ar-c7f5`, `supports` →
  it-rmqy. Body carries the rejected alternative (a worktree flag on dispatch),
  the render-vs-join placement rationale, and the missing-instrument gap.
- **it-x4bb** — the defect above.
- **q affirm cl-6ctr --to file:src/ops.rs** — the only claim this edit pushed
  behind. Re-read against the change: `q-join` still describes reality.
- **Not affirmed, deliberately:** the other thirteen `file:src/ops.rs` stamps
  were already behind before this arc touched the file, part of a repo-wide
  drift backlog. Affirming those would claim review of changes never read.

## user-owned calls

none.

## Reflections

- **The composed register**, flagged per dc-vzvf for ratify-or-amend: one new
  string, the `WHERE YOU STAND:` banner in `ops::fork_banner`, plus the
  reworded spawn line. It sits in `ops.rs` rather than `framings.rs` because
  `framings.rs` was outside the lease — a placement not chosen freely, worth a
  look at harvest. The brief's other structural lines are composed inline in
  `render.rs` too, so inline composition is at least house-consistent;
  framings holds the *ratified species prose*, and this is not that.
- **The irony is instructive**: an arc fixing a hand-off that misdirects
  worktree agents was itself half-crippled by a different silent flaw in the
  same hand-off. Both failures share a shape — the dispatch station accepts
  something unusable without looking, and the agent discovers it far downstream
  in a seat that cannot repair it. If there is a general move here, it is that
  `q dispatch` should validate its own hand-off artifacts (globs that can
  match, a store that exists) at the firing station, where the dispatcher can
  still fix them cheaply.
- **The duplicate paragraph bothers me.** The acceptance was met and the output
  made slightly worse in the same act, because the half that would have made it
  clean is `main.rs`. Please do not accept the banner without removing that
  block.
- **Doubt on placement, honestly held**: the item's own file edges name
  `src/render.rs`, which suggests the shaping conversation expected the line in
  the brief renderer. The `ops.rs` argument (the brief is graph-derived; cwd is
  not graph) is real but was found *after* the lease forced the hand. It holds
  up on re-reading, but judge it fresh.
