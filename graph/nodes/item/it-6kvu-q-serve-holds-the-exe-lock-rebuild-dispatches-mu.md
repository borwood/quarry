---
id: it-6kvu
type: item
title: 'q serve holds the exe lock: rebuild dispatches must stop-build-restart the view server'
v: 1
status: sketch
provenance: assistant
created: 2026-08-16T03:36:35Z
actor: claude
kind: debt
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Two occurrences 2026-08-15 (vein rename sweep, lexicon trio): a release rebuild cannot replace q.exe while q serve runs; both agents stopped the server, built, relaunched detached, probed 200. Repeatable snag for any src-touching dispatch on this machine; candidates: serve detects a newer binary and re-execs, a build protocol rider naming the dance, or serve running a copied exe so the lock never binds the build path.
