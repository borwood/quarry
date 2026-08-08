---
id: it-f8qc
type: item
title: 'hook-bound session identity: env injection via PreToolUse'
v: 3
status: done
provenance: assistant
created: 2026-08-08T21:34:15Z
actor: claude-fable-5
kind: feature
archived: true
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: it-ubqe
  at: 2
---

User-proposed (clobber lineage): a Claude chat session id becomes associated with a q session; the PreToolUse hook injects QUARRY_SESSION into Bash/PowerShell commands so agents never think about it. Design depends on verified hook capabilities (the spike). Candidate binding flow: agent runs q session adopt <name> writing an adopt-request; the next PreToolUse hook (which knows the chat session_id) consumes it and binds.
