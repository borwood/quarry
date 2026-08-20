---
id: it-nngn
type: item
title: 'protocol-gate memos spend the holding session: the once-per-rule key rides env session, not the acting identity'
v: 2
status: sketch
provenance: assistant
created: 2026-08-20T09:50:24Z
actor: claude
kind: bug
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-pwyd
  at: 2
---

Found landing it-csm3. protocol::gate_if_needed and take_intent key their once-per-(session, rule) delivery memo on current_session (src/protocol.rs) - the same env-inherited key it-csm3 evicted from watermarks and drift deliveries. A joined agent tripping a protocol gate marks the rule DELIVERED to the holding session, whose eyes never saw the teach; conversely an agent never receives a protocol its holding session already consumed - the dc-pwyd has-this-READER-seen-it question answered with the wrong reader. The fix shape is the one it-csm3 landed: key the memo on coord::attention_key (badge-scoped for a joined arc, session otherwise). Note the fallback divergence while there: protocol uses "default" where attention state uses "unbound" - unifying changes memo continuity once, machine-local. Filed on sight per dc-ygzz.
