---
id: it-6gj9
type: item
title: 'unpack click-target: the whole expansion is the link, and reads as one'
v: 5
status: done
provenance: assistant
created: 2026-08-12T11:37:34Z
actor: claude-fable-5
kind: feature
acceptance:
- clicking anywhere on an unpacked citation navigates to its node; unpacked citations read as marked spans
write_set:
- src/mention.rs
- src/view.rs
- tests/basic.rs
- src/surface.rs
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: file:src/mention.rs
  at: 54f63678b662
---

unpack_html links only the bare id inside an expanded citation; the title text beside it is dead to the pointer and visually unmarked (user, 2026-08-12, ui-cleanup session). Wrap the full id [type status: title] expansion in the anchor and mark the whole span with a highlight class in the page CSS. HTML register only — CLI and brief registers untouched (dc-nnf5 atoms own the registers; cl-qxxp defined the unpack).
