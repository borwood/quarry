---
id: it-69mn
type: item
title: 'fast follows: status labels, mentions section, boundary-verb guard, wrap view regen, q unlink'
v: 14
status: done
provenance: assistant
created: 2026-08-11T05:04:56Z
actor: claude
kind: feature
acceptance:
- unpack labels carry live status everywhere bodies render (open, brief, view), test-covered
- open and the view show a derived mentions-outbound section parallel to mentioned-by, both labeled
- wrap and session resume/retire refuse under an active badge (env or state file) with a teaching error naming q harvest, test-covered
- wrap regenerates the view page at close; the stale-view class dies
- 'lands `q-unlink`: edge retirement as a logged act, no version bumps, frontmatter updated; verified live by retiring the dc-wcyc builds-on dc-wwnk mislink'
- cargo test passes with the raw test result line read unfiltered
write_set:
- src/**
- tests/**
- .claude/**
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
- rel: depends-on
  to: dc-wcyc
  at: 2
---

User fast-follows plus fix-on-notice gaps, one arc under dc-wcyc (gaps fix on notice). (1) Unpack labels carry STATUS always, not only dead states: id [type status: `title`] wherever bodies render (open, brief, view). (2) The view and q open grow a "mentions →" section on the mentioning node, paralleling "← mentioned by" on the mentioned — derived, labeled, both surfaces. (3) The work of it-ymsj: boundary verbs (wrap, session resume, session retire) refuse under an active dispatch badge — env var or machine-local dispatch state — with a teaching error naming the incident class and q harvest; C8 logic applied to boundary acts. (4) The work of it-n3fu: wrap regenerates the view page at its close from the already-loaded graph (harvest too if it costs nothing extra). (5) Edge retirement, designed but never built (DESIGN.md § 5: edge mutations affirm/retire are log events): q unlink <src> <rel> <dst> retires an edge as a logged act — no version bump on either node (the affirm rationale), the edge leaves the frontmatter, the log records who and why (--note). Live verification target: retire dc-wcyc builds-on dc-wwnk, a mislink minted 2026-08-11. Landing this item also lands it-ymsj and it-n3fu (batch arc, one lease — note it in their close).
