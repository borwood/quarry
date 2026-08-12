---
id: dc-f79h
type: decision
title: 'the view renders per request: no baked freshness contract'
v: 1
status: in-force
provenance: user
created: 2026-08-12T11:37:34Z
actor: claude-fable-5
ratified:
  by: user
  date: 2026-08-12
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

User ruling 2026-08-12 (ui-cleanup session): the view is rendered per request — the user keeps a long-lived tab, and a baked file lies to it between boundaries. Serve mode wraps the existing render in a request loop; no new dependency warranted. Completes dc-5pb3's tense: a render, not a record — now in time as well as in git. The boundary regens and the baked file remain as courtesy for q view --open, demoted from freshness contract to cache.
