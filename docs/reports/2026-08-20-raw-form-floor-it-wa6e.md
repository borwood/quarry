# Dispatch report — it-wa6e: the reverse floor sees the written form

Eleventh worktree dispatch — the serial run's last feed item. Agent report
condensed faithfully; dispatcher's landing addendum follows.

## Outcomes against the RETURN SPEC — MET

- **Floors measure the raw token before folding, both passes.**
  `contains_word_floored(text, word, floor)` in src/queries.rs: a word at or
  above the floor joins exactly as `contains_word`; a shorter word joins only
  through a fold variant of floor length actually written in the text.
  `fold_variants` extracted as the one generation point shared by both
  predicates, so the join and the floor can never disagree about what folds.
  Both relatedness passes ask it at six (the reverse pass's ≥6 pre-filter on
  title tokens is gone; the forward lone-hit gate arm re-measured).
- **A five-char folded pair joins forward and reverse alike** — lease/leases
  (and lease/leaseless via the derivational family) in all four configurations.
  The reverse arm was measured failing against pre-fix code (stashed src:
  "a five-char title token meets its plural in an old body: []"), then green
  restored — the bug shape proven, not assumed.
- **Bare floors stand unchanged**: sig_tokens at five, six at both relatedness
  sites; an exact five/five pair still joins neither direction, asserted both
  ways. The floor-site sweep confirmed the only ≥6 sites were the two
  relatedness lines; shelf_match untouched.
- **Lands nothing new**: extends the existing predicate family in place.

## Test results (verbatim, read raw)

```
test result: ok. 129 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.55s   (tests/basic.rs)
test result: ok. 2 passed; 0 failed (broken_pipe) · 1 passed (observed_set) · 1 passed (store_lint) · 1 passed (surface_lint) · 2 passed (worktree_proof) — all 0 failed
```

Bug-shape probe (pre-fix code): `test result: FAILED. 0 passed; 1 failed`.

## Mechanics

- Branch `worktree-agent-a790da9e3a97c7ff3`, commit `8d79cf5` ("The reverse
  floor sees the written form: five-char pairs join both ways"), 2 files,
  +138/−9. Fast-forwarded at landing. No teach.rs edit, no regeneration owed.
- Graph acts under the badge: vein cl-rs2r (`raw-form-floor`, source
  file:src/queries.rs), supports it-wa6e; affirms on cl-rbp9, cl-xxkw, cl-2dj3,
  cl-baxa, cl-en99 (all re-verified through the restructure).
- In-scope judgment calls flagged: `contains_word_floored`/`fold_variants` API
  surface (same precedent as it-rddg's widening — accepted); the forward gate
  arm change rides the fix's own "in both passes" wording — without it,
  symmetry would break the other way.

## user-owned calls:

none.

## Agent reflections (verbatim highlights)

- The brief's fix sentence took real excavation: the blind configuration (the
  five-char member on the title side, inflection written in the body) was
  disambiguated only by a line in the trio report (do-2kw8). If the item body
  had named the four configurations, an hour of archaeology would have been a
  minute of reading — the graph held the answer, but in a report, not the item.
- Bug-shape replay hit a structural snag: a test exercising new API cannot run
  against pre-fix code; proving the e2e arms bug-shaped required temporarily
  removing the predicate arms. Repair-pinning tests might separate e2e arms
  (old-API-only) from predicate arms for exactly this replay.
- Doubt, stated: the forward gate now admits lone five-char hits whose
  inflected form is written on the new side — symmetric and additive, but if
  fragment noise shows in live touches-lines, the -er family composed with this
  arm is where to look first.
- Worktree friction: essentially none; the 87KB brief still took paged reads —
  a fork-side floor render remains worth a thought (third sighting).

## Dispatcher's landing addendum

Eleventh consecutive worktree arc, fast-forward again, dc-g5x5 clean again.
The feed is dry: this landing closes the serial run — eleven dispatches, eleven
fast-forward merges, zero graph-side collection steps needed, every arc's acts
landed at the canonical store mid-flight. Run-level evidence lives across the
eleven report docs; the recurring seams (brief size, scratch-store hygiene,
behind-parallax, the regeneration seam) are each named in at least two of them.
