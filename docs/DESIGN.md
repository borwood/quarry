# quarry — design

**Status: PROPOSAL for ratification.** Sections restating rulings from the
2026-08-07 founding conversation are tagged **[ratified]**; everything else is
**[proposed]** — assistant-originated, awaiting the user's read. Once v0 exists,
this doc's own decisions migrate into its own graph (see § 13) and this file
becomes a rendered view.

---

## 1. What quarry is

A standalone work-graph tool for AI-native development. It holds the knowledge
a project accumulates — decisions, claims, questions, work items, prose
artifacts — as **typed nodes with typed, version-stamped edges**, stored as
plain files in the host project's repo, written only through the tool's verbs,
and consumed as **derived views** (session briefs, dispatch briefs, handoffs,
a browsable UI) that are never hand-maintained.

quarry is to a project's knowledge what git is to its files: the tool lives in
its own repo; each host project holds its own data (`graph/`) inside its own
history, so graph writes and code writes share commits and travel together.

**[ratified]** The tool is standalone, not project-local. **[ratified]** The
founding motivation: deepcraft's process corpus demonstrated that
discipline-based invariants decay — every failure mode (stale cross-references,
one-directional pointers, doc-vs-doc contradictions, re-derived dead ideas,
assistant reconciliations silently overwriting user designs, numbering
collisions, ballooning doctrine) is a graph invariant being enforced by prose
and attention. quarry makes each one **an edge, a query, or a denied write**.

## 2. Principles

1. **Truth lives in files + log; the index is derived.** `graph/nodes/` and
   `graph/log/` are the artifact. Any index (SQLite, rendered views) is
   rebuildable from them and is never the authority.
2. **Derived views are never hand-edited.** A roadmap, a handoff, a brief, a
   staleness report — all rendered. If a view is wrong, the graph is wrong; fix
   the graph.
3. **Writes go through verbs.** A host-repo hook denies freehand edits under
   `graph/`. The verb is where constraints, version bumps, and stamping happen —
   discipline is mechanical, not attentional.
4. **Every ref bears the version it was written against.** **[ratified]**
   Whole-artifact history is git's job; per-node versioning is built in, and an
   edge records its target's version (or a file's blob) at write time. Staleness
   stops being discovered by sweep and becomes a query.
5. **Provenance is first-class.** Who a thing came from (user / assistant /
   measured / external) and whether it was ratified are fields, not emoji
   conventions.
6. **Retroactive identity.** **[ratified]** Prose stays prose. A claim is never
   written; it is *extracted* at the moment something depends on it or kills it.
   The graph's population rate is the rate of rulings and reuse, not the rate
   of writing.
7. **Schema capped, grown by seam.** Six node types, eight edge types, until a
   real query hurts. A new type must displace prose that is demonstrably
   failing, not anticipate it.
8. **The tool must stay small.** quarry's job is to make process rules
   *deletable*. If quarry's own doctrine grows, quarry is failing.

## 3. Storage layout (host repo) — [proposed]

```
graph/
  nodes/
    area/      ar-xxxx-<slug>.md
    item/      it-xxxx-<slug>.md
    thread/    th-xxxx-<slug>.md
    decision/  dc-xxxx-<slug>.md
    claim/     cl-xxxx-<slug>.md
    doc/       do-xxxx-<slug>.md
  log/
    2026-08.jsonl        # append-only event shards, monthly
  .index/                # gitignored; rebuilt by `q reindex`
  view/                  # gitignored; rendered by `q view` / `q handoff`
```

- **IDs** are immutable: type prefix + 4 base32 chars, collision-checked at
  mint. Slugs are mutable labels; renames leave an alias. Prose wikilinks may
  use slugs; verbs resolve and store IDs.
- One node = one file: YAML frontmatter (machine surface) + markdown body
  (human surface). Diffable, greppable, mergeable.
- The **log** is the event stream: every mutation appends one JSON line
  (node, version, op, from/to, actor, note, ts). Per-node history is a filter
  over the log — no git archaeology required. Monthly shards keep files small.

## 4. Node types — [proposed inventory; the concept set is ratified]

| type | is | key fields beyond the common set |
|---|---|---|
| `area` | subject vocabulary — systems, domains, the coarse attachment surface | charter body |
| `item` | unit of work — arc, feature, slice, debt, process task | `kind`, `status` (sketch/shaped/ready/in-flight/done/dropped; **blocked is derived, never stored**), `acceptance[]`, `write_set[]` |
| `thread` | an open strand needing the user — a fork to pick, a discussion to have, a topic that may spawn intermediate work (a spike, a research task) before it can resolve | `status` (open/queued/resolved/parked) |
| `decision` | a ruling | `status` (in-force/superseded), `ratified{by, date}` |
| `claim` | a falsifiable statement something leans on | `status` (asserted/measured/ratified/refuted), `method` |
| `doc` | registered prose artifact — journal entry, spike report, audit, brief | `kind`, optional `path` into the host repo (body may live at the path — journals stay where they are) |

Common fields: `id`, `type`, `title`, `v` (monotone integer), `status`,
`provenance` (user/assistant/measured/external), `created`, `actor`, `edges[]`.

Example:

```yaml
---
id: cl-9x2m
type: claim
title: Bound-water halo is 4–11 cells at post-edit budget
v: 3
status: measured
provenance: measured
created: 2026-08-07T18:40:00Z
actor: claude-fable-5
edges:
  - {rel: source,  to: do-s11r,  at: 1}
  - {rel: about,   to: ar-water, at: 2}
  - {rel: about,   to: "file:crates/dc-worldgen/src/water/body.rs", at: "b3f9c2ae01d4"}
---
Measured in S11: geometric decay, gone by d8; player-visible halo 0–6 cells.
Method: matched-step differencing against drainage-pinned boundary, 15 configs.
```

Notes:
- **Stubs / loose ends** are `item` with `kind: debt` plus an `about` file-ref
  to the site and a `depends-on` to the heir. No dedicated type.
- **Journal entries** register as `doc` nodes wrapping their existing path —
  the journal's narrative form is untouched **[ratified]**; it gains edges.
- **Threads, not questions** (user, 2026-08-07). The queue's unit is whatever
  needs the user's input — as small as picking A/B/C, as large as a discussion
  that spawns a spike before it can be answered. A thread `depends-on` the
  intermediate items it spawns, and only surfaces as answerable once they land.
- **Upcoming work enters as `sketch`** — anticipated relationships recorded as
  ordinary edges, details refined (bumping `v`) as threads resolve. `ready` is
  the stored intent; whether anything still blocks it is always derived.
- Lifecycle distinction that earns `claim` its own type: a decision *could have
  gone otherwise* and is **superseded**; a claim has a truth-maker outside
  anyone's will and is **refuted**. Queries against unverified assistant
  assertions depend on this split.

## 5. Edge types — [proposed inventory]

| rel | from → to | meaning |
|---|---|---|
| `about` | any → area \| file | subject attachment (claims require ≥1) |
| `part-of` | item → item, area → area | hierarchy; a slice names its arc |
| `depends-on` | item \| thread → item \| thread \| decision | blocked-by while the target is unlanded/unresolved — an item waiting on an open thread, a thread waiting on a spike |
| `settles` | decision → thread | the ruling that closes a thread |
| `supports` | claim \| doc → decision \| item \| claim | evidence leaned on |
| `refutes` | claim \| doc → claim \| decision | falsification |
| `supersedes` | X → X (same type) | replacement; target status flips |
| `source` | claim → doc | where the claim was extracted from |

- Edges are stored **on the source node only**; backlinks are an index query.
  A one-directional pointer is fine when the inverter is free — the disease was
  one-directional pointers with no inverter.
- Every edge carries `at`: the target's `v` at write time, or a file's
  abbreviated blob hash. Stamping is automatic; authors never supply it.
- Edges are identified (src, rel, dst); mutations to them (affirm, retire) are
  log events.

## 6. Versioning — [ratified concept; proposed mechanics]

- `v` bumps on any **content** mutation: title, status, body, acceptance,
  ratification. It does not bump on aliasing or index-only changes. Bumps
  happen only inside verbs; each writes a log event with actor + note.
- **Staleness is a query, not a sweep.** An edge is *behind* when
  `edge.at < target.v` (or the file's current blob differs). Severity is
  derived, worst first:
  1. target **refuted/superseded** — every leaner is enumerable (`q blast`);
     this replaces corrections.md and the banner-stamping cascade entirely;
  2. target content changed — review wanted;
  3. file drifted — the ref's blob no longer matches.
- **`q affirm`** re-stamps an edge at the target's current version after a
  human or agent has actually re-read it — reviewing is cheap, and the review
  is *recorded*, so "behind" never becomes ambient noise that gets ignored.
  **RATIFIED 2026-08-08 (user, settles the affirm-ripple thread): an affirm
  that only restamps does not bump `v`.** A restamp is bookkeeping, not
  content — review must never cascade review. This narrows "every content
  edit bumps" by one clause; the affirm is still logged.
- **Mutating verbs report the homework an action creates**, in the verb's own
  output: citers put behind (with the affirm command that clears each after
  review), work unblocked, threads made answerable, sources newly blocked.
  Consequences arrive in-loop, programmatically — never via primed context.
- **Rendered context interleaves the delta.** `q open` shows each behind-edge
  with the log notes between `at` and `v`: *"you cite this at v2; v3: status
  asserted → refuted (S14 measurement)."* The reader learns what changed at
  the moment of reading, which is the moment it matters.

## 7. File references — [proposed]

An edge may target `file:<path>` (optionally `file:<path>:<line>`) in the host
repo. At write time the verb stamps the file's current git blob. Consequences:

- The *original referent is recoverable forever* — the blob resolves the
  content the ref meant even after the file changes beyond recognition; a line
  number is meaningful relative to its blob permanently.
- Drift is detectable: current blob ≠ stamped blob → behind, with
  `git diff <blob>..<current> -- path` available as the delta.
- This is the direct kill for the stale-`file.rs:342` class of rot.

## 8. Constraints (denied writes) — [proposed]

| # | rule |
|---|---|
| C1 | a claim with no `about` edge is rejected |
| C2 | a claim with no `source` is rejected unless provenance is `user` |
| C3 | an assistant-provenance decision may not `settle` or `supersede` a user-provenance target; the verb refuses and points at the queue (a contest is surfaced as a thread, never written) |
| C4 | `supersedes` across types is rejected |
| C5 | a new edge to a refuted/superseded target requires `--acknowledge` |
| C6 | freehand Write/Edit under `graph/` is denied by the host hook; only the tool mutates graph files |
| C7 | reserving a `write_set` overlapping a live reservation is refused with the holder named |

C3 is the mechanization of the costliest deepcraft failure class (a user
design silently reconciled away by an implementation slice).

## 9. Verbs — [proposed]

v0 (the walking skeleton):

```
q init                      bootstrap graph/ in a host repo
q new <type> "<title>"      mint a node   (--about, --kind, --provenance, --body-file)
q link <src> <rel> <dst>    add an edge   (stamps automatically)
q set <node> k=v            mutate fields (status, title, ...) — bumps v, logs
q edit <node>               body edit via file handoff — bumps v, logs
q open <node>               render the neighborhood brief (context payload)
q rule <thread> "<text>"    mint decision + settles edge (--by user marks ratification)
q claim "<text>" --source <doc> --about <area>     extraction-on-citation
q refute <claim> --by <node>                       status flip + edge + blast list
q affirm <edge|node>        re-stamp after review
q q <query>                 canned queries (§ 10)
q log <node>                per-node history from the event log
q reindex                   rebuild .index from files + log
```

**Built since v0:** `q view` (map-first single-file HTML render; markdown
bodies of path-backed `.md` docs render in-app — source code deliberately
does not) · `q wrap` (boundary-time lint: owed threads, stale refs,
in-flight items, unfiled nodes, last user-provenance write, uncommitted
graph changes) · `q guide` · `q hook guard` (C6) · `q hook orient`
(SessionStart one-liner).

Later, in need-order: `q queue` (single-thread topic queue — push/order/pop
thread nodes) · `q reserve` / `q release` (write-set reservations for
dispatch, the two-parallel-sessions mediator) · `q brief <item>` (generated
dispatch brief: neighborhood as read-first, write-set complement as
do-not-touch, acceptance as RETURN spec) · `q handoff` (rendered session-close
view; replaces the hand-rewritten close block).

**Ownership is data.** What the user owes is exactly the set of queued
threads — if the user owes a call and no thread exists, minting one is the
fix; prose and memory never carry user obligations. C3 is the hard edge
(agents cannot settle the queue); the honesty of `--by user` itself is a
logged, auditable contract rather than a technically enforced one — the
tool has no user identity, so the log trail is the guard.

## 10. Canned queries — [proposed]

| query | answers |
|---|---|
| `ready` | items with status ready, no live blocker, write-set free — "what can I dispatch right now" |
| `queue` | threads awaiting the user whose prerequisites have landed — answerable now |
| `shaping` | upcoming work (sketch/shaped) with the threads and items blocking each |
| `behind` | stale edges by severity (§ 6) |
| `blast <node>` | reverse closure over `supports`/`source` — who leans on this |
| `contested` | assistant-provenance writes touching user-provenance targets |
| `idle` | nodes with no inbound edges after N days — built-and-nothing-calls-it |
| `unverified` | claims with provenance assistant and no `supports`/`method` — the corrections-#101 class, on demand |

## 11. Concurrency — [proposed]

Two main sessions with disjoint concerns are the supported topology
**[ratified]**. Node files have single-writer character in practice (disjoint
write-sets), the log append takes a file lock, and reservations (C7) make the
write-set contract visible instead of conventional. The index is per-checkout
and disposable.

## 12. The UI — [ratified as a goal; design deferred]

A generated, browsable view of the whole graph — areas as the map, items and
questions as the live state, staleness severity as visual weight. Rendered
from the same index the queries use; read-only; regenerated on demand. First
version can be a static HTML export; nothing in the storage design constrains
it.

## 13. Adoption and bootstrap — [proposed]

1. **v0 acceptance test is dogfood:** record *this design's own decisions* —
   the rulings and proposals in this file — as quarry's own `graph/`. The doc
   you are reading then becomes a rendered view, and the first real staleness
   is caught by the first real query.
2. The deepcraft successor starts with `q init` on day zero — the clockwork
   before the engine. deepcraft itself stays a frozen, read-only quarry;
   nothing migrates wholesale; knowledge is extracted on touch, as claims with
   `source` pointing at registered legacy docs.
3. Claude-side integration is thin by design: a hook that denies freehand
   `graph/` writes (C6) and a skill that teaches the verbs. Session workflow
   doctrine shrinks to the ~150 lines of genuine judgment; everything else is
   the tool.

## 14. Open questions — for the user

1. **Implementation language.** Recommendation: **Rust** — single static
   binary (`q`), fast startup for hook-path use, the ecosystem you live in;
   the crate is small enough that the machine's build-serialization constraint
   is irrelevant. Alternative: Python for a faster-to-mutate v0, at the cost
   of a rewrite later.
2. **Edge placement.** Recommendation: frontmatter of the source node
   (readable, diffable, single-owner). Alternative: log-only with files
   holding no edges (purer event-sourcing, worse human readability).
3. **Bump granularity.** Recommendation: every content mutation bumps, and
   `affirm` keeps review cheap. Alternative: distinguish "editorial" from
   "semantic" edits at write time — rejected for now because it reintroduces a
   judgment call into the mechanical layer.
4. **Doc bodies.** Recommendation: journals and spike reports keep their
   existing homes; `doc` nodes wrap paths. Inline bodies allowed for small
   notes.
5. **Binary name.** `q` as the invocation (`quarry` the formal name).

**RATIFIED 2026-08-07 (user):** Rust · edges in source-node frontmatter ·
every content edit bumps · doc bodies stay at host paths · binary `q`. The
thread model (§ 4) replaced `question` the same day, from the user's
description of how their queue actually works.
