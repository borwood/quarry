---
id: it-4q6t
type: item
title: 'commit sweep: git add graph stages other sessions uncommitted nodes'
v: 1
status: sketch
provenance: assistant
created: 2026-08-12T21:12:19Z
actor: claude
kind: watch
edges:
- rel: about
  to: ar-xa38
  at: 1
---

Observed 2026-08-12: committing dc-ydvb with git add graph swept thirteen uncommitted nodes from the live ui-cleanup session into the decisions-session commit. Two sessions share one working tree; leases guard file writes, not commit staging, and the commit-with-the-work convention has no per-session scoping. Cheap discipline exists (stage by explicit path); the structural question - whether wrap or a commit helper should derive "my session touched these graph files" from the log - waits for the pattern to hurt again.
