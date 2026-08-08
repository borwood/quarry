---
id: th-v9tg
type: thread
title: mid-session cross-session alerts, hook-injected
v: 2
status: resolved
provenance: assistant
created: 2026-08-08T13:46:31Z
actor: claude-fable-5
edges:
- rel: about
  to: ar-xa38
  at: 1
---

Today a session learns another is blocked on it at SessionStart (orient) or by discovery. The user should not be the message bus. Candidate: a hook that checks for new arrivals in the session's purview (items filed by other sessions, steals, releases of leases we waited on) and injects an alert mid-session. Needs: which hook point, polling cadence, and noise discipline (every alert must name an act). User flagged 2026-08-08 as open, non-blocking.
