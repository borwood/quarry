# DISPATCH REPORT — it-qmhm, "the earned lines: defects in shaping and the dry feed on the design wake"

Sanity at close: code diff additions-only, src/main.rs (+84), src/queries.rs (+84), tests/basic.rs (+223), entirely inside the leased write-set. Tests read raw: basic.rs `test result: ok. 101 passed; 0 failed` (98 pre-existing + 3 new), store_lint 1, surface_lint 1, worktree_proof 2. Release rebuilt; the live `q query defects` run surfaced 13 real defects sitting in shaping — the exact stall class dc-dty5 was ruled against.

## Against the return spec, by outcome

**Lands `defect-count` — met.** Both design wake surfaces (q session resume ~main.rs:1540, SessionStart orient ~1207) count kind=bug items in shaping under the `shape.owed_threads` gate, beside the sibling waiter lines: "defects in shaping: N bug(s) … defects fix with urgency, the standing ruling waiters them (q query defects)". Zero renders nothing. The query in hand is new: `q query defects` (alias `bugs`) lists every live bug, shaping stratum leading, oldest first within stratum, closing with the dc-ygzz teaching line. Mechanism: `queries::defects` and `queries::in_shaping`. Measured by `design_wake_counts_defects_in_shaping`: the shaped bug counts, the ready bug and the plain shaped item don't, dispatch wake omits the line, and landing the bug silences it.

**Lands `dry-feed` — met.** When ready is empty while shaping holds work, both surfaces draw: "the feed is dry: nothing in ready while N item(s) shape — the feed itself waiters the shaping pool; top promotion candidates, ranked derived (q query shaping)" followed by up to three candidate atom lines. Any item in ready and the line is absent entirely. Ranking is fully derived in `queries::promotion_candidates`, no stored priority: defects first, then feed distance (shaped-with-acceptance 0, shaped 1, sketch 2), then derived holds weight (`queries::item_weight` — count of live, unsettled nodes standing on the item via depends-on), age as tiebreak only, id as determinism floor. Measured by `promotion_ranking_is_fully_derived` (exact six-item order across all four keys) and `design_wake_states_the_dry_feed_and_any_ready_item_silences_it`.

**Holds the silence — met.** Additions-only inside the two owed_threads blocks, the Query enum/handler, and queries.rs. No aging alarms anywhere; wrap untouched; orientation order unchanged; dispatch wake and generic brief untouched.

## Graph writes

Veins cl-mdd7 (`defect-count`) and cl-wz9z (`dry-feed`), feature receipt cl-mv8t (`earned-lines`); both veins support cl-mv8t, each with source blob stamps on both mechanism files.

## Provisional calls, flagged

1. "Any item in ready" read as the **derived feed** (`queries::ready` — unblocked ready items); a blocked status=ready item does not silence the hunger. dc-dty5 says the feed is the waiter, so feed = what a dispatcher can actually pull.
2. **Resume purview-scopes both earned lines; orient stays global** — the asymmetry the sibling waiter lines already carry (cl-2sk9/cl-2zna precedent).
3. `q query defects` lists **all live bugs** (ready/in-flight trailing, labeled), slightly broader than the counted stratum — mirroring awaiting-acceptance.
4. "The derived holds weight" for **items**: items carry no outgoing supports edges, so it derives as the reverse-lean count — live unsettled depends-on dependents. A reading of the ruling's phrase, not a settled definition.
5. Candidates cap at **three** on the wake; the full pool size prints in the line and q query shaping is in hand.

## Reflections

Two honest doubts: item weight (call 4) is the one invented definition — the ruling's "derived holds count" has an exact meaning only for claims; if design meant something else, the comparator is one closure to change. Arc-kind and blocked items stay in the candidate pool — the spec was silent; an arc "never fires itself" (dc-yd9s), so a shaping arc raised as a promotion candidate would read wrong; a one-line filter if it happens live; spec-fidelity chosen over anticipation. Surprise: the fresh query found **thirteen** defects sitting in shaping — the new wake line will not be quiet on this repo, presumably the point. Friction: the release build was blocked by the long-lived `q serve` process holding the exe — stopped, rebuilt, relaunched detached; "rebuild release" and "a user-owned server holds the binary" collide every time and might deserve a line in the build protocol doc (the standing it-6kvu debt names this). Vocabulary: `acceptance` (never `contract`) kept on every new surface per the dc-dty5 dogear.

---
Harvested by the dispatcher 2026-08-18: observed-vs-leased exact (3 files), all four suites re-run and read raw (101+1+1+2 passed, 0 failed). The serve/exe-lock collision recurred for the third time this arc-series — it-6kvu's case is now well-fed; a protocol-doc line is the cheap palliative until it lands. The item-weight invention (call 4) is flagged forward: it feeds ranking only, no stored state, one closure to re-aim if design rules a different meaning. Landed done; three mints ratified.
