# The deepcraft process autopsy

Written 2026-08-08 from the survey that founded quarry (conducted 2026-08-07
in the founding conversation, reading the deepcraft corpus directly). This is
the migration's read-first: what was diseased, what was sound, and the
mapping from each scar to the quarry mechanism that replaces it. Deepcraft
itself: `B:\repos\borwood\deepcraft`, a Rust/Bevy voxel game — an engine
whose content is plugins, its earth-science pack first among them. The
codebase is largely healthy; the KNOWLEDGE LAYER is what this documents.

## The measurements

- 814 markdown files, ~49,800 lines of docs, against ~128,000 lines of Rust.
- `CLAUDE.md`: 549 lines, every rule carrying its origin story, corrections
  to corrections, warnings that earlier warnings were false.
- `.claude/skills/session-workflow/SKILL.md`: 1,389 lines of accreted
  incident doctrine. `wrap`: a 13-step hand-executed close ritual.
- `journal/corrections.md`: 4,152 lines, 122 falsified claims in ~7 weeks.
- `ROADMAP.md`: 5,663 lines. `spines.md`: 2,559 lines.
- Near-daily hygiene passes: roadmap staleness sweeps, doc-topology sweeps,
  spine audits (see `docs/audits/` — the immune system cost as much as the
  organism).
- Their own measurement: 8 of 15 correction→file edges one-directional;
  14 of 30 audit/spike files carrying no staleness banner. Their own words:
  "a one-directional pointer is not a pointer."
- Conventions asking authors to remember died on contact: `JUSTIFIED-BY` was
  documented in two places with a promised sweep — 3 uses, 0 in crates/.
  The one invariant that stopped failing was the build mutex, the day it
  became a hook. Enforcement beats doctrine, measured.

## The disease taxonomy → the quarry mechanism

| deepcraft scar (worked case) | mechanism | quarry answer |
|---|---|---|
| "Read one source, assert about another" — corrections #101, five instances in ONE session | claims duplicated as prose in N places; no single authority | one authority per fact; derived views re-run queries; `behind` is a query, not a sweep |
| Assistant reconciliation silently overwrote a user design (corrections #65, cost three days, produced `DeepAxis`) | no provenance machinery; ratification protocol only covered recording, not superseding | provenance as a field; C3 denies assistant settling/superseding user provenance; contests surface as threads |
| Stale cross-refs, `file.rs:342` rot, spike numbers cited after refutation | refs carried no version; staleness discoverable only by sweep | every edge stamps target version / file blob; severity-ranked `behind`; `blast` replaces corrections.md |
| Close block said "engine work"; read-first item 0 refuted it by name (corrections #71) | hand-written handoffs = a second authority that drifts | handoffs derived, never written: `q session resume` |
| Bodies session filed findings about geology's expected-red artifacts (corrections #92) | no cross-session presence | purviews, leases, presence notes, arrivals, hook alerts |
| Journal/corrections numbering collisions between parallel agents | id allocation by convention | tool-minted ids |
| Doctrine ballooned because every scar added a rule | rules stored as prose, enforced by attention that degrades over 500k tokens | rules become edges, queries, or denied writes; the skill stays ~100 lines of judgment |
| Sweeps as garbage collection by grep | derived state hand-maintained | ROADMAP/close-block/staleness views all rendered from the graph |
| `roster` skill § 0: "sweep the corpus for rulings newer than this file" | a materialized view maintained by hand | protocol nodes version and stale like all content |

## The session-workflow bucket analysis (1,389 lines sorted)

1. **Invariants that wanted to be mechanical** (majority): numbering, stamp
   the target, one-directional pointers, "excluded is not sequenced",
   defer-=-write-it-now, provenance marking. → all became quarry mechanics.
2. **Brief-template content**: accept-by-outcome, process-not-snapshot,
   "name what the acceptance criterion is NOT", premise-as-hypothesis,
   resource rules. → belongs in the generated dispatch brief (`q brief`,
   unbuilt) and in successor project protocol.
3. **Genuine judgment (~150 lines — the part worth porting as prose)**:
   triage the ratification list (user-owned vs measurable); adversarial
   checkers must not read the analysis they check; an agent's MECHANISM is a
   hypothesis, only measured numbers are evidence (and a number without a
   method is not evidence either); verify the load-bearing claim, not the
   whole report; live tours over screenshot loops; harvest from the diff,
   not the report.
4. **Incident narratives**: valuable history, wrong storage — belongs in
   journals, not process docs.

## What was genuinely sound (port, don't fix)

- **The journal** — narrative memory and blog feedstock; the best artifact
  the project produced. Already adopted into quarry (charter as protocol).
- **Measurement culture** — spikes with methods, literature calibration
  ("a closed system cannot detect its own scale error"), goldens as
  tripwires never intent, accept-by-outcome.
- **Ratification instinct** — became C3 + `--by user` + ratified field.
- **The fingerprints ritual** (session retro → process improvement) — in
  quarry, friction becomes a thread or a protocol edit, same loop, cheaper.
- **The four hats** (design partner / delegator / integrator / archivist)
  and walk protocols — successor project protocol, not tool.

## Migration implications (the standing rulings)

- deepcraft is FROZEN as a read-only quarry: never swept again, never made
  consistent again. Queried like literature.
- **Migrate on touch, extract by seam.** Nothing ports wholesale. Code that
  passes the subtraction test (the correlation kernel, the water body graph)
  moves with a node recording why it qualified; knowledge is extracted as
  claims/decisions citing registered excerpts when work actually touches it.
- The successor starts with `q init --claude` on day zero — clockwork before
  engine. Its first protocol entries: the journal charter (with deepcraft's
  four lenses and blogworthy flags, if the user carries that ambition over),
  the walk protocol, the dispatch-brief house rules.
- Cross-repo note: `file:` refs blob-stamp only within the host repo —
  deepcraft sources are cited by registered excerpt or doc body, not by
  live path.
