---
name: quarry
description: The work graph in this repo's graph/ directory — decisions, claims, threads, items, docs. Use at session start to get oriented (q query queue / ready / shaping), before design work (q open the relevant nodes), when recording a user ruling, extracting a claim, queueing a thread for the user, or closing a session (review behind, affirm what you re-read). All graph writes go through q verbs, never file edits.
---

QUARRY — THE JUDGMENT LAYER
(mechanics live in `q --help` and `q <verb> --help`; this is when and why)

WHAT THIS IS
A work graph: decisions, claims, threads, items, docs, areas — typed nodes
with version-stamped edges, stored under graph/ in this repo. Files there
are never edited by hand (a hook denies it); every write goes through a q
verb, which bumps versions, stamps refs, and enforces the constraints.

SESSION SHAPE
Open with the three orientation queries: queue (threads awaiting the user,
answerable now), ready (items dispatchable now), shaping (upcoming work and
its blockers). Before designing anything, `q open` the areas and items it
touches — the neighborhood brief IS the context payload, and it shows what
changed behind every stale ref. Close a session by reviewing behind and
affirming ONLY what you actually re-read: affirm is a recorded act of
review, never a way to silence a marker. `q wrap` runs the whole boundary
lint — owed threads, stale refs, in-flight work, unfiled nodes, unrecorded
rulings, uncommitted graph changes. In conversation, refer to nodes by
TITLE — ids belong in commands, not in prose to the user.

PROVENANCE HONESTY
Your analyses and proposals are assistant-provenance — the default. Mark
user provenance or ratification only for what the user actually said or
approved, in their words. The tool refuses an assistant decision that
settles or supersedes a user-provenance node (C3). That refusal is not an
obstacle: it is the queue telling you the call belongs to the user.

CLAIMS — EXTRACT, NEVER MINT WHILE WRITING
Prose stays prose: journals, spike reports, and design notes are docs,
written normally and registered whole. A claim node exists only when
something depends on a statement or kills it — extract it at that moment,
with its subjects and its GROUNDING: a source doc, a source file (the code
it was read off, blob-stamped), a method (how it was measured), or user
provenance. No free-floating assistant assertions (C2). A journal entry
written later can be attached as a source after the fact — never stop
mid-flow to manufacture one. If you are not building on it or refuting it,
it is not a claim yet.

THREADS — ANYTHING THAT NEEDS THE USER
A thread is a strand needing user input: a pick between options, a
discussion, a topic that needs a spike before it can be answered. Queue it
rather than asking ad hoc; give it depends-on edges to any intermediate
work it spawns, and it will surface only when answerable. Record the user's
ruling with the rule verb; the thread resolves and its dependents unblock.
OWNERSHIP IS DATA: the queue is the COMPLETE list of what the user owes.
If the user owes a call and no thread exists, mint one — never track a
user obligation in prose or memory. The hard edge is C3: agents cannot
settle what the queue holds.

ITEMS — SKETCH EARLY, DERIVE BLOCKAGE
Upcoming work enters as sketch the moment it is anticipated, with its
expected relationships as ordinary edges, and is refined as threads
resolve. Ready is a stored intent; whether anything still blocks an item is
always derived — never write "blocked" anywhere.

REFS AND STALENESS
Cite nodes and files with edges; stamps are automatic. behind is
information, not noise: severity 1 means something you cite was refuted or
superseded — read it before building further. When a claim falls, blast
enumerates everything leaning on it. That list IS the correction; there is
no sweep. Mutating verbs print the HOMEWORK an action creates — citers put
behind, work unblocked, threads made answerable — with the command that
addresses each. Do the homework (or queue it) before moving on.

PROJECT PROTOCOL — HOUSE RULES AS CONTENT
Quarry is an engine; this project's house rules are content, living IN the
graph. A protocol entry is a doc node, kind=protocol: its body is the
instruction; its fields pick the trigger (on=<verb>, node_type=<type>,
node_kind=<kind>) and the delivery tier. tier=inline rides the verb's
confirmation. tier=gate intercepts the FIRST matching attempt per session:
read the delivered context, do the work under it, then run q resume
<token> (your args are remembered). Author protocol like any doc:
  q new doc "journal charter" --kind protocol --body-file charter.md \
    --field on=new --field node_kind=journal --field tier=gate --about <area>
It versions, attaches, and goes stale like everything else. Engine-native
gates also exist: a steal demands --reason. Gates are for rare,
consequential, or authoring-shaped acts — never for frequent verbs.

ARCHIVING — SETTLED LEAVES LEAVE THE DEFAULT VIEW, NEVER THE GRAPH
Archived is a flag: ids resolve, edges hold, blast and behind always see
everything. Only settled statuses archive (status decides, never age);
parents with live children refuse — a parent indexes its archived
offspring, so aging leaves are found through the thing they were part of.
Docs and journals never archive (they are the record); areas retire by
status. Every surface that hides archived content says how many it hid and
how to reach them — nothing hides silently. Superseded decisions are
routine archive candidates; wrap lists what qualifies.

SESSIONS AND PARALLEL WORK
A main session's identity is its PURVIEW — a named set of areas in the
committed registry. The q CLI is agent-operated: the user never authors a
session by hand. SESSIONS PERSIST BY DEFAULT — defining one makes it
re-enterable forever, via the optional launcher script or by the user
saying "you're <name>" and the agent adopting; ephemeral (this chat only)
is the marked odd case, asked about at creation. WHEN THE USER ASKS FOR A
NEW SESSION ("start a q session covering X"): ask whether it should
persist across chats or is ephemeral; mint or verify the areas; register
the purview (session set — heed its overlap warning; distinct concerns
are the point, and deliberate overlap wants --shared leases there); offer
the launcher as a convenience, not a default; then make THIS chat the
session: q session adopt <name> (binds on the next shell call, env
injected automatically). WHEN A CHAT STARTS UNBOUND in a repo that has
sessions and the user launches into substantive work: before the first
graph write or dispatch, ask — adopt an existing session, or define a new
one (persistent or ephemeral)? The orient hook reminds you of exactly
this. AT THE WRAP OF AN EPHEMERAL SESSION, retiring it is the default
close act — do it and say so (q session retire); if its purview proved
durable during the session, instead offer converting it to a durable
session and let the user choose. WAKING AS A SESSION: run q session resume —
the derived handoff: holdings, your session's recent acts, arrivals from
other sessions, what the user is owed. Nobody writes a close block; the
wake brief is derived from the log and cannot be stale. Sessions start LEASELESS: browsing, design, and graph writes
never need a lease. Reserve AT DISPATCH — when an agent is about to touch
files — attaching the lease to the item it serves. Exclusive is the
default; a shared lease marks a co-write zone where presence-awareness
replaces mutual exclusion. Release explicitly when the arc lands (wrap
nags); a steal is always loud and logged. Cross-session requests need no
machinery: file an item into the other purview's areas with a depends-on
from your blocked item — their orientation surfaces it, and homework
reports to both sides when it lands.

EDGE MATRIX (names only)
about (anything → area or file) · part-of (hierarchy) · depends-on
(item/thread → item/thread/decision) · settles (decision → thread) ·
supports (claim/doc → decision/item/claim) · refutes (claim/doc →
claim/decision) · supersedes (same type) · source (claim → doc)

ENVIRONMENT
Set QUARRY_ACTOR to your model/agent name so provenance derivation and the
event log stay honest.

---
Generated by `q init --claude` (quarry v0.1.0). Do not hand-edit — re-run after upgrading the tool. Mechanics live in `q --help`.
