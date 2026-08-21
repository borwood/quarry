---
id: cl-2j49
type: claim
title: '`gate-memo`: a protocol gate memo is attention state — it rides the reader key and dies with the badge'
v: 5
status: ratified
provenance: assistant
created: 2026-08-21T15:49:34Z
actor: claude-opus-5
kind: vein
ratified:
  by: claude-opus-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/protocol.rs
  at: d0d96f5ae2d0
- rel: source
  to: file:src/coord.rs
  at: b891f4b5a953
- rel: source
  to: file:tests/basic.rs
  at: e5b9747a8c32
---

`gate-memo`: a protocol gate memo is attention state — it rides the reader key and dies with the badge

The gate-tier protocol memo answers has-this-READER-seen-the-teach, so it keys on coord::attention_key like every other attention surface (dc-pwyd; it-nngn extends cl-b2z2 onto this one). protocol::gate_if_needed spends the ACTING identity — badge-scoped for a joined arc, session otherwise — so a joined agent tripping a gate marks the rule delivered to its own eyes, never to the holding session whose env it merely inherits, and never inherits one that session already consumed. protocol::save_intent records the same reader, so the area gate (which already gated on attention_key) and the intent it saves name one identity. coord::clear_dispatch calls protocol::clear_delivered beside clear_area_reads: BOTH badge-scoped attention surfaces die on the one key, so a re-dispatched arc's next agent is taught rather than handed the replaced agent's consumption. Saved intents are deliberately left alone — one-time tokens on a 1h TTL are work-in-progress, not attention; an orphaned token still resumes and its rule re-gates.

Continuity across the rename is carried by serde: `#[serde(alias = "session")]` on Delivered.reader and StoredIntent.reader reads every pre-it-nngn row, and for any BOUND session the old env-session string IS the new reader key, so no real memo re-delivers. The one row that moves is the unbound fallback — protocol said "default" where attention state says "unbound"; unified on coord::session_key's spelling, it re-delivers once, machine-local.

The roster this completes, for whoever adds a fourth: area watermarks and drift deliveries (cl-b2z2), and the protocol gate memo (here) — one key, one clearing point. What does NOT ride it: the STORED intent's own lifecycle, and every session-keyed surface outside attention (heartbeat, leases, held entries), which dc-pwyd deliberately leaves on the session.

Pinned by a_protocol_memo_is_spent_by_the_reader_that_saw_the_teach and a_legacy_session_keyed_memo_carries_onto_the_reader_key in tests/basic.rs, both end to end through the spawned binary because the reader only diverges from the session in a real dispatched shell. Each half probed against the regression it guards: restoring current_session keying reproduces the defect verbatim (the holding session's own first mint executes untaught, the memo row reading "design" where "badge:it-…" is right); dropping the clear_delivered call fails the re-dispatch assert; and dropping the serde alias loses every standing memo to the load's silent default.
