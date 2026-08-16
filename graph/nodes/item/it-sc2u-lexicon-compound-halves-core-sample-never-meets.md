---
id: it-sc2u
type: item
title: 'lexicon compound halves: core-sample never meets core sample'
v: 5
status: done
provenance: assistant
created: 2026-08-12T11:09:24Z
actor: claude-fable-5
kind: feature
acceptance:
- 'lands `compound-halves`: sig_tokens emits hyphen compounds whole plus halves of five-plus chars; contains_word adopts hyphen-as-boundary; compound hits outrank fragment hits'
write_set:
- src/**
- tests/**
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/queries.rs
  at: 033f0da8b572
---

sig_tokens keeps hyphen compounds whole and never emits their halves, and contains_word treats hyphens as word chars — so core-sample joins only its identical spelling: a spaced mention (core sample) or a bare fragment (sample) misses silently (census with the user 2026-08-12, ui-cleanup session). Settled middle path, not yet warranted: emit the compound whole plus its halves of five-plus chars, adopt find's hyphen-as-boundary edge rule, and score compound hits above fragment hits — names stay atoms (deep-time survives), spelling variance joins, fragment noise stays behind the score gate. Full unification with find's predicate was considered and declined 2026-08-12: splitting alone dissolves the naming register's atoms at the floors. Build when a real missed join shows up — misses are silent today, so evidence arrives by hand. Re-grounds cl-en99 like any matcher change.

User-ruled 2026-08-15 (dc-qvtz sitting): the settled middle path is confirmed - build it. Emit the compound whole plus halves of five-plus chars, adopt find's hyphen-as-boundary edge rule, score compound hits above fragment hits - names stay atoms, spelling variance joins, fragment noise stays behind the score gate. Re-grounding of cl-en99 happens once, after all three trio changes land. Ships as one of three dispatches carried by one agent.
