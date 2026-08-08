---
id: dc-8mf2
type: decision
title: session identity is launcher-owned; the handoff is derived at wake
v: 2
status: in-force
provenance: user
created: 2026-08-08T21:34:15Z
actor: claude-fable-5
ratified:
  by: user
  date: 2026-08-08
edges:
- rel: about
  to: ar-xa38
  at: 1
- rel: settles
  to: th-mcwa
  at: 2
---

Identity is launcher-owned: QUARRY_SESSION is set by a per-role launcher before the chat starts (q session set --launcher emits the script); /clear inherits the identity, switching roles means relaunching. The wake flow is derived, never written: q session resume renders the handoff from the log — holdings (with release nags), in-flight items in purview, the session's own recent acts, arrivals from other sessions since the last act, and what the user is owed. The close-block ritual dissolves and the planned q handoff verb is retired. Write verbs touch a machine-local last_seen so resume can flag a double incarnation (<120s). Hook-bound identity (automatic env injection via PreToolUse, chat-id binding) is filed as an enhancement pending capability verification.
