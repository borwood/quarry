---
id: it-nmhu
type: item
title: 'log-index cursors: retire the timestamp blind spot'
v: 2
status: done
provenance: user
created: 2026-08-09T14:56:06Z
actor: claude
kind: feature
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

User-ruled 2026-08-09: the watermark and alert cursors move from second-truncated timestamps (ts-greater-than scans, which permanently miss foreign events landing the same second as the cursor) to log-index positions — the log is append-only, so events-after-position-N is exact with no clock in the question. Fix now rather than watch-list: the failure is hard to discover later (an agent would have to catch the miss, derive the cause, surface it, and carry it back to the quarry repo). Legacy timestamp cursors convert on first read by counting events at-or-before the stamp, preserving old semantics without re-delivery noise.
