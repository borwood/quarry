---
id: it-rzk8
type: item
title: 'q claim --title override: register-length spine titles without the retitle two-step'
v: 3
status: done
provenance: assistant
created: 2026-08-10T07:54:36Z
actor: claude
kind: debt
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Mined from the builds-on dispatch report (dispatch report: the builds-on edge and the intent delta query): q claim truncates titles at 72 chars, so register-length spine titles need mint-then-retitle, costing a version bump every time. Every spine-registering agent hits this. Sketch: a --title flag on q claim (text stays the body seed), or raise the truncation for name-first claim titles.
