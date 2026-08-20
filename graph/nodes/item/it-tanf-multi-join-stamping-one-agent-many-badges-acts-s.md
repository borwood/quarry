---
id: it-tanf
type: item
title: 'multi-join stamping: one agent, many badges - acts stamp the last join, not the item served'
v: 6
status: done
provenance: assistant
created: 2026-08-16T03:36:35Z
actor: claude
kind: bug
acceptance:
- 'lands nothing new: q join refuses a second live badge for an identity, naming the live badge and the roads out; re-join of the same badge stays an idempotent read; the acting map holds one badge per identity by construction'
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
---

Observed 2026-08-15 at the lexicon trio harvest (one agent, three sequential joins per dc-qvtz): the acting map binds an identity to one badge, so each join silently overwrote the last — every file write and all 11 graph acts stamped the last-joined badge (it-sc2u); the first two badges observed nothing but their join events. The work was real and spanned all three items; per-item replay (q query dispatch <item>) and observed-vs-leased misreport under this shape — exactly the surfaces harvest trusts.

The settled fix (user-ruled 2026-08-20, option 3 of the sketch's three): SINGLE ITEM AT A TIME, HELD BY CONSTRUCTION.

1. q join refuses a second live badge for an identity: the refusal names the live badge and the roads out — stop and report so the dispatcher harvests, or take the work as separate dispatches, or the bundle shape when th-zzqv lands. Re-join of the same badge stays an idempotent read.
2. The acting map holds one badge per identity by construction — the overwrite path is dead, not discouraged.
3. Rejected alternatives, for the record: per-act badge selection (graph acts have no mechanical routing — judgment-ware where construction is possible) and the joint badge (institutionalized over-attribution; every item's replay shows every act).

Consequence, named: since harvest is the dispatcher's verb, an agent can never clear its own badge, so this ends the sequential-multi-join pattern entirely. Multi-item work by one agent becomes separate dispatches or the bundle shape — the trio WAS a bundle, dispatched as three loose items because the shape does not exist yet; this incident is evidence on th-zzqv. th-78uc (one verb set) leans on this refusal: self-join requires one-badge-per-identity to be true.
