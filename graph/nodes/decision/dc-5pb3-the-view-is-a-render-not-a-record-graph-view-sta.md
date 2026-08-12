---
id: dc-5pb3
type: decision
title: 'the view is a render, not a record: graph/view stays untracked'
v: 1
status: in-force
provenance: user
created: 2026-08-12T11:21:38Z
actor: claude-fable-5
ratified:
  by: user
  date: 2026-08-12
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

User ruling 2026-08-12 (ui-cleanup session): the rendered page graph/view/index.html is a derived artifact — no reason whatsoever to git-track it. Already the implemented reality (.gitignore carries graph/view/); this node makes it policy, not accident: any machine renders its own with q view, and no fix to a stale or missing page ever involves git. Grounds stated with the ruling: most of quarry's UI is renders of verb responses — the page is a render too, never a record.
