---
id: it-x8ms
type: item
title: 'area-first-touch gate: read-first delivered at the first write into an area'
v: 3
status: sketch
provenance: assistant
created: 2026-08-09T08:02:50Z
actor: claude
kind: feature
ratified:
  by: user
  date: 2026-08-09
acceptance:
- first write into an area not opened this session intercepts with the area's derived read-first; q resume executes the saved intent
- a same-session q open of the area satisfies the gate silently; the committed event log gains no read events
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Settled 2026-08-09. The choke point is the about-attachment: the first write into an area per session intercepts unless the area was already opened this session. Memoization is per (session, area), so it fires rarely by construction and never ritualizes. The payload is DERIVED — the area's read-first (charter, in-force decisions, registered docs, open threads; essentially q open output), never a hand-authored body that could stale. q open records the read in machine-local session state alongside gate memoization — the committed log stays mutations-only. Diligent path: open first, gate passes silently, zero friction. Backstop path: intercept, deliver, q resume with saved args. PER-AREA WATERMARK (user refinement, 2026-08-09, replacing the one-time drift re-fire): the read record generalizes to a per (session, area) watermark over the event log. Own-session events advance it silently; any verb touching the area with foreign events beyond the watermark prints the delta INLINE (not a gate) and advances the watermark on delivery — each change said once. First touch with no watermark = the full read-first gate; thereafter, a continuous drift surface. Coverage relative to existing surfaces: same-node 24h presence notes are node-scoped; resume arrivals are boundary-time; the dc-sgyu alert bus is mid-session but deliberately CLOSED to acts (filed-into-purview, steal, unblock) and excludes content drift, which stays pull-paced by ruling. The watermark surface shares cursor machinery with the alert bus but is NOT an alert: area-touch-paced, consistent with citation staleness staying pull-only. Silence default; scan is log-tail-since-watermark, watch-listed like the alert scan.
