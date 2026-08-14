---
id: cl-ybth
type: claim
title: '`find-tiers`: word and id hits lead, substring-only hits trail labeled loose; find''s word predicate states its divergence from the lexicon join'
v: 3
status: asserted
provenance: assistant
created: 2026-08-14T06:50:14Z
actor: claude-fable-5
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/queries.rs
  at: 254f54de795c
---

q find tiers its hits: id substring plus title and body word-boundary matches lead (body-only hits keep the matched-in-body label); substring-only hits always trail as the loose tail, labeled '(loose: substring only)' - shown, never hidden, no flag. Every hit line rides atom_line (cl-89b5). The predicate find_word counts ASCII alphanumerics as the only word chars - everything else bounds, hyphens included - deliberately divergent from the lexicon join's contains_word, which keeps hyphen compounds whole for the naming register; each predicate states its rule and the divergence beside its definition in src/queries.rs. Unification considered and declined 2026-08-12 (it-hjed).
