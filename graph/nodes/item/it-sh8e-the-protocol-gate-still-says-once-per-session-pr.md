---
id: it-sh8e
type: item
title: 'the protocol gate still says once per session: print_gate speaks to the wrong reader now that the memo rides the acting identity'
v: 2
status: sketch
provenance: assistant
created: 2026-08-21T15:49:11Z
actor: claude-opus-5
kind: bug
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: it-nngn
  at: 4
---

Surfaced building it-nngn, and created by its landing. The gate-tier protocol memo now keys on coord::attention_key (badge-scoped for a joined arc, session otherwise), but main.rs::print_gate still tells the reader "this act carries project protocol, delivered once per session" and closes with "(args are remembered; to change them, re-run the original command - this session is now cleared for this rule)". Both sentences are false at a joined agent's seat: what was spent is the ARC's memo, and the holding session is emphatically NOT cleared - that is the whole point of the fix. A joined agent reads the line and concludes its dispatcher will not be taught; a dispatcher reads it and believes it has been.

The cure is already in the same file: main.rs::attention_scope(reader) renders "this dispatch arc" or "this session", and the area gate beside it (area_gate_if_needed) already uses it in exactly these two slots - "gated: first write into area(s) unread {scope}" and "{scope} will not be gated on these areas again". print_gate needs the same two substitutions; the reader is one call away (coord::attention_key(&store), with store in scope at the Cmd::New arm that calls it).

Filed rather than fixed because src/main.rs sits outside the it-nngn write-set ["src/protocol.rs", "src/coord.rs", "src/queries.rs", "tests/**", the report path] - an agent may not extend its own lease. Filed on sight per dc-ygzz. Depth at the surface per dc-dsdm; the wording joins the register the area gate already speaks.
