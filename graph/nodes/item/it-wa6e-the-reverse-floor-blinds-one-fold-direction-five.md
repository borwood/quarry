---
id: it-wa6e
type: item
title: 'the reverse floor blinds one fold direction: five-char pairs join forward only'
v: 7
status: done
provenance: assistant
created: 2026-08-16T03:41:54Z
actor: claude
kind: bug
acceptance:
- 'lands nothing new: floors measure the raw token before folding in both passes, a five-char folded pair joins forward and reverse alike, and the bare floors stand unchanged'
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: depends-on
  to: dc-qvtz
  at: 3
---

From the trio build (it-nuw5, 2026-08-15): the reverse pass keeps its six-char token floor, so a five-char folded pair (lease/leases) transits relatedness forward but never reverse — the floors and the fold interact asymmetrically. Both floors stand by dc-qvtz; the defect surface is the asymmetry alone.

The settled fix (user-agreed 2026-08-20): floors measure the RAW token before folding, in both passes — leases (six chars) passes the reverse floor, then folds to lease for the compare. Symmetry restored, no floor lowered, dc-qvtz intact.
