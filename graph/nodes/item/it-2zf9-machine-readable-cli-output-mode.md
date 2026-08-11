---
id: it-2zf9
type: item
title: machine-readable CLI output mode
v: 2
status: sketch
provenance: assistant
created: 2026-08-08T22:53:39Z
actor: claude-fable-5
kind: feature
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

CLI output is title-first prose by ruling (agents speak titles to humans). Scripts parsing it broke once already (the awk id-extraction incident). A --porcelain or --json flag on queries and verbs would give automation a stable surface without touching the human/agent one.
