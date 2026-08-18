---
id: cl-az38
type: claim
title: '`ratify-at-harvest`: the landing act ratifies the arc''s vein and feature mints - badge by the dispatcher''s hand, solo self-ratified, assayer stamped, no bump'
v: 3
status: ratified
provenance: assistant
created: 2026-08-17T09:21:56Z
actor: claude-fable-5
kind: vein
ratified:
  by: claude-fable-5
  date: 2026-08-17
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/ops.rs
  at: c4e91d7fe548
- rel: supports
  to: cl-2cam
  at: 2
---

The landing act ratifies the arc's vein and feature mints (dc-drr6). ops::ratify_landing fires from the q set status=done handler: a dispatched item ratifies the claims minted under its badge (read off the log's dispatch stamps, so the trace survives release); a never-dispatched item landing under the session that worked it self-ratifies the session's own mints across the arc window (lease since, else the last in-flight flip; inclusive RFC3339 compare). Only vein and feature claims on the asserted rung ratify - measured, readings, contracts, and kindless mints never ride this ladder. The assayer is stamped in front.ratified {by, date} and the act logs op ratify with assay harvest|solo and the landing item as cause - a fallen claim shows its assayer. No version bump: an assay records who stood behind the claim, not new content (the affirm-no-bump rationale, th-uvu9), so citers never go behind over good news. q harvest names the pending assay with the ratified verbatim line before the dispatcher's hand moves (render.rs); the done-flip prints the harvest or solo line with what ratified (main.rs).
