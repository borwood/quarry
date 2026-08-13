---
id: it-5j34
type: item
title: recent activity carries type and area
v: 4
status: done
provenance: assistant
created: 2026-08-12T11:37:34Z
actor: claude-fable-5
kind: feature
acceptance:
- recent activity rows show type and area columns
write_set:
- src/view.rs
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/view.rs
  at: 63656ac7ea63
---

The map page's recent-activity strip names events without type or area columns; scanning it means opening rows (user, 2026-08-12, ui-cleanup session). Add both columns at least — type from the node, area from its about edges, in the register the page's uniform tables already use (type · title · status · v · updated).
