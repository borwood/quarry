---
id: it-jsu5
type: item
title: 'stale acting row survives a same-chat re-dispatch: the replaced agent keeps stamping'
v: 1
status: sketch
provenance: assistant
created: 2026-08-20T11:01:54Z
actor: claude
kind: bug
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Noticed 2026-08-20 building it-tanf (one badge per identity). The same-chat re-dispatch arm of ops::dispatch (the documented recovery flow: fresh token, joined reset to None) leaves the acting map untouched - the REPLACED agent association still points at the live badge, so its stray acts keep stamping into an arc it no longer works. The steal arm clears exactly this and states the rationale on the clear ("so a stolen-from agent later acts stop stamping into an arc it no longer works"); the re-dispatch arm has the same replacement semantics and no clear. Sharpened by it-tanf: under one-badge-per-identity the replaced agent identity also stays BLOCKED from joining any other work until harvest, because its stale row is a live binding (coord::live_acting_badge honors it while the badge is held). Repro: dispatch item X, join as agent A; re-dispatch X same chat (joined resets, token fresh); agent B joins; load_dispatches shows acting A->X and B->X both live - A stamps X and refuses all other joins. Candidate shape: the re-dispatch upsert clears acting rows for the replaced item the way the steal does - a same-agent recovery re-binds at its re-join anyway (join now restores a lost row by construction, cl-kggw).
