---
id: it-w5yv
type: item
title: 'prose ids: unpack, mention index, and the authoring surfaces'
v: 15
status: done
provenance: assistant
created: 2026-08-11T04:26:48Z
actor: claude
kind: feature
acceptance:
- 'lands `id-unpack`: rendered bodies in open, brief, and the view expand bare ids to id [type: `title`], dead targets labeled, view hyperlinked; code spans skipped'
- 'lands `mention-index`: open and the view list derived mentioned-by backlinks, labeled distinct from edges; blast and behind never traverse mentions, test-covered'
- 'lands `mention-surfaces`: mint/edit confirmations echo resolved titles beside ids, dangling id-shapes warn quietly as a question never a gate, real-edge upgrades offered as judgment; wrap lints danglers'
- teach surfaces carry the split rule (bodies cite by id, conversation keeps titles); q init --claude regenerated
- 'instance 3 retro-fixed live: th-n8h6 body names th-sjus by id and both directions render (unpack forward, derived mention reverse)'
- cargo test passes with the raw test result line read unfiltered; false-positive shapes (th-read, do-over) and code-span skipping covered
write_set:
- src/**
- tests/**
- .claude/**
archived: true
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
- rel: depends-on
  to: dc-wwnk
  at: 3
---

Build shape ratified in dc-wwnk (prose ids: bodies cite by immutable id, unpacked at render; mentions derive, never store) — read its body as the spec; this body carries the build particulars. (1) `id-unpack`: a shared expansion pass over rendered bodies — q open, q brief, and the view — replacing each bare id with id [type: `title`]; archived/refuted/superseded targets carry their status label; the view hyperlinks to the node anchor. The regex is the closed shape \b(ar|it|th|dc|cl|do)-[a-z0-9]{4}\b with backticked code spans skipped. (2) `mention-index`: the same scan, derived wherever backlinks are computed today — q open and the view list "mentioned by <id>" under a derived-mentions label, distinct from edge backlinks; blast and behind never traverse mentions. (3) `mention-surfaces`: on mint and edit (new/edit/claim/rule paths), each resolved id in the body echoes its title beside it in the confirmation output (verification against wrong-but-real ids); id-shapes resolving to nothing warn quietly (never a gate; expect hyphenated false positives like `th-read` — the warning must read as a question, not an error); when ids resolve, offer the real-edge upgrade commands as judgment (an index for judgment, never an auto-link — same voice as the touches line); wrap lists dangling id-shapes in bodies as lint. (4) Teach: the skill/guide line splits the house rule — bodies may cite by id (always unpacked at render), conversation with the user keeps titles; q init --claude regenerated. (5) Retro-fix of the post-mortem's instance 3, the feature's first live use: q edit th-n8h6's body so "the separate deferred thread" names th-sjus by id, then verify the unpack and the derived mention render in q open both directions. Watch perf: the scan rides render and index paths, not the write hook; the write path stays graph-load-free.
