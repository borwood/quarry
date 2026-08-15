---
id: th-xvv6
type: thread
title: 'the liveness echo: subagent acts heartbeat the dispatching session'
v: 2
status: queued
provenance: assistant
created: 2026-08-14T06:42:40Z
actor: claude-fable-5
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
---

Filed 2026-08-14 from the it-hapc dispatch report: a dispatched agent's q acts run under the env-injected identity of the dispatching chat, so its every verb heartbeats that session - a design session with agents in flight reads "live" to fire-time routing (cl-9v7c), an echo of its own dispatches. Harmless today: routing only tests dispatch-kind liveness, and design sessions are not routing targets. It matters the moment any consumer keys on design-session liveness, or if heartbeat ever needs to mean "chat open" rather than "verbs recently ran" (the open-but-quiet dispatcher already reads asleep for the same reason; DISPATCH_LIVE_SECS compensates and printed ages let the user overrule). Candidate shapes when pulled: a heartbeat channel of its own, or badge-stamped acts not touching the holding session.

User direction 2026-08-14 (shelf ruling conversation, dc-xfgz): fix it - it is not that serious, and we will NOT create debt by disclosing the side-effect in brief prose. The fix path, not documentation, resolves this thread.
