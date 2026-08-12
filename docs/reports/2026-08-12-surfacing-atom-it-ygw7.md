# Dispatch report: the surfacing atom lands (it-ygw7)

Dispatched and harvested 2026-08-12, badge `it-ygw7`, lease `src/**` + `tests/**`
(widened from `src/**` before first write — the acceptance itself demanded test
edits). 14 files touched, all inside the lease. 52 + 1 tests pass, read off the
raw `test result:` lines.

## Outcomes, judged against the RETURN spec

- **`surface-atom` — lands.** `src/surface.rs` owns the `Atom` struct (id,
  title, type, kind, status, v, areas as titles with ids kept beside them,
  provenance, grounding presence for claims, archived) and the three renderers.
  No other module formats a node title; the 96 audited sites across 9 files are
  gone, replaced by calls into the module. `line()` in main.rs survives only as
  a two-line bridge onto `atom_line`.
- **`atom-line` — lands.** Every list register — find, queue, ready, shaping,
  touches, homework, brief shelves (read-first, area, decision, doc, spine,
  evidence), cross-session arrivals, session-resume holdings/acts/arrivals,
  wrap lists, the leaseless nudge, and the SessionStart hook enumeration —
  prints the full atom: kind rides type (`item·debt`), areas render as titles,
  archived rides the status slot, grounding rides provenance
  (`assistant·sourced`). The hook's owed-threads list moved from inline
  comma-joined titles to one atom_line per thread under the summary counts.
- **`atom-ref` — lands.** Refusals (C3, C5, C8, archive, dispatch, lease,
  queue), confirmations (lease, release, refute, queued/front/drop), and
  composed sentences (blocked-on notes, danglers, unharvested dispatches,
  landed-uncited, contested, intent-delta lines) carry at least
  `"title" [type status] (id)`. The audit's named id-less offenders —
  cross-session arrivals, lease confirmations, the landed-uncited lint, most
  refusals — all carry ids now.
- **`atom-unpack` — lands.** `mention::unpack`/`unpack_html` build their
  expansions from the atom and take the citing node; a cited node whose areas
  reach outside the citing node's announces them
  (`th-yzmj [thread queued: \`…\` — areas: cli, process]` seen live from
  it-ygw7, whose only area is cli), home mentions stay quiet, verified in the
  open render and covered by tests.
- **`atom-lint` — lands, totalized.** Rather than distinguishing "formatted"
  from "read", the ban is total: every `front.title` token in src/ lives in
  surface.rs — matching rides `title_raw`, the one mutation (`q set title=`)
  rides `retitle`. `tests/surface_lint.rs` fails CI on any new token;
  `q wrap` runs the same scan (`surface::lint_sources`) when the graph root
  carries `src/surface.rs`, so host-repo wraps skip it naturally.
- **`carrier-replumb` — lands.** `queries::Behind` holds `src: Atom` and
  `to_atom: Option<Atom>` (files and danglers stay path-described); homework,
  wrap, the brief behind-check, and `q query behind` print from the atoms.
  Protocol inline/gate carriers now carry the rendered atom_ref as their
  header. Deliberately NOT replumbed: `Lease`/`DispatchState` keep their
  stored `item_title` field — they are machine-local persisted state (schema
  break mid-arc would have orphaned the live lease), and every print site now
  resolves the node to an atom at render time, falling back to the stored
  title only when the node is gone.
- **`find-areas` — lands.** Find hits are atom_lines; the deepcraft-salvage
  hits in this session's own queue arrive labeled with their foreign area —
  the dc-qhru incident class is closed at the display layer (match hygiene
  itself remains the separately-filed defect it-hjed).

Spine claims minted under the badge, sourced to the code: cl-3qwu
(`surface-atom` + the three renderer names), cl-syj7 (`atom-lint`), cl-uqqn
(`carrier-replumb`), cl-89b5 (`find-areas`) — the intent delta for these
names closes.

## Reflections

- **The fence caught its builder, again.** The wrap-half warning message
  originally contained the literal token `front.title` — the lint's own text
  tripped the lint. Reworded. Same incident class as the fast-follows report's
  fence; a string-scan lint will bite any surface that talks about itself.
- **Total ban beat judgment ban.** dc-nnf5 says "formatted outside the module
  fails"; a grep cannot see "formatted", so the honest mechanization was to
  banish the token entirely and give non-render paths named accessors. The
  side effect is pleasant: `title_raw` at a call site is now a visible
  declaration that a use is deliberately below the atom.
- **Sentence floors don't need the graph.** `atom_ref` renders no areas, so
  refusal sites without `all` in scope build their atom from `&[]` — correct
  output, no load. If atom_ref ever grows an areas segment, those sites start
  lying; the lint will not catch that drift.
- **atom_line is long.** A queue line now carries kind, areas, provenance,
  and id. On this graph it reads well; on a graph with many areas per node it
  may crowd. The ruling says append-never-subtract, so any trim is a future
  ruling, not a builder call.
- **The open header stayed richer than the atom** (actor, created, ratified,
  path, method live only on the node); `atom_head` therefore takes the Node,
  not the Atom. That is a fourth register shape in the surface module —
  within the ruling's letter (one module owns all registers), worth knowing.
- **Doubt:** `unpack` now loads the citing node's areas per body render —
  O(edges) per call, fine at 150 nodes, unmeasured beyond that.
