# dispatch report: a comma-joined `--files` value leases one dead glob (it-x4bb)

Landed: `glob-shape-floor` (cl-74qt) — one predicate, both stations, ahead of
every mutation. Arc under badge it-x4bb, 2026-08-21.

## Against the return spec

> a dispatched agent is never denied a write to a path its own brief prints as
> leased: a comma-bearing `--files` value is caught where the globs enter —
> split into its globs or refused with the repeatable flag taught, never leased
> whole — and `q reserve` inherits the same check

**Met, on the REFUSE arm.** `coord::check_glob_shapes` is the one predicate
that judges the shape of a lease's globs. It is called from two places:

- `coord::reserve`, right after the empty-globs check — so **every**
  lease-taking road inherits the floor: `q reserve`'s own station, `q dispatch`
  taking a fresh lease, and `q dispatch` falling back to an item's recorded
  write-set alike. That last road refuses beside the foreign-lease refusal and
  behaves like it.
- `ops::dispatch`, on the flag value, **ahead of everything that mutates** —
  ahead of `ops::acceptance_backstop`, ownership, the brief event, the lease,
  and the in-flight flip. A mistyped fire therefore has nothing to undo: no
  gate-refusal event, C8 unsatisfied, status untouched, no badge minted.

The refusal, live:

```
Error: --files takes ONE glob per flag and no value delimiter, so
"src/ops.rs,src/render.rs,tests/**" would be leased whole as a single pattern,
never a list: the overlap test compares static prefixes, so that lease matches
only the FIRST path in the value — every path after a comma is leased in name
only, printed as leased in the agent's brief and then DENIED to it at the write
by its own badge, mid-arc, in a seat that cannot extend a lease (it-x4bb). The
flag is REPEATABLE — pass each glob its own: --files "src/ops.rs" --files
"src/render.rs" --files "tests/**". (An item's recorded write-set, which a
dispatch falls back to, is authored the same way, one glob per act:
q set <item> write-set+="<glob>".)
```

The corrected command is derived from the offending value, not templated — the
dispatcher's next act is a paste away.

Both stations also state the shape *before* they refuse it: the `--files` arg
help on `q reserve` and `q dispatch` now says "one glob per flag, repeatable",
and both `after_help` blocks carry an example of the repeated form plus the
one-line reason a comma-joined value is refused.

## The fork, settled: REFUSE, not SPLIT

it-x4bb left the arm to build time. Refusing won on three grounds:

1. **A comma is legal in a real filename** (and inside a brace alternation this
   matcher does not read — `static_prefix` stops at `*?[`, never `{`). A silent
   split of `docs/notes,draft.md` mints exactly the two dead globs the check
   exists to prevent, in the one direction nobody would look. Splitting trades a
   loud failure for a quiet one; that is the wrong direction for this defect,
   whose whole character was degrading quietly.
2. **Cost asymmetry runs the other way from the defect.** The defect was
   expensive because it surfaced mid-arc in a seat that could not fix it. The
   refusal surfaces at the firing station, in the dispatcher's own hands, where
   the fix is a re-run of one command with nothing to undo. One meaning per
   flag, never a guess.
3. **The house register refuses and teaches** — C7, the `--steal` reason gate,
   the acceptance backstop. Splitting would have taught nothing, and the
   dispatcher would keep writing comma-joined values forever.

## Measured

`a_comma_joined_files_value_is_refused_where_the_globs_enter` (tests/basic.rs)
pins seven things:

- **the defect itself**, on `globs_overlap` directly: the joined value matches
  `src/ops.rs` and *not* `src/render.rs`, *not* `tests/basic.rs` — the two halves
  the it-rmqy arc actually lost;
- `coord::reserve` refuses, teaches `REPEATABLE`, emits the derived corrected
  command, and writes no lease;
- `ops::dispatch` refuses with the same teaching;
- the pre-mutation property: no lease, no badge, **no brief event** (C8 stays
  unsatisfied, so the refusal cannot be walked past), status unchanged;
- the write-set fallback road refuses too, with no `--files` in hand;
- the positive control: three `--files` flags lease three globs and every path
  passes the overlap test for real;
- end to end through the spawned binary: the value reaches the station unsplit,
  the process exits non-zero with the teaching on stderr, no lease anywhere
  holds a comma-bearing glob, and the corrected repeated form fires and puts all
  three globs on the badge the agent's brief renders from.

Probed against the regression it guards, each half independently:

- removing the check entirely → the `coord::reserve` refusal assert fails
  (`unwrap_err()` on an `Ok(ReserveOutcome …)`);
- removing **only** the `ops::dispatch` call site → the reserve arm still
  passes, and the no-brief-event assert fails. The second call site is not
  decoration; it is the pre-mutation property.

Suite: `test result: ok. 132 passed; 0 failed` (tests/basic.rs) plus 2 / 1 / 1 /
1 / 3 in broken_pipe, observed_set, store_lint, surface_lint, worktree_proof —
140 green, 0 failed. Release binary rebuilt; `cargo clippy --all-targets` adds no
warning on the new code.

## Diff

- `src/coord.rs` — `check_glob_shapes` (the predicate, its refusal, and the
  reasoning for refuse-over-split in the doc comment); one call in `reserve`.
- `src/ops.rs` — one call at the top of `dispatch`, ahead of every mutation.
- `src/main.rs` — `--files` arg help on both `Reserve` and `Dispatch`; the
  repeatable-flag paragraph in both `after_help` blocks; a repeated-form example
  on `q dispatch`.
- `tests/basic.rs` — the instrument.

## Graph

- **minted** cl-74qt — `glob-shape-floor`, kind vein, about ar-c7f5, sourced to
  `file:src/coord.rs` and `file:src/ops.rs` (the second call site is half the
  claim: a refactor moving it falsifies the pre-mutation property).
- **affirmed**, each genuinely re-read against the code as it now stands:
  - cl-p6aj `q-dispatch` — read `ops::dispatch` end to end; the one-act sequence
    stands, now with a shape floor in front of it.
  - cl-t7nx `reserve-backstop` — read `acceptance_backstop` and both call sites;
    still the one *fire-time acceptance* check on both stations, still ahead of
    ownership, brief-logging, and lease logic.
  - cl-kr7f `location-neutral-handoff` — read the spawn-line composition and
    `fork_banner`; the line still names no working directory.
  - cl-qm6t (measured, instrument `file:tests/basic.rs`) — the instrument file
    changed under this arc; re-read, the claim still describes what it asserts,
    and the suite ran green.
- **not affirmed, deliberately**: the other ~100 `behind` entries. They were
  already behind before this arc (their stamps name other sessions' blobs), and
  affirming what I did not read would be the dishonest restamp th-ybv9 names.

## Composed prose (the dc-vzvf flag channel)

Three new composed strings, presented for ratify-or-amend:

1. the `check_glob_shapes` refusal (quoted in full above) — `src/coord.rs`;
2. the `--files` arg-help line on `q reserve` and `q dispatch`: "Write-set globs
   for the lease, one glob per flag, repeatable" — `src/main.rs`;
3. the `after_help` paragraph on both stations teaching the repeatable flag and
   naming the refusal — `src/main.rs`.

None of them rides `framings.rs` (which sat outside this lease). They follow the
existing house shape for refusals and help text, which are composed inline at
the station — the same shape as the C7 refusal, the `--steal` gate, and the
acceptance-backstop refusals. If the register wants these in `framings.rs`
instead, that is a one-move relocation.

Vocabulary check against dc-ywe8 (ruled during this arc): none of the new prose
says "contract" — the dispatch-side register stays on acceptance vocabulary.

## user-owned calls:

none.

The one fork this work met — SPLIT versus REFUSE — was handed to the build by
it-x4bb's own body ("the fork to settle at build time"), and refusing
contradicts no standing ruling: the acceptance gate, C7, and the `--steal`
reason gate are all hard refusals at the same firing station, and the
never-a-gate rulings (dc-grrb, dc-dty5) govern *presence prompts* and *pressure
lines*, not malformed input. The composed prose above rides the dc-vzvf flag
channel, which is the report, not a thread.

## Reflections

**The scope I declined, and why it is worth a look.** `q set <item>
write-set+="a,b"` still accepts a comma-joined value. It cannot hurt an agent —
the fire-time floor catches it before any lease exists — but it sits latent in
the item until a `--files`-less dispatch falls back to it, and the refusal it
then draws talks about `--files` when the fix is a `q set`. I left it alone on
purpose: the load-bearing property ("no agent is ever denied its own
write-set") is fully held by the two stations it-x4bb named, and adding a new
refusal at a *shaping* station is a seat question (dc-p6z4, dc-mpg8) that this
item did not ask me to open. It is one line if the dispatcher wants it, and the
refusal message already names the authoring road. Say the word and it is a
thread.

**The item's diagnosis was one word off, and the word mattered.** it-x4bb says
the comma-joined value "mints ONE glob whose static prefix is the entire
string". Not quite: `static_prefix` stops at the first `*?[`, so for
`src/ops.rs,src/render.rs,tests/**` the prefix is
`src/ops.rs,src/render.rs,tests/` — everything up to the first wildcard. The
consequence is identical (prefix-of only matches the first path and its
directory ancestors), but I wrote the claim and the refusal against the actual
prefix rather than the item's phrasing, because a claim that mis-describes its
own matcher is fool's gold the next reader will trip on.

**`static_prefix` does not read `{`.** Brace alternation — `src/{ops,render}.rs`
— is not supported by this matcher and never has been: the whole braced string
becomes the static prefix, so it matches nothing real. It is not a defect this
arc introduced or should fix, but it is the reason the SPLIT arm was riskier
than it looks, and it is worth someone deciding deliberately whether the lease
vocabulary should grow braces or keep refusing them by silence.

**cl-t7nx's wording now has a sibling.** It says "`ops::acceptance_backstop` is
the one fire-time check". It is still the one fire-time *acceptance* check — but
there is now a second, cheaper check ahead of it in `ops::dispatch`, and a
reader of that sentence alone might expect the backstop to be first in the
function. I affirmed cl-t7nx because the claim it actually makes is unchanged
and true; I am flagging the phrasing here rather than editing a ratified claim
under a badge. My own vein body states the ordering explicitly so the two read
together.

**The `behind` queue is 107 entries deep and I could barely see my own work in
it.** Finding which claims my diff actually touched meant grepping the query
output for ids I already suspected. th-ybv9 is exactly this and is queued; this
arc is one more instance of its evidence. Two of the four affirms I made were
findable only because I had read the source in the same hour — an agent that had
not would have either affirmed nothing or affirmed everything, and both are
worse than the truth.

**What surprised me, in a good way.** The pre-mutation ordering was not in the
acceptance line, but the probe made it worth having: removing that one call site
leaves the refusal working and the *item* dirtied — brief logged, C8 satisfied
by a fire that never happened. Small ordering choices in `ops::dispatch` are
load-bearing in a way the function's length hides, and the instrument is the
only thing that will remember.
