---
id: th-8sdq
type: thread
title: 'the channel back: host-repo sessions noticing quarry gaps'
v: 2
status: queued
provenance: assistant
created: 2026-08-11T05:04:16Z
actor: claude
edges:
- rel: about
  to: ar-xa38
  at: 1
---

Raised by the user 2026-08-11 alongside the fix-on-notice ruling (dc-wcyc): once quarry runs in host repos (the deepcraft salvage first), gaps an agent notices there have no path into quarry's own graph — the noticing session's context dies with it, and some gaps are silent precisely in sessions that are not working on quarry's codebase. Cross-session machinery exists within one repo (file an item into the other purview); nothing crosses repos. Candidate shapes to weigh: a field-reports area in the quarry repo plus a taught convention that host-repo agents file items through the quarry checkout when the tool misbehaves; a dedicated verb (q gap?) writing into a quarry-repo path configured at init; wrap/harvest prompts in host repos asking about tool friction as they close. Needs the user's pick on ergonomics and how much machinery the first cut earns.
