---
id: cl-rs2r
type: claim
title: '`raw-form-floor`: the six-char floor measures the written form on either side of the fold; five-char pairs join through their inflected member'
v: 3
status: ratified
provenance: assistant
created: 2026-08-20T11:20:04Z
actor: claude
kind: vein
ratified:
  by: claude
  date: 2026-08-20
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/queries.rs
  at: beee011cfc44
- rel: supports
  to: it-wa6e
  at: 7
---

`raw-form-floor`: the six-char floor measures the written form on either side of the fold; five-char pairs join through their inflected member

contains_word_floored measures the floor on the RAW written form, never the folded stem - the settled fix of it-wa6e, the trio build's known blind spot (it-nuw5): a word at or above the floor joins exactly as contains_word; a shorter word joins only through a fold variant of floor length actually written in the text. fold_variants is the one generation point shared by both predicates, so the join and the floor can never disagree about what folds. Both relatedness passes ask the predicate at six - the reverse pass token filter (the pre-filter at six is gone; every sig token is eligible, each measured raw against each body) and the forward lone-hit gate arm - so a five-char folded pair (lease/leases, lease/leaseless) joins forward and reverse alike through its six-plus member, whichever side carries the inflection, while an exact five-char pair (lease/lease) stays below the floor: symmetry restored, no floor lowered, the bare floors of dc-qvtz stand (sig_tokens at five, the floor value at six). Extends cl-rbp9 and cl-xxkw; shelf_match carries no six-char floor and stays untouched. Pinned by lexicon_floors_measure_the_raw_token_before_folding in tests/basic.rs - predicate and end to end, both directions, with the reverse blind arm measured failing against the pre-fix code.
