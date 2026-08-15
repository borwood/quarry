---
id: it-6ydd
type: item
title: 'graph read hook: direct reads of graph/ deny with teaching toward q'
v: 2
status: sketch
provenance: assistant
created: 2026-08-15T06:46:07Z
actor: claude-fable-5
kind: slice
acceptance:
- 'lands `read-hook`: direct Read/Grep/Glob of graph/ files denies with authored teaching naming the q verbs; q init --claude wires it beside the write-deny'
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-pwyd
  at: 2
---

Lands dc-pwyd's raw-read clause: a PreToolUse hook on the read tools scoped to graph/** denies with teaching text naming the replacement verbs (q open, q find, q query) - the read-side twin of the existing write-deny hook, wired by q init --claude from teach.rs. After teach.rs changes: rebuild, re-run q init --claude. Enforcement beats doctrine (cl-eb8j): the bypass habit observed in the payload experiment dies at the hook, not in prose. Subagents inherit the hook with the repo.
