---
id: it-auj7
type: item
title: 'no verb shows an agent how its own report parsed: three throwaway instruments built in two days'
v: 2
status: sketch
provenance: assistant
created: 2026-08-21T11:11:39Z
actor: claude-fable-5
kind: feature
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Flagged from two agent seats and confirmed from the dispatcher's, 2026-08-21. it-drsu repaired the parse; this is the surface gap that let the parse go unseen, and it survives the repair.

queries::declared_user_owned_calls reads a report's user-owned section and its count feeds framings::user_owned_reconcile at the harvest seat. No verb prints that reading. An agent writing a report cannot see what the machine will make of it: not at write time, not before it stops, not anywhere. The only roads to certainty are reading the function or building an instrument.

Three instruments in two days, by three different hands, all thrown away after one use: the it-2eqk agent's tests/zz_scratch_parse_check.rs (confirming its own draft would have counted zero), and two more at the it-drsu build — a scratch check and a corpus probe over all 48 registered reports. Each was correct work; each died with its session; the next agent starts from nothing. That repetition is the measurement here.

The cost is not the wasted instrument. It is that an agent who does NOT build one has no way to know, and the failure it cannot see is silent in the dangerous direction: a section the parser reads as zero prints 'reconciled' at the judgment seat over a call the report declared. it-drsu made that specific shape safe; it did not make the reading visible.

The shape, roughly: a verb that reads a report — a registered doc, or a path before registration — and states what the parse found, in the same words the harvest will use. q brief now names the arc's report path (cl-ue2e), so the natural seat is a flag or verb an agent can point at that path before it stops. main.rs and queries.rs are the surfaces; declared_user_owned_calls is already pub and needs no change.

Note the narrower half is cheaper and may be enough: harvest could print the parsed count beside the reconcile line it already renders, which costs nothing new and at least makes the reading visible to the judge. The agent-facing half is what actually closes the loop, because the agent is the one who can still fix the report.

Neighbourhood: it-drsu for the parse this leans on, cl-cjbb for why the accounting is load-bearing, and cl-ue2e for the leased path that makes an agent-facing check reachable.
