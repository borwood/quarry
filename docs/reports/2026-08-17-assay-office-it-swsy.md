# Dispatch report — it-swsy: the assay office

Footprint: exactly the write-set plus q-verb graph writes; view regenerated.

**RETURN spec — lands `ratify-at-harvest`: MET.**

**The landing act ratifies.** `ops::ratify_landing` (src/ops.rs) fires from the `q set <item> status=done` handler. A dispatched item ratifies the vein and feature claims minted under its badge, read off the log's dispatch stamps so the trace survives release; the ratifier is stamped in `front.ratified {by, date}` and the act logs `op: ratify` with `assay: harvest|solo` and the landing item as cause — a fallen claim shows its assayer. A never-dispatched item landing under the session that worked it self-ratifies that session's own mints across the arc window (lease `since`, else the last in-flight flip). One mind stays legible by construction: minter actor vs. ratifier, plus the log's assay tag.

**Load is display.** `queries::weight_held` counts a claim's distinct standing supports targets; the Atom carries `weight` and every register renders `· holds N` (silence at zero). `q query load` (alias `unassayed`) warns heaviest-first under the ratified fool's-gold header.

**Every ruled surface teaches, verbatim.** Harvest line + solo variant (`framings::assay_harvest_line/assay_solo_line`), claim help (`q claim --help`), warning query header, and the VEINS framing assay sentence (src/framings.rs + docs/brief-framings.md, stale parenthetical removed, do-g8r4 re-affirmed). All five pinned by exact-match test assertions so drift fails the suite.

**Standing claims untouched** — least-committal held; the retroactive pass was not run (available on ask).

**Verified:** `test result: ok. 87 passed; 0 failed` (tests/basic.rs, three new tests: harvest-ratify, solo-self-ratify, weight/load-query), surface lint `1 passed; 0 failed`, release build clean, live smoke: `q query load`, `q claim --help`, area-open shelf all speak correctly.

**Graph acts under the badge:** veins cl-az38 (`ratify-at-harvest`), cl-hv3m (`weight-held`); feature receipt cl-2cam (`assay-office`); supports edges from both veins to the receipt; affirm on do-g8r4; view regenerated.

## Provisional calls (flagged for harvest judgment)

1. The landing act is the done-flip — `q harvest` names the assay, the flip performs it (harvest's never-transitions doctrine held).
2. **No version bump on ratification** (affirm-no-bump rationale, archive precedent) so citers never go behind over good news — the call the agent most wants reviewed, since every other status flip bumps.
3. Only kind vein|feature ratify — kindless badge mints stay asserted (it-pgn9 names that gap).
4. Any dispatch trace makes the arc harvest-path; a dispatched arc with zero badge-stamped mints ratifies nothing — no session fallback, mis-attribution beats nothing.
5. Query typed name `load`/`unassayed` (the decision's long name is the header, not the command).
6. Composed mechanical strings, not ratified: `· holds N` mark, the load query's empty-case line, the claim bullets under the assay lines — th-4w3a's ratify-or-amend can absorb them.

## Reflections

The pleasing part: when it-swsy lands with `q set it-swsy status=done`, the assay office ratifies its own three mints by the dispatcher's hand — the mechanism demonstrates itself at its own landing. Doubts: solo attribution rests on session-stamped log events, so a solo worker without QUARRY_SESSION ratifies nothing, silently — the silence-when-empty pattern felt right but a teaching line there is arguable. The weight mark now rides every atom_line register (find included), not just shelves — one construction point per dc-nnf5, but if it reads noisy the trim is one line in surface.rs. And it-tanf's multi-join stamping wart flows into assayable: bundle arcs will mis-attribute mints until that lands. Design friction worth naming: "ratify at harvest" vs. "harvest never transitions" was a real tension in the ruling's own vocabulary; resolved by letting harvest name and the landing perform — if the user meant the harvest verb itself to flip claims, that's an amend, not a rebuild.

---
Harvested by the dispatcher 2026-08-17: suite re-run and read raw (87 passed, 0 failed), all five assay lines verified verbatim against the item's record, the doc's stale parenthetical retirement reviewed. **Observed-set miss reproduced (it-bj3b, second instance):** the harvest's observed list omitted tests/basic.rs while the arc's diff shows +240 lines there — the badged tests write escaped observation again; the diff, not the observed set, was the judging evidence. Landed done; the landing act ratified the arc's three mints (the self-demonstration held).
