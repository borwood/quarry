---
id: cl-pq2t
type: claim
title: '`acceptance-edit`: q set removes or replaces acceptance lines - exact-or-unique-substring resolves, refusals list candidates, the record is the resolved line'
v: 3
status: ratified
provenance: assistant
created: 2026-08-20T10:07:40Z
actor: claude
kind: vein
ratified:
  by: claude
  date: 2026-08-20
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/ops.rs
  at: 289ccba3cb9e
- rel: supports
  to: it-ds6b
  at: 12
---

The acceptance repair verb (it-ds6b): q set "acceptance-=<text>" resolves the victim line - the exact line wins outright, else any substring matching exactly one line; zero or multiple matches refuse listing candidates, never a silent no-op (ops::resolve_acceptance_removal). Replace is both fields in one act - "acceptance-=<old>" "acceptance+=<new>" - one act, one log event, one bump: the contract is content, so citer stamps go behind (unlike q unlink). The echo and the log carry the FULL resolved line removed, never what was typed (the mint-echo pattern: a wrong-but-real match reads wrong in the echo) - the set event rewrites the logged field to the resolved line and carries removed_acceptance whole. The dc-p6z4 invariant inherits at mutation time: a strip that empties a ready or in-flight item demotes it to shaped loudly in the same act (ops::SetOutcome carries demoted_from; the demotion is logged with its cause). Seat rules follow the pen (dc-mpg8): authoring-by-subtraction is authoring - a witness seat removal lands a removed-flagged WitnessMark and rides the design wake review channel until the user ratifies (queries::witness_flags surfaces removal marks regardless of line presence, the line being gone by construction); design seats mutate freely; a replacement += arm passes the standing line and sequence checks and marks like any authored line. Pinned by acceptance_removal_resolves_exact_or_unique_substring_and_refuses_ambiguity, stripping_a_readied_items_last_line_demotes_to_shaped_in_the_verbs_own_output, and witness_seat_removal_rides_the_review_channel_until_the_users_word in tests/basic.rs. Extends cl-t7nx and cl-psau at the mutation station; cl-78yz channel derivation now carries the removal arm; the future acceptance-change verb cl-5n6m anticipated exists - witness_ratify now teaches the amend arm.
