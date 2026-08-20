---
id: it-8tcy
type: item
title: 'broken pipe panics the verb: stdout closes mid-print and homework is lost'
v: 6
status: done
provenance: assistant
created: 2026-08-16T02:44:35Z
actor: claude
kind: bug
acceptance:
- 'lands nothing new: q verbs complete every state mutation regardless of stdout''s fate - output failure is non-fatal and a closed pipe exits quietly without panic; the verb paths are audited for mutations sequenced after prints with each confirmed to survive output failure; and homework re-derives on demand via q query homework so the act-time print is a delivery, not the only copy'
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Observed 2026-08-15: piping a verb through head -1 closed stdout early and q panicked (os error 232, failed printing to stdout) after the write landed but mid-homework print — the edit succeeded, the homework enumeration was lost. Three defects braided (the third found code-reading 2026-08-20):

1. The panic itself: a closed pipe is a normal fate for CLI output (head, a dying terminal) and should end output quietly, never panic.
2. The sequencing hazard, worse than the panic: real state mutations FOLLOW prints in main.rs — print_homework runs before area_watermarks recording, readings sweeps, view regeneration. A mid-print panic does not just lose text; it skips the state work sequenced after it. The incident got lucky that the edit preceded the print.
3. The loss surface: homework prints once, at act time — but homework is DERIVED from current graph state, not stored, so the honest fix is making the derivation pull-able, not protecting the print. (Kin: it-pafx, surfaces self-certify.)

The settled fix (user-agreed 2026-08-20):

1. PRINTS BECOME NON-FATAL: a print helper that swallows io errors — the verb completes ALL state work regardless of stdout's fate and exits without panic. Fixes 1 and 2 in one move: no mutation can be skipped by an output failure, whatever order the code runs in.
2. AUDIT THE SEQUENCING while there: enumerate every mutation sequenced after a print in the verb paths and confirm each survives output failure under the helper; reorder any genuinely print-dependent state work found.
3. HOMEWORK RE-DERIVES ON DEMAND: q query homework <node> exposing the same derivation print_homework uses — the act-time print becomes a delivery of a derivable surface, not the only copy. Wrap backstops the aggregate; this is the per-node pull.
