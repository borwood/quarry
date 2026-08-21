# Dispatch report — it-p8rp: the reconcile parses a carried report

## Outcomes against the RETURN SPEC

**"the harvest reconcile parses no report that predates the arc's dispatch: a
carried prior supports-linked doc never stands in as the arc's return, and the
await arm speaks until this arc's report registers" — MET.**

Two changes, both inside the lease:

`src/queries.rs` — new `arc_dispatched_at(log, item_id) -> Option<String>`:
the stamp on the newest `op: dispatch` event the log carries for the item.
`latest_report_doc` grows a third parameter, `since: Option<&str>`, and filters
the supports-linked report set by `created >= since` before newest-created
wins.

`src/render.rs` — the harvest call site passes
`arc_dispatched_at(&log, id).as_deref()`. `log` was already in hand there (the
badge-acts count reads it), so nothing new is loaded.

Design points worth judging:

- **Time, not authorship.** A report is honestly registered by the agent under
  its badge *or* by the dispatcher afterward (do-qcvy, the it-rmqy stop-report,
  is the second shape), so "who registered it" is no join key. What a prior
  arc's report can never be is younger than the dispatch it precedes.
- **The log, not the held entry's `since`.** Both stamp the same instant, but
  `DispatchState` dies at the clear, so a harvest re-run after landing would
  lose the bound. The log is the durable seat, and `queries.rs` already takes
  `log` for its arc derivations (`item_was_dispatched`, `session_claim_mints`).
- **Inclusive bound.** `Store::now()` zeroes nanoseconds, so the clock is
  second-resolution; a report registered inside the dispatch's own second can
  only be the arc's own. `>=`, documented at the function.
- **Degrades to unfiltered, never to empty.** An item that never flew as a
  dispatch yields `None` and filters nothing — the old behaviour, not silence.
- **The carry itself is untouched and stays.** `render::brief` still shows a
  supports-linked prior report as evidence ("a prior dispatch's report — read
  it before repeating its ground"). The edge was never wrong; only the harvest
  reconcile's reading of it was. cl-cjbb holds, as the brief said.

The unbounded pick also broke cl-cjbb's own body sentence — "newest by created
stamp wins, so a re-dispatch reconciles against the arc's own report, not its
predecessor's" — which was false whenever the new arc had not yet registered.
That is fixed by the same bound.

**Verification.** New test in `tests/basic.rs`:
`the_reconcile_never_parses_a_report_that_predates_the_arcs_dispatch`. It
registers a carried prior report (supports-linked, prose declaring
`user-owned calls: none`), backdates its node file to `2020-01-01T00:00:00Z`
on disk (a test-authored "prior" is otherwise the same second as the dispatch
it must predate), dispatches, and measures both halves: the pure pick returns
the carried report unbounded and `None` bounded; the harvest surface prints
`user_owned_await(0)` and does **not** contain
`user_owned_reconcile(Some(0), 0)`. Then the arc's own report registers and
the surface reconciles against *it*; the re-dispatch direction is measured on
explicit stamps, where the second-resolution clock cannot blur it.

Probed against the regression it guards: restoring the unbounded call
(`latest_report_doc(&all, id, None)`) fails the suite, and the failure output
is the incident verbatim — `user-owned calls: report declares 0, thread(s)
filed under this badge: 0 — reconciled` — a stranger's prose accepted as the
arc's return.

Full suite, read raw:

```
tests\basic.rs           test result: ok. 131 passed; 0 failed; 0 ignored
tests\broken_pipe.rs     test result: ok. 2 passed; 0 failed; 0 ignored
tests\observed_set.rs    test result: ok. 1 passed; 0 failed; 0 ignored
tests\store_lint.rs      test result: ok. 1 passed; 0 failed; 0 ignored
tests\surface_lint.rs    test result: ok. 1 passed; 0 failed; 0 ignored
tests\worktree_proof.rs  test result: ok. 3 passed; 0 failed; 0 ignored
Doc-tests quarry         test result: ok. 0 passed; 0 failed; 0 ignored
```

`cargo build --release` clean, no warnings.

## Graph writes under this badge

- **cl-hct3** — vein, `` `arc-bounded-return` ``: the harvest reconcile joins
  only reports registered after this dispatch; a carried prior report is
  evidence, never a return. About `ar-c7f5`, sourced `file:src/queries.rs`,
  `supports` → it-p8rp. The body carries why time and not authorship, why the
  log and not the held entry, the inclusive-bound reasoning, and the sentence
  of cl-cjbb it amends (the join key is now kind + supports edge + registration
  at or after this arc's dispatch). Following the cl-rs2r precedent, the
  extension of cl-cjbb is a mention in the body, not an edge — the vein does
  not lean on it, it corrects one of its sentences.
- **th-t4j3** — the user-owned call below, queued.
- **q affirm cl-cjbb** — the one claim this fix re-verified: its parse pair,
  its three surfaces, and its pinned tests all still describe reality, with the
  join sentence amended by cl-hct3. Re-stamped 1 ref.
- **Not affirmed, deliberately:** the other ~20 claims stamped against
  `file:src/queries.rs` / `file:src/render.rs` were already behind before this
  arc touched either file (a repo-wide drift backlog — `q query behind` shows
  102 entries, most on files this arc never opened). Affirming them would claim
  review of changes never read.

## user-owned calls

**One.**

1. **The ratified `user_owned_await` wording now carries a second meaning** —
   filed as **th-t4j3** (queued). The line says "no report file to parse yet",
   which was simply true before this fix. It now also speaks when a report doc
   *is* registered and supports-linked and visible in the brief's evidence
   section, and the reconcile is correctly refusing to parse it. Amending that
   string is the user's pen: it is ratified register prose (cl-h3r8, do-g8r4)
   on the dc-vzvf flag channel, presented for ratify-or-amend — and
   `framings.rs` was outside this arc's lease besides. **Provisional path
   taken: the string is untouched, verbatim.** The acceptance line demanded
   only that the await arm speak, and it speaks. The thread carries a proposed
   wording if the user wants it.

## Reflections

- **The fix is four lines; finding the honest join key was the work.** My first
  instinct was to key the return on the badge — the report the *agent*
  registered under this dispatch. That is wrong, and do-qcvy is the
  counterexample sitting in this repo: a dispatcher registered a stopped arc's
  report on the agent's behalf, so a badge key would have made a real return
  invisible. Time is the only property a prior arc's report cannot fake.
- **Second-resolution stamps are a real edge here, not a theoretical one.** The
  bound compares `created` against a dispatch stamp, and both come from
  `Store::now()`, which zeroes nanoseconds. Inclusive is right for the arc's
  own report, but it means a carried report registered in the *same second* as
  a re-dispatch would still slip through. Unreachable by a human dispatcher and
  reachable in a fast test — which is why the test backdates on disk rather
  than racing the clock, and why the re-dispatch direction is measured on
  explicit stamps. If sub-second stamps ever matter elsewhere, this join is one
  of the places that would sharpen.
- **The `q query behind` backlog is loud enough to hide things.** 102 entries
  "wants action", the large majority ambient drift on files nobody in this arc
  touched. I had to read the list carefully to find the two stamps my own edit
  moved. The it-rmqy report made the same observation about `ops.rs`; two arcs
  noticing the same fog is probably a signal about the surface, not about the
  arcs. Not filed — it is a shaping judgment, not a defect on sight.
- **A small doubt on the vein.** cl-hct3 states a rule that is currently
  enforced at exactly one call site, and a rule with one consumer is thin
  material for a vein. I minted it anyway because the *rule* (an arc's return
  is bounded in time by its dispatch; a carried edge is evidence, never a
  return) is the kind a future builder re-breaks — the bundle shape of th-zzqv
  will need to answer the same question for a parent item's children. If the
  assay reads it as fool's gold, the honest alternative is folding its body
  into cl-cjbb and refuting cl-hct3.
- **What I did not touch, on purpose:** `latest_report_doc` still counts
  archived report docs, and still ignores whether the report's `path` file
  exists. Both were pre-existing, neither is in the acceptance line, and
  guessing at them would have widened a bug fix into a redesign.
