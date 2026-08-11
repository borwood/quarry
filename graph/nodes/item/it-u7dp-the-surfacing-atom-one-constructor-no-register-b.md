---
id: it-u7dp
type: item
title: 'the surfacing atom: one constructor, no register below it'
v: 6
status: dropped
provenance: assistant
created: 2026-08-11T09:50:47Z
actor: claude
kind: debt
acceptance:
- 'lands \surface-atom\: one public constructor renders the node atom (id, title, type, status, v, kind, areas, grounding, archived); every register composes it and may append, never subtract'
- "lands \\\atom-lint\\: front.title formatted outside the surface module fails the lint at wrap or CI — the invariant is checked, never remembered"
- 'lands \carrier-replumb\: homework and query carriers hold atoms, not (id, title) pairs'
- "lands \\\find-areas\\: find hits name their areas, so foreign-domain matches arrive labeled"
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/main.rs
  at: 2d2daff8ff48
- rel: about
  to: file:src/render.rs
  at: e74fdd8e85fa
- rel: about
  to: file:src/queries.rs
  at: c85b3612113b
- rel: depends-on
  to: th-skv9
  at: 2
---

Yield of the 2026-08-11 surfacing audit (96 front.title sites across 9 files): one shared formatter, line(), private to src/main.rs; roughly 25-30 handrolled registers beside it; no register carries areas, kind, or grounding; id-less registers include cross-session arrivals, lease confirmations, the landed-uncited lint, and most refusals; queries.rs homework carriers bake the starvation into the data layer as (id, title) pairs, so no print-layer fix can retrofit the missing fields. Extracting every register into one surface module is also the first real seam out of the src/main.rs monolith (th-yzmj) and turns the invariant grep-lintable — enforcement by construction, not discipline. The atom's field list and its exception classes are th-skv9's ruling; acceptance below assumes the proposed list and adjusts when that thread rules.
