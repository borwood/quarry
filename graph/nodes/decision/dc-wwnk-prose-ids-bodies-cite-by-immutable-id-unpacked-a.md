---
id: dc-wwnk
type: decision
title: 'prose ids: bodies cite by immutable id, unpacked at render; mentions derive, never store'
v: 2
status: in-force
provenance: user
created: 2026-08-11T04:26:28Z
actor: claude
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
- rel: builds-on
  to: dc-w9vx
  at: 1
---

Ratified 2026-08-11 (user floated the feature; assistant shape confirmed by the user). Node ids are legal citations in NODE BODIES; conversation with the user keeps titles — the house rule splits, not flips. Rationale: titles are mutable labels, ids immutable anchors. A title reference orphans silently on retitle; an id reference never rots, and render-time unpacking always shows the current title — the blob-stamp design applied to prose. Revives DESIGN.md § 3's unbuilt wikilink provision, with ids instead of slugs. The machinery is DERIVED, never stored: (1) `id-unpack` — every rendered body (open, brief, the view) expands bare ids inline as id [type: `title`], hyperlinked in the UI, dead targets labeled. (2) `mention-index` — a regex scan for the closed id shape ({ar,it,th,dc,cl,do}-xxxx; code spans skipped) derived at index time; open and the view show "mentioned by" backlinks labeled as derived; blast and behind stay real-edge-only (a mention references; it does not lean). (3) `mention-surfaces` — mint and edit echo each resolved id's title beside it (catching the wrong-but-real id, the hallucination case a dangling warning cannot), warn quietly on id-shapes resolving to nothing (never a gate — hyphenated prose false-positives like th-read exist), and offer a real-edge upgrade as judgment, never an auto-link. No mentions rel (the reverse channel derives); no auto work items from danglers (wrap lints them). Upgrading a load-bearing mention to a real edge is the recorded act — extraction-on-citation applied to references.
