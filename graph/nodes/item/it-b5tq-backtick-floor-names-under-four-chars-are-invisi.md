---
id: it-b5tq
type: item
title: 'backtick floor: names under four chars are invisible to the lexicon joins'
v: 8
status: done
provenance: assistant
created: 2026-08-11T05:25:26Z
actor: claude
kind: bug
acceptance:
- 'lands `backtick-floor`: backticked spans join from two characters (2-60 filter); bare-token floors stand as the noise gate - cli joins backticked, bare short prose stays below'
write_set:
- src/**
- tests/**
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-qvtz
  at: 3
---

backticked_spans inherits the 4-60 char filter, so a capability named in under four characters never joins relatedness or intent-delta (do-hvmd) - a silent floor, unlikely in this register.

Wider than backticks (census with the user 2026-08-12, ui-cleanup session): sig_tokens drops tokens under five chars and the reverse pass drops under six, so the graph's shortest core vocabulary — wrap, view, find, cli — never joins relatedness from either direction; cli at three chars is invisible even backticked. Same silent-floor species, one lever. Sibling levers from the same census: it-nuw5 (plural-folding), it-sc2u (compound halves).

User-ruled 2026-08-15 under dc-qvtz: the backticked floor drops to two (2-60); the bare-token floors stand as the noise gate. Build shape: backticked_spans filter 2-60, no change to sig_tokens or the reverse pass - cli joins backticked from titles and bodies, bare short prose stays below the floor. Re-grounds cl-en99 like any matcher change - one re-grounding pass after all three trio changes land, not per-item. Ships as one of three dispatches carried by one agent.
