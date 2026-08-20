---
id: it-csm3
type: item
title: 'attention misattribution: agent verbs spend the holding session''s watermarks and drift'
v: 9
status: done
provenance: assistant
created: 2026-08-15T05:48:35Z
actor: claude-fable-5
kind: bug
acceptance:
- 'lands nothing new: area watermarks and drift-delivery consumption key on the acting identity - a joined agent''s reads spend badge-scoped attention, the holding session''s watermarks and deliveries survive its arcs untouched, and a session''s own first write into an area its own eyes never read still gates'
aliases:
- side-effect-misattribution-agent-verbs-spend-the
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-pwyd
  at: 2
- rel: depends-on
  to: dc-zbxj
  at: 1
---

Narrowed 2026-08-15 under dc-pwyd; heartbeat struck from the original filing - subagent heartbeat touches are correct by harness fact (dc-p9xr: a subagent cannot outlive its parent chat). THE BUG: watermarks and drift-delivery consumption bind to the env-injected session, not the acting identity. A dispatched agent's q verbs advance the holding session's area watermarks (gates later pass silently for a session that never read) and consume its since-your-last-read drift deliveries (the it-hjed agent watched one spend on its own screen, owed to a chat that never saw it). THE FIX: attention state keys on the ACTING identity - a joined agent's reads spend badge-scoped attention, never the holding session's. dc-zbxj's stamping-follows-the-work extends to attention. Fix before successor-repo adoption (dc-ygzz urgency).
