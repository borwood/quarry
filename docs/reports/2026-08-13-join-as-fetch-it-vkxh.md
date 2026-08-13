# Dispatch report: join-as-fetch lands (it-vkxh)

Dispatched 2026-08-13 under dc-zbxj; the last dispatch to ride the
paste-whole payload and per-shell export — this landing retires both.
Lease `src/**` + `tests/**`; 7 files observed, all inside. Verified twice:
agent and dispatcher `cargo test` both read 57 passed / 0 failed (basic)
and 1 passed / 0 failed (surface lint) off the raw result lines.

## Against the RETURN spec — all four land

- **`q-join`**: dispatch mints a 10-char single-use token
  (protocol::mint_token_n) stored on DispatchState (token/joined); the
  paste-whole payload is replaced by the one-line spawn prompt. `q join`
  consumes the token, binds identity (agent → chat → session key), records
  the association, logs a join event under the badge, renders the brief
  fresh via render::brief. Re-join by the same identity is idempotent;
  re-dispatch mints a fresh token and kills the old. The dispatcher's
  behind-check moved into the dispatch confirmation, since the brief no
  longer renders at dispatch. Spine cl-6ctr.
- **`agent-identity-injection`**: empirically verified live — hook input
  carries top-level snake_case `agent_id` (and `agent_type`) in BOTH the
  session hook and the write-guard hook, captured via a temporary probe
  (removed before the final build). teach::hook_agent_id reads defensively
  across key spellings. QUARRY_AGENT injects alongside CHAT/SESSION/ACTOR
  (round-trip verified in the agent's own shell); badge_for resolves
  env → acting[agent] → acting[chat] → acting[session], each only while its
  dispatch is live. The export instruction is gone from the payload;
  QUARRY_DISPATCH env survives as the out-of-coverage override. Spine
  cl-z6gc.
- **`join-gate`**: lease_check gained a dispatched parameter — no-badge
  writes into a dispatched lease's globs deny with the teaching line naming
  the item and q join, for the holder session (the old holder-Allow is
  gone) and unbound contexts; foreign sessions keep C7. Solo leases keep
  today's behavior. The gate fired on its own author in production: the
  agent's probe-removal edit was denied (its dispatch predates tokens), it
  took the env-override road, the retry passed — deny → associate → allow
  verified live end to end. Spine cl-2pc9.
- **`work-only-stamping`**: badge_for dropped held/session resolution
  entirely; a new boundary_badge (env → associations → held) feeds
  boundary_refusal, so a dispatching chat still refuses wrap/resume/retire
  while its own acts stamp nothing. Spine cl-9nxr, which supersedes
  cl-vsew (its held/session-resolution claim died with this landing;
  assistant provenance, edge affirmed).

Backward compat: pre-token state files parse (Option + serde defaults);
the agent's own live pre-token dispatch was never orphaned — association
added, trace and observed set clean, harvested normally by the dispatcher.

## Documented choices

A second identity presenting a consumed token refuses (one badge, one
agent; multi-badge stays deferred on th-6upm) — re-dispatch for a fresh
token. An identity-less q join refuses WITHOUT consuming the token, naming
both fixes. Re-dispatch resets joined but leaves prior acting associations
pointing at the still-live badge — accepted, inside the deferred
multi-badge scope.

## Agent's reflections (substance)

The gate denying its own author mid-build was the best possible test:
the moment the new exe went live the agent became the unjoined agent the
teaching line was written for, and the line read correctly aimed at
itself. Doubts: (1) the join-gate can over-deny a holder session that also
holds a separate overlapping solo lease — rare, errs toward denial,
uninvestigated; (2) an env-badged act with only chat identity from a true
subagent still smears the association onto the dispatcher's chat key —
near-theoretical now that agent_id is verified present. Friction: mid-
build, source and enforcement disagree until the exe swap, and the serve
loop holding the exe lock makes that window a manual dance. Surprise: the
agent's hook session_id really is the dispatcher's chat id — agent-keyed
association is not a nicety, it is the only thing separating an agent's
writes from the chat that spawned it.

## Dispatcher's judgment

All four outcomes hold: independent test run agrees, observed set exactly
the lease, and the production firing of the gate on its own author is
stronger evidence than any test. Landed done and released. it-pdnq
(ordering-dependent observation) closed done — the constructed association
and the gate remove the learned-association window it watched. The
over-deny doubt (solo lease overlapping a dispatched zone) left as a
recorded edge; it errs toward denial and composes with it-z2sy's existing
solo-lease accrual bug if anyone hits it.
