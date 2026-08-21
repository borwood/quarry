---
id: it-nngn
type: item
title: 'protocol-gate memos spend the holding session: the once-per-rule key rides env session, not the acting identity'
v: 5
status: done
provenance: assistant
created: 2026-08-20T09:50:24Z
actor: claude
kind: bug
acceptance:
- 'a protocol memo is spent by the reader that actually saw the teach: the once-per-rule key rides the acting identity, badge-scoped for a joined arc and session otherwise, so a joined agent never marks a rule delivered to its holding session nor inherits one that session already consumed'
witness:
- line: 'a protocol memo is spent by the reader that actually saw the teach: the once-per-rule key rides the acting identity, badge-scoped for a joined arc and session otherwise, so a joined agent never marks a rule delivered to its holding session nor inherits one that session already consumed'
  by: chat:b8214cb4-bf6e-42fa-9ceb-317d5660ca2d
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-pwyd
  at: 2
---

Found landing it-csm3. protocol::gate_if_needed and take_intent key their once-per-(session, rule) delivery memo on current_session (src/protocol.rs) - the same env-inherited key it-csm3 evicted from watermarks and drift deliveries. A joined agent tripping a protocol gate marks the rule DELIVERED to the holding session, whose eyes never saw the teach; conversely an agent never receives a protocol its holding session already consumed - the dc-pwyd has-this-READER-seen-it question answered with the wrong reader. The fix shape is the one it-csm3 landed: key the memo on coord::attention_key (badge-scoped for a joined arc, session otherwise). Note the fallback divergence while there: protocol uses "default" where attention state uses "unbound" - unifying changes memo continuity once, machine-local. Filed on sight per dc-ygzz.
