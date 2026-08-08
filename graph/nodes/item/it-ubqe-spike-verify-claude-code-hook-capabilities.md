---
id: it-ubqe
type: item
title: 'spike: verify Claude Code hook capabilities'
v: 2
status: done
provenance: assistant
created: 2026-08-08T21:34:15Z
actor: claude-fable-5
kind: spike
acceptance:
- session_id availability per hook event confirmed
- PreToolUse input-modification capability confirmed or refuted
- C:/Program Files/Git/clear session_id and env behavior confirmed
archived: true
edges:
- rel: about
  to: ar-xa38
  at: 1
---

Dispatched to the claude-code-guide agent 2026-08-08: does hook stdin carry session_id (PreToolUse, SessionStart, UserPromptSubmit); can PreToolUse rewrite tool input (updatedInput schema); does /clear change session_id and does process env persist. Result decides the hook-bound identity design.
