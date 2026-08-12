---
id: it-kxdu
type: item
title: 'q serve: the view rendered per request'
v: 4
status: done
provenance: assistant
created: 2026-08-12T11:37:53Z
actor: claude-fable-5
kind: feature
acceptance:
- a refresh of a long-lived tab always shows the current graph, with no regeneration act anywhere
write_set:
- src/view.rs
- src/main.rs
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/view.rs
  at: 319ee9c14967
---

Ruled dc-f79h: the view renders per request. A std-only TCP accept loop in the q binary — each GET re-runs the view render over the live store and returns the page; no new dependency, no cache, no watcher. Port flag with a sane default; q view --open may point at the served URL when the loop is up, and the baked write stays as the offline courtesy (dc-5pb3: a render, not a record). The template items it-6gj9 (unpack click-target) and it-5j34 (activity columns) land template-side and ride every request automatically.
