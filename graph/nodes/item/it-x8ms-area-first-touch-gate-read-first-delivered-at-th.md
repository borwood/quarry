---
id: it-x8ms
type: item
title: 'area-first-touch gate: read-first delivered at the first write into an area'
v: 1
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

Settled 2026-08-09. The choke point is the about-attachment: the first write into an area per session intercepts unless the area was already opened this session. Memoization is per (session, area), so it fires rarely by construction and never ritualizes. The payload is DERIVED — the area's read-first (charter, in-force decisions, registered docs, open threads; essentially q open output), never a hand-authored body that could stale. q open records (session, area, area-version) in machine-local session state alongside gate memoization — the committed log stays mutations-only. Diligent path: open first, gate passes silently, zero friction. Backstop path: intercept, deliver, q resume with saved args. Flagged extension (assistant, strike if unwanted): drift re-fire — if the area neighborhood changed after the recorded open, the gate fires with only the delta (interleave-the-delta philosophy, section 6).
