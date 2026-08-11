---
id: it-j2rk
type: item
title: 'landing leaks the stale-view class: done and release do not regenerate the page'
v: 3
status: done
provenance: assistant
created: 2026-08-11T06:05:32Z
actor: claude
kind: bug
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Third stale-view incident 2026-08-11: harvest regenerates the view while the item is still in-flight; the landing that follows (q set status=done, q release) regenerates nothing, so the page shows in-flight until the next wrap - the user read it-69mn as unlanded. Fix: the settled-status flip and release arms regenerate best-effort, same pattern as wrap and harvest. Completes what it-n3fu started.
