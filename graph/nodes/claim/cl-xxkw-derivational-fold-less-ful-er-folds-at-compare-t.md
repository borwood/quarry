---
id: cl-xxkw
type: claim
title: '`derivational-fold`: -less/-ful/-er folds at compare time under the plu…'
v: 3
status: ratified
provenance: assistant
created: 2026-08-20T10:47:07Z
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
  to: it-rddg
  at: 6
---

`derivational-fold`: -less/-ful/-er folds at compare time under the plural-fold pattern; the family boundary is a stated decision

contains_word folds the derivational suffix family -less/-ful/-er at compare time, lexicon side ONLY, under the same pattern as the plural fold (cl-rbp9, it-nuw5): the word reduces to its stems - the word itself, its plural-stripped forms (s/es), then each stem's family-stripped form, three-char stem floor throughout - and every stem is tried bare, re-pluralized (+s/+es), and re-derived (+less/+ful/+er, each also +s for the plain plural of a derived form). Either side of the compare may carry the inflection or the derivation: leases meets leaseless in both directions through the shared stem lease, watches meets watchful, dispatch meets dispatcher, watch meets watchers. Built under it-rddg.

The family boundary is stated at one point - the DERIVATIONAL_SUFFIXES const teaching in src/queries.rs: what stays unfolded (-ness, -ment, -able, -ing/-ed, prefixes like un-/re-, and the vowel-drop respellings such as lease to leaser) is a decision, not an accident; growing the family is a deliberate edit at that const, earned by a real silent miss. find_word, find's predicate, stays exact - the compare-time fold (plural plus derivational) is now the whole of the deliberate divergence between the two predicates, which amends the sole-divergence sentence of cl-rbp9's body. Pinned by lexicon_derivational_fold_compare_time in tests/basic.rs, both at the predicate and end to end through relatedness.
