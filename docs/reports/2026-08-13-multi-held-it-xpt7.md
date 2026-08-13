# Dispatch report: multi-held dispatch lands (it-xpt7)

Dispatched 2026-08-13 under dc-qyr5 — the first dispatch in this repo to
ride the join-as-fetch hand-off end to end: the spawn prompt was one line,
the agent joined with its token, fetched its own brief, built, and
reported. Lease `src/**` + `tests/**`; 6 files observed, all inside.
Verified twice: agent and dispatcher `cargo test` both read 59 passed /
0 failed (basic) and 1 passed / 0 failed (surface lint) off the raw
result lines.

## Against the RETURN spec — all three land

- **`multi-held`**: the same-chat second-dispatch refusal is deleted
  (flip site commented in ops.rs). The keying rethink taken: `held` keys
  by ITEM id, the dispatching chat rides each entry as a `holder` field —
  the dc-qyr5 invariant expressed directly in the map shape, so the
  double-hold refusal and per-item clear are exact lookups by
  construction. Both legacy on-disk shapes (v1 single-slot, v2
  per-chat-keyed) migrate in place at load, each with a migration test,
  including a live token and joined agent surviving the re-key.
- **`item-one-chat`**: cross-chat dispatch of a live item refuses naming
  the holding chat key and session, advertising `--steal --reason`.
  Same-chat re-dispatch stays free (fresh token, old dies, joined reset).
  Steal demands its reason and takes the dispatch WHOLE: held entry
  re-homed, lease re-homed with globs intact, fresh token, the old
  agent's acting associations cleared so it stops stamping into an arc it
  no longer works, and a logged steal event carrying from_chat,
  from_session, from_joined, and the reason. One test covers every leg.
- **`boundary-harvests-all`**: boundary_badge (first-found) became
  boundary_badges (enumerate, deduped: env, joined association, every
  held entry under this chat/session). The refusal shared by wrap,
  resume, and retire lists each live dispatch with its q harvest command.
  Tested: two dispatches enumerate both, a foreign chat's dispatch stays
  out, an unbadged chat wraps freely while others fly.

Spines: cl-s98g (`multi-held-dispatch`, supersedes cl-amza) and cl-ahzk
(`boundary-harvests-all`, supersedes cl-farc), both sourced to
src/coord.rs, both supporting dc-qyr5, supersede edges affirmed.

## Provisional calls (none contradict rulings; dispatcher concurs)

1. Dispatch --steal takes only live dispatches; a foreign solo lease
   keeps its old refusal — q reserve --steal remains that road.
2. Same-chat re-dispatch does not clear the old agent's acting
   association (steal does) — a re-dispatched item's prior agent can
   stamp until the new one joins. Accepted residue, adjacent to the
   deferred multi-agent-per-item scope th-6upm closed around.
3. A chat re-bound to a new q session mid-flight still refuses same-chat
   re-dispatch at the lease layer — unchanged behavior.

## Agent's reflections (substance)

The keying rethink was the whole job — once held keyed by item, the three
acceptance lines mostly fell out as lookups and the diff came in smaller
than the seam list suggested. Frictions flagged for mining: stolen_from
threads through DispatchOutcome only so the CLI can print the loud line —
the steal event is the real record, and the echo could render from the
log instead; the boundary refusal for an env-only badge renders a bare id
with no title (the held map is the only title source there, and loading
the graph inside a refusal path was declined). A surprise worth keeping:
the v2→v3 migration made a pre-existing blur visible — under v2 two chats
bound to one session could each hold the same item; item-keying collapses
that to last-writer at load, which IS the new invariant, but the collapse
is silent. The eight-argument dispatch() signature is past wanting a
params struct; flagged with an allow rather than reshaped under lease.

## Dispatcher's judgment

All three outcomes hold: independent test run agrees, observed set is
exactly the lease, and the hand-off itself was the ruling's proof — one
line out, brief fetched at join, report back. Landed done and released.
The exe-lock dance (serve holding target/release/q.exe through the
release build) has now cost three dispatches a manual step; left visible
here for the next boundary conversation rather than silently absorbed.
