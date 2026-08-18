---
id: it-4q6t
type: item
title: 'commit sweep: git add graph stages other sessions uncommitted nodes'
v: 3
status: sketch
provenance: assistant
created: 2026-08-12T21:12:19Z
actor: claude
kind: bug
edges:
- rel: about
  to: ar-xa38
  at: 1
---

Observed 2026-08-12: committing dc-ydvb with git add graph swept thirteen uncommitted nodes from the live ui-cleanup session into the decisions-session commit. Two sessions share one working tree; leases guard file writes, not commit staging, and the commit-with-the-work convention has no per-session scoping. Cheap discipline exists (stage by explicit path); the structural question - whether wrap or a commit helper should derive 'my session touched these graph files' from the log - waits for the pattern to hurt again. Instance two, 2026-08-17 (dispatcher session): git add -A on the dispatcher's shaping commit swept the design session's fresh, uncommitted acceptance-gate nodes (dc-p6z4, it-33bb, plus its edits to it-ds6b, th-wxr9, th-jvwe) into a dispatcher commit minutes after they were written. Parallel main sessions are now the normal shape, so the collision window is standing, not occasional. The log already knows which session wrote each node (every event carries the session stamp) - the derive-my-touched-set helper the first instance deferred is buildable on data that exists today.
