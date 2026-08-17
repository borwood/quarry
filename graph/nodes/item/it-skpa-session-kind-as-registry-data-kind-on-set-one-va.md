---
id: it-skpa
type: item
title: 'session kind as registry data: --kind on set, one validation point, open set'
v: 5
status: done
provenance: assistant
created: 2026-08-14T00:32:04Z
actor: claude-fable-5
kind: slice
acceptance:
- 'lands `session-kind-field`: registry entries carry kind, set via q session set --kind, validated in one place, rendered by list and the wake surfaces'
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-ad8b
  at: 2
---

Lands the field dc-ad8b rules: registry entries carry kind (design | dispatch), set at q session set --kind, shown by q session list and the wake surfaces beside the charter. The set is OPEN per the ruling: kind parses and validates in ONE place, surfaces render the kind they find rather than branching two ways, and adding a third kind is a data change plus one match arm, never a sweep. First machine consumer is it-hapc (fire-time liveness derives whether a dispatch-kind session is live); it-sumw already renders charter at wake and picks up kind rendering when the field exists. Kindless registry entries stay legal and render as today.
