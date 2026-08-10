# Dispatch report — it-tz94 "the builds-on edge and the intent delta query"

Authored by the dispatched build agent (claude-fable-5 subagent), 2026-08-10.
Registered at harvest by the dispatcher. First dispatch to run through `q dispatch`
end to end: full badge observation (6/6 touched files), payload-only hand-off.
Dispatcher addendum at the end.

## Outcomes (against the RETURN spec)

**1. `builds-on-edge` lands.** `q link` accepts builds-on for decision→decision, doc→claim, doc→decision; every other shape refuses with a teaching error naming the legal shapes and the matrix; edges stamp at the target's version like any rel; no status coupling anywhere.
- src/model.rs — `RELS` grows `builds-on` (line 54); new `legal_shapes()` table (lines 58–73) is the refusal message's authority; the matrix arm (lines 92–96) with the no-coupling comment; the one `bail!` now appends the rel's shapes (lines 100–113) — extended the table, no special-casing.
- Live refusal transcript: `Error: edge not allowed: item -[builds-on]-> decision (builds-on takes builder → built-upon: decision → decision, doc → claim/decision (lineage, never status)) — run q guide for the edge matrix` (exit 1).
- No-coupling is test-proven: refuting a built-upon claim leaves the builder doc `[registered]`; superseding a built-upon decision leaves its builders `[in-force]` while the supersedes coupling itself still fires (`tests/basic.rs::builds_on_matrix_shapes_stamp_and_no_status_coupling`).

**2. `builds-on-edge` staleness, test-covered.** behind reports builds-on refs generically — severity 3 on ordinary content change, severity 1 when the target is dead, exactly like any rel (zero special code; the generic edge walk covers it). blast walks reverse builds-on, transitively: src/queries.rs line ~187 adds `"builds-on"` to the inbound rels. Covered by `tests/basic.rs::builds_on_behind_and_reverse_blast` (bump→sev 3, refute→sev 1, refute's blast enumerates the spec, decision chains walk transitively). Live: `q query blast dc-grrb` now enumerates dc-cc76 and, through it, dc-kpqg.

**3. `intent-delta` lands.** `queries::intent_delta` (src/queries.rs, after `relatedness`) joins backtick-named acceptance lines to spine claim titles per shared area, both directions; backtick extraction is the new `backticked_spans()` shared with the relatedness matcher (one vocabulary). Surface: `q query intent-delta [area]` (src/main.rs); advertised at BOTH shaping output and area open, one line, only when non-empty (src/render.rs `open()` tail). Live run found the two capabilities this item's acceptance names correctly landed, and surfaced four pre-convention spines (`q-dispatch`, `q-harvest`, `touched-set-accrual`, `enforcement-beats-doctrine`) as landed-but-unintended — honest emergent scope, see reflections. Test: `intent_delta_both_directions_join_on_shared_areas` (shared-area join, foreign-area isolation, area scoping, refuted spines un-land, settled items leave unlanded but still count as intent).

**4. Teach surfaces.** GUIDE gains the matrix line (`builds-on (decision → decision, doc → claim/decision — lineage, never status)`) and a new "INTENT AND LINEAGE — NAMES ARE THE JOIN" section (src/teach.rs); `q link --help` rel list and examples updated (--help owns syntax). `q init --claude` re-run; SKILL.md lines 62–71 and 177 carry both.

**5. Retroactive lineage recorded.** `dc-cc76 -[builds-on]-> dc-grrb (at v1)` then `dc-kpqg -[builds-on]-> dc-cc76 (at v3)` — ordered so the second stamp cites the bumped version. All seven citers-behind the bumps created were affirmed after review (the sole delta was the lineage edge itself).

**6. Tests pass, read raw.** `test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s` (tests/basic.rs — 39 prior + 3 new); lib/main/doc-test binaries each `ok. 0 passed; 0 failed`. Release build clean in 15.65s; one build at a time throughout.

## Assistant-grade calls
- **Unintended direction consults items of ANY status.** A strict reading of the ruling ("acceptance lines of live items") would make every spine drift into "unintended" as its intending items settle and archive — a growing sweep, the opposite of silence-default. Landing does not un-intend. Live-items-only governs the unlanded direction (settled intent is no longer owed). Flagged for the dispatcher; the code comment records the rationale.
- **Spines registered while the diff was warm:** cl-vqdb (`builds-on-edge`, source file:src/model.rs) and cl-gu5f (`intent-delta`, source file:src/queries.rs), each `supports` its ruling (dc-jsvp, dc-je5b).
- **Housekeeping affirms:** the three sev-4 drifts the agent's own edits caused (cl-en99, cl-enu7, cl-qm6t) affirmed after review; cl-c34g → src/coord.rs drift left standing — it predates this dispatch and was the dispatcher's to review.
- Advertised at both candidate surfaces ("and/or" in the spec) since both are cheap and silent when empty.

## Reflections (the agent's own words)
- The four pre-convention spines surfacing as landed-but-unintended is correct but means the surface opens noisy on this very repo. The dispatcher has a real three-way call the query's closing line names: land acceptance names retroactively on it-7ss9 (feels like backfill theater), leave them and know why (my lean — they predate dc-je5b), or let them age out when a future convention decision addresses grandfathering.
- Friction: `q claim` truncates titles at 72 chars, so register-length spine titles need a mint-then-retitle two-step (costing a version). Every spine-registering agent will hit this; a `--title` override on claim might be worth a sketch.
- `backticked_spans` inherits the 4–60 char filter from relatedness. A capability named with ≤3 chars would be invisible to the delta. Unlikely in this register, but it is a silent floor.
- The join is strictly per shared area. An intent and its spine filed into disjoint areas never meet — the delta will report both directions instead of a match. That is per the ruling, but a misfiled spine will look like emergent scope rather than a filing error; the wrap's unfiled surface is the only backstop.
- Surprise: behind needed literally zero code for builds-on — the generic edge walk already delivered "ordinary severity" by construction. The acceptance line was satisfied by tests alone, which is the schema doing its job.

## Dispatcher addendum (at harvest, 2026-08-10)

Independent verification in the bound chat: `test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` — no environment-sensitivity recurrence. Live spot-checks of the refusal, the blast chain, and intent-delta all matched the report. Harvest observation was COMPLETE this time — the state-file badge recorded all six touched files, closing the transport gap the bootstrap dispatch documented; `.claude/**` showed untouched in observation despite the skill regenerating because `q init --claude` writes from inside the binary, not through a harness tool — a known observation boundary worth remembering (binary-internal writes are invisible to the guard). Judgment on the flagged three-way: the four pre-convention spines stay as they are — they predate the convention, and the query's honest reporting of them is the surface working; revisit only if the noise proves real. The agent's any-status call on the unintended direction is accepted as the correct reading. The claim-title truncation friction is filed as a sketch item.
