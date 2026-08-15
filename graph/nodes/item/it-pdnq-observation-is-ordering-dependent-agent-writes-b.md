---
id: it-pdnq
type: item
title: 'observation is ordering-dependent: agent writes before the first badged act go unattributed'
v: 2
status: done
provenance: assistant
created: 2026-08-13T08:05:16Z
actor: claude
kind: watch
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Observed 2026-08-12 at the per-chat badge landing (do-75ut, hole 1 of the five the agent documented): the chat-to-badge association is learned from an agent first badged q act, so file writes BEFORE that act resolve no badge and the accrual never sees them - observation is ordering-dependent. The hand-off payload now says export early, which is discipline, not construction. Construction candidates when this hurts: dispatch learning the spawned chat at hand-off time, or the guard buffering unattributed writes for late association. The other four holes stay as documented trade-offs in the report (deliberate no-attribution, forgetful-agent wrap, session-key collision without hook coverage, double-dispatch of one item across chats). Promote when a harvest shows a hole in the observed set.
