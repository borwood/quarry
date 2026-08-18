---
id: cl-3rx9
type: claim
title: '`session-kind-field`: kind is registry data — one validation point in `parse_kind`, surfaces render what they find'
v: 6
status: asserted
provenance: assistant
created: 2026-08-14T00:40:28Z
actor: claude-fable-5
kind: vein
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/coord.rs
  at: 5f2e8b4afc49
- rel: supports
  to: dc-ad8b
  at: 2
- rel: source
  to: file:src/main.rs
  at: 4bfea30a6071
---

registry entries carry an optional kind set at q session set --kind; coord::parse_kind is the only place the string is VALIDATED - it alone can refuse, and the open set means a new kind is one new arm there, never a sweep. Render and mapping sites (q session list, resume, the SessionStart orient, coord::wake_shape) consume whatever kind an entry carries and never judge it - a catch-all keeps unknown and absent kinds on the generic path. Kindless entries render exactly as before the field. (Clarified 2026-08-14 in the report-dormancy sweep: judged means validated; mapping what is found is not judgment - the wake_shape match added by it-wub5 does not breach the one-validation-point claim.)
