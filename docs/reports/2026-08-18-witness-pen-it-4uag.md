# REPORT — it-4uag: the witness pen built

Diff clean against the lease: file writes exactly src/* (7 files) and tests/basic.rs; every graph/ change went through q verbs.

## Against the RETURN spec

**Outcome 1 — lands `witness-pen`: DELIVERED, all three by construction.**
- `coord::witness_seat` (src/coord.rs) is the one predicate point: a session with a *registered* kind other than `design` holds the witness pen; it carries session, kind, and the acting identity key.
- **Register-name refusal**: `ops::witness_line_check` fires at both authoring stations — `q new --acceptance` and `q set acceptance+=` — refusing any backticked span in a witness-authored line, teaching the plea channel.
- **Sequence refusal**: `ops::acceptance_backstop` now logs a `gate-refusal` event before either bail; `ops::witness_sequence_check` refuses a witness seat's `acceptance+=` on any item where that seat's session or key met the gate — the it-hapc class. The design seat's author-after-refusal flow stays free.
- **Author is never executor**: `ops::witness_execution_check` refuses at `q join` (key comparison only, *before* the single-use token is spent — a refused join leaves the token live, proven by test) and at `q reserve` (key and session comparison). Dispatching the item stays legal: author fires, a fresh mind executes. Authorship does not wash off with ratification.
- Pinned by `witness_pen_refuses_register_names_and_marks_the_rest`, `witness_pen_refuses_the_self_authorization_sequence`, `witness_author_never_executes_by_join_or_solo`.

**Outcome 2 — lands `witness-flags`: DELIVERED; the three inaugural lines seed the channel.**
- Marks stamp at authoring: `WitnessMark` (line verbatim, author key, session, kind-at-filing, date, optional user ratification) on `Front.witness` — stored because it is a filing-time stamp the mutable registry cannot re-derive.
- The channel derives at read: `queries::witness_flags` (load-is-display) — unratified marks whose line still sits in acceptance, any status, archived out; review outlives landing.
- Both design wake surfaces count the channel beside owed threads with `q witness` in hand; the dispatch-kind wake omits it — the authoring seat is never the review surface.
- `q witness` lists the channel; `q witness <item>` shows marks; `--ratify --by user` is the only clear (the `q rule --by user` trust channel), no version bump (dc-2wes); `--mark --author-session` is the transcription road for pre-pen lines.
- **Seeded live**: it-4p7m, it-ez2y, it-kjgz each carry their inaugural line marked `by session:dispatcher`; `q witness` lists exactly those three; `q open` annotates each line `[witness-authored … under review]`.
- Pinned by `witness_flags_ride_the_design_wake_until_user_ratified`.

**Outcome 3 — lands `plea-taught`: DELIVERED.** One register of alternative — "evidence onto the thread it informs, or `q new thread` for the plea" — stated at all three pen refusals, in both amended acceptance-gate refusal arms (witness-aware so the solo authoring teach is never a trap for a witness seat), in `q guide` (new section THE WITNESS PEN — PLEAS ARE THREADS), and as a standing close on the dispatch-kind wake (a teach, not a pressure count — dc-dty5 undisturbed).

## Verification

Full suite green, raw lines read directly — basic `test result: ok. 105 passed; 0 failed` (4 new witness tests), store_lint 1, surface_lint 1, worktree_proof 2. Release rebuilt; guide, channel, and annotations verified live against this store.

## Graph writes under the badge

Feature cl-5n6m (`witness-pen`), veins cl-78yz (`witness-flags`), cl-ce58 (`plea-taught`), cl-xfkw (`witness-seat`), three supports edges, thread th-bf2q (queued), three witness-mark acts.

## Provisional call, flagged — the kindless middle

dc-mpg8 says "any session whose kind is not design"; dc-p6z4 teaches the kindless solo station the authoring command directly. Read literally, dc-mpg8 would witness-constrain every kindless, unregistered, and unbound context — killing the kindless solo flow and fresh-repo usage. The least-committal path consistent with both: only a *registered non-design kind* sits in the witness seat (the dispatcher — the ruling's actual subject — fully covered); the middle keeps the design-capable default. **th-bf2q** queues the call; widening is one arm change at `coord::witness_seat`.

## Composed strings awaiting ratify-or-amend

The three pen refusal texts, the two amended gate refusal arms, the witness-ratify refusal, the wake count line, the dispatch-wake plea line, the `q witness` surface strings, the `q open` annotation tags, and the guide section.

## Reflections

The kindless-middle contradiction was the real design work of this arc — the ruling's phrase and the standing gate teach genuinely cannot both be construction; queued rather than silently picked. Surprise: the executor check at join *must* ignore session identity or it bans every subagent of the dispatcher (they inherit QUARRY_SESSION); the join road (keys only) vs solo road (keys and session) asymmetry is deliberate and documented, but subtle. Known soft spot: an author can still flip its item in-flight and write leaselessly — dc-cc76 keeps leaseless writes observed-never-denied, so "solo-build" is sealed only at the reserve and join stations. Friction: no amendment road exists for acceptance lines (append-only until it-ds6b's verbs), so "user-ratified or amended" is mechanically ratify-only today; the marks are line-text-keyed and will need updating by whatever mutation verbs land. The `q witness` verb name collides softly with claim-ratification vocabulary if a `witness` claim species ever wants the word.

---
Harvested by the dispatcher 2026-08-18: observed-vs-leased exact (8 files), all four suites re-run and read raw (105+1+1+2, 0 failed), and the seeded channel verified live by the dispatcher's own `q witness` — exactly the three inaugural lines, correctly attributed to session:dispatcher. The kindless-middle plea is judged exactly right: the two rulings genuinely conflict at that seam and the thread is the correct instrument. The join-road/solo-road asymmetry reviewed and accepted — subagents inheriting the session env makes key-only comparison at join the only workable shape. Landed done; four mints ratified. Post-land: q init --claude re-run for the teach.rs change.
