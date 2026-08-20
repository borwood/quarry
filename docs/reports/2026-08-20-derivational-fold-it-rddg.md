# Dispatch report — it-rddg: the lexicon folds the family

Ninth worktree dispatch (serial run). Agent report condensed faithfully;
dispatcher's landing addendum follows.

## Outcomes against the RETURN SPEC — MET

- **Nothing new landed**: the fold extends `contains_word` in place — the one
  compare-time predicate every lexicon join rides (relatedness both passes,
  shelf_match both directions, the brief renderer's truncated-body matching).
- **The pattern**: word reduces to stems (itself, plural-stripped s/es, then
  family-stripped -less/-ful/-er; three-char stem floor throughout); each stem
  tried bare, re-pluralized, and re-derived (each derived form also +s). Same
  variant-generation shape as the plural fold, composed one strip deep each way —
  leases and leaseless both reduce to lease.
- **Both directions, measured**: `lexicon_derivational_fold_compare_time` pins
  leases↔leaseless at the predicate plus watchful/watches, dispatcher/dispatch,
  watchers/watch, and end-to-end through relatedness both passes. Negative arms
  pin the boundary: -ness out, prefixes out, stem floor holds (user never
  collapses to "us"), find_word stays exact.
- **The boundary stated**: `DERIVATIONAL_SUFFIXES` const carries the teaching —
  what stays unfolded (-ness, -ment, -able, -ing/-ed, prefixes, vowel-drop
  respellings) is a decision until a real silent miss earns an entry at that one
  point.

## Test results (verbatim, read raw)

```
test result: ok. 126 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.92s (tests/basic.rs)
test result: ok. 2 passed; 0 failed (broken_pipe) · 1 passed (observed_set) · 1 passed (store_lint) · 1 passed (surface_lint) · 2 passed (worktree_proof) — all 0 failed
```

## Mechanics

- Branch `worktree-agent-a0876d98e2ac0bce5`, commit `2a9df06` ("The lexicon folds
  the family: leases meets leaseless, the boundary a decision"), 2 files,
  +124/−18. Fast-forwarded at landing. No init or view regeneration owed.
- Graph acts under the badge: vein cl-xxkw (`derivational-fold`, source
  file:src/queries.rs), supports it-rddg; affirms on cl-2dj3 and cl-baxa
  (re-verified exactly true); deliberate non-affirms flagged for harvest:
  cl-rbp9 and cl-ybth (below).

## user-owned calls:

none declared by the agent (the family was user-agreed per the brief). Two
in-scope judgment calls flagged: `contains_word` visibility widened
pub(crate)→pub so the instrument reads the predicate directly (API-surface
change, dispatcher glanced and accepted — sibling find_word was already pub);
the derived-form plain-plural composition ({stem}{suf}s, watch meets watchers).

## Dispatcher's judgment acts at landing

Both flagged claim bodies amended by q edit with cause-notes (assistant
provenance, body-only staleness — titles stood, so no supersession needed):

- **cl-rbp9**: "the fold is now the sole deliberate divergence" → the plural
  fold and the derivational family are the two deliberate divergences.
- **cl-ybth**: the pre-existing stale clause ("contains_word keeps hyphen
  compounds whole") — falsified by the compound-halves landing before this run
  and never flagged — restated as the folds-vs-exact divergence.

## Agent reflections (verbatim highlights)

- "Lands nothing new" earned its place: first instinct was a helper function;
  the phrase pushed the fold into the existing variant-generation shape.
- Genuine surprise: both neighboring fold veins carried divergence sentences
  that were or became stale — cl-ybth's was stale before this change and
  nothing had flagged it. An affirm-eagerly habit would have laundered the
  staleness; the catch happened mid-act.
- The -er member is the noisy one (number→numb, after→aft produce dead stems);
  the three-char floor kept every probed case inert, but -er is where to look
  first if fold debris shows in live joins.
- Worktree evidence: frictionless arc; only note — the 148KB brief took three
  paged reads; a fork-side smaller floor render might be worth thought.

## Dispatcher's landing addendum

Ninth consecutive worktree arc, fast-forward again, dc-g5x5 clean again. The
new user-owned-calls harvest line rendered on its first post-landing arc
(0 threads, await arm). The stale-divergence catch is the run's best argument
yet for the source-drift pass: cl-ybth taught a dead clause for six days and
only a neighboring diff surfaced it.
