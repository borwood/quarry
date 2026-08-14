---
id: cl-3rx9
type: claim
title: '`session-kind-field`: kind is registry data — one validation point in `parse_kind`, surfaces render what they find'
v: 4
status: asserted
provenance: assistant
created: 2026-08-14T00:40:28Z
actor: claude-fable-5
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/coord.rs
  at: a2aa307a1f00
- rel: supports
  to: dc-ad8b
  at: 2
- rel: source
  to: file:src/main.rs
  at: be7523f920c9
---

registry entries carry an optional kind set at q session set --kind; coord::parse_kind is the only place the string is judged (open set: a new kind is one new arm there, never a sweep); q session list, q session resume, and the SessionStart orient render whatever kind an entry carries beside the charter, and kindless entries render exactly as before the field
