---
id: cl-pmm3
type: claim
title: '`match-tiers`: backdrop sections tier by distinct shared terms; adjacency multiplies a lexical match'
v: 3
status: asserted
provenance: assistant
created: 2026-08-17T08:31:05Z
actor: claude-fable-5
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/queries.rs
  at: fc5971117294
---

Every backdrop section — claims per species, decisions, threads, docs — scores each entry by distinct shared terms: title lexicon plus backticked names, read off title, body, and acceptance on both sides through the trio-repaired contains_word. 0 terms folds into a counted remainder with its reach-them hint; 1-2 renders the atom line plus only the body lines carrying a match, elisions marked [...], the dig-in command closing; 3+ renders the full body; a shared backticked capability name is an automatic full body. The adjacency bonus adds +1 per depends-on, builds-on, or supports edge between the candidate and the item or its one-edge neighborhood (dc-hjad), only when at least one term matched. Ordering is weight descending then alphabetical. Machinery: queries::ShelfCtx and queries::shelf_match; rendering: render::backdrop_section.
