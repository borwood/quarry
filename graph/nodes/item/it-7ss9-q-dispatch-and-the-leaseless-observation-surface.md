---
id: it-7ss9
type: item
title: q dispatch and the leaseless observation surfaces
v: 15
status: done
provenance: assistant
created: 2026-08-09T19:19:00Z
actor: claude
kind: feature
acceptance:
- q dispatch performs brief+reserve+in-flight+payload as one act, refuses unbound with a session-adopt pointer, and carries QUARRY_DISPATCH=<item-id> in the payload
- the guard accrues touched paths machine-locally (per-session leaseless, per-item under badge) with no graph load added to the write path; the first badged write echoes the contract; the leaseless threshold nudge fires once per session with a derived item match
- q verbs stamp the badge on logged events and a dispatch-writes query exists, advertised in harvest output
- landing prints observed-vs-leased and report-registration homework; spine_check consumes observed files when present; wrap lists leaseless touched files and unharvested dispatches
- the brief renders spine bodies, a render-time behind check addressed to the dispatcher, and a reflections/stop-report RETURN section; a resource-rules protocol rider exists on=brief
- cargo test passes with the raw test result line read unfiltered, and the dispatch chain is exercised end-to-end once in a scratch host repo
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: about
  to: ar-xa38
  at: 1
- rel: depends-on
  to: dc-cc76
  at: 2
- rel: depends-on
  to: th-dbn4
  at: 3
- rel: depends-on
  to: dc-kpqg
  at: 2
- rel: depends-on
  to: it-ubqe
  at: 2
---

Build shape ratified in dc-cc76 (lease boundary) and dc-kpqg (dispatch citizenship). (1) `q dispatch <item>`: one act — render the derived brief, reserve, set in-flight, print the hand-off payload with QUARRY_DISPATCH=<item-id>; refuses unbound and points at session adopt; advertised in ready listings, brief output, and the guide. (2) Solo-path advert: q reserve surfaced when a session takes up an item and is about to write code. (3) Accrual: machine-local touched-set, per-session when leaseless / per-item under a badge; O(1) append on the write path, no graph load. (4) Guard growth: first-touch contract echo under a badge (item, globs, RETURN); badge-keyed drift notice off accrual state; once-per-session threshold nudge (leaseless) with derived item match over write-sets and file-edges; denial only at C7/C8. (5) Verbs stamp the badge on events — "what did this dispatch write" becomes a query, advertised in harvest output. (6) Harvest: dispatcher judges acceptance and lands (harness done is a stop signal, never a transition); observed-vs-expected comparison (files touched, claims minted under badge, threads filed); report registration as doc, one homework command, partial/stop reports register too; re-dispatch briefs carry prior reports. Wrap picks up leaseless touched files and unharvested dispatches. (7) Brief fixes: spine shelf renders bodies not titles; render-time behind check confronts the dispatcher; consider backlinks; RETURN spec grows the reflections section and stop-report instruction; resource-rules protocol rider (on=brief) for this machine's build serialization. Test the chain end-to-end when built — user flagged testing explicitly.
