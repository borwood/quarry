---
id: it-hjed
type: item
title: 'find match hygiene: substring matching admits foreign debris'
v: 6
status: done
provenance: assistant
created: 2026-08-11T10:29:40Z
actor: claude
kind: bug
acceptance:
- a three-char query like cli returns word-boundary hits first; substring-only hits trail labeled loose, never hidden
- find's word predicate states its divergence from the lexicon join beside its definition
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/main.rs
  at: a4910d6f7000
---

find matches raw substrings across title, id, and body: the query cli matched the substring inside click, admitting dc-qhru — a deepcraft-salvage ruling — into a search for the cli area (2026-08-11; the confabulation instance motivating dc-nnf5). Wants word-boundary or token matching for short queries, or a match-quality marker on the hit line.

Settled with the user 2026-08-12 (ui-cleanup session), queued for build: tiered output. Tier one is id substring plus title and body word-boundary hits (body-only hits keep the matched-in-body label); substring-only hits trail as a labeled loose tail — always shown, never hidden, no flag. Find's word predicate: word chars are ASCII alphanumerics only, everything else a boundary, hyphens included — deliberately divergent from the lexicon join's contains_word, which keeps hyphen compounds whole (load-bearing for the naming register). Unifying the two predicates was considered and declined 2026-08-12; the lexicon's real levers filed separately as it-nuw5, it-sc2u, and the widened it-b5tq. Each predicate states its rule beside its definition. Implementation rides the surfacing atom: hit lines render via atom_line (cl-89b5, dc-nnf5), so the loose label belongs in the atom's register, not a raw print.
