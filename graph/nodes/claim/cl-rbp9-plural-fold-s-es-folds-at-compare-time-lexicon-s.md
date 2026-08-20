---
id: cl-rbp9
type: claim
title: '`plural-fold`: s/es folds at compare time, lexicon side only; find stays exact'
v: 4
status: asserted
provenance: assistant
created: 2026-08-16T03:33:12Z
actor: claude
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/queries.rs
  at: beee011cfc44
---

contains_word folds plural s/es in both directions at compare time — watches meets watch, leases meets lease, renderer meets renderers — by trying the word own +s/+es variants and stripped stems under the word-boundary scan. Lexicon side ONLY: find_word, find predicate, stays exact (it-hjed unification stays declined; the plural fold and the derivational family fold cl-xxkw records are now the two deliberate divergences between the predicates). Built under it-nuw5.
