# Payload review — it-hjed ("find match hygiene"), first read

## What it contains
- **THE WORK** (2 paragraphs): the bug (substring matching admitted `dc-qhru` into a `cli` search via "click"), and — crucially — the already-settled design from 2026-08-12: tiered output (tier one = id substring + title/body word-boundary; substring-only hits trail as a labeled loose tail, always shown, no flag), the exact word predicate (ASCII alphanumerics are word chars, everything else including hyphens is a boundary), the deliberate divergence from the lexicon's `contains_word`, the declined unification with pointers to the three lexicon items filed instead, and the instruction that hit lines ride `atom_line`.
- **READ-FIRST**: the whole `cli` area neighborhood — 7 open threads, 10 decisions in force, ~25 spine claims, ~16 doc reports.
- **WRITE-SET**: `src/**`, `tests/**`. Clear and sufficient.
- **RETURN SPEC**: two crisp, outcome-shaped acceptance lines. Good.
- **PROTOCOL**: machine build rules (one cargo at a time, TMP redirect, read the raw `test result:` line). Genuinely useful, right length.
- **ACTOR RULES**: boilerplate but short, and the "never flip done" line is worth repeating every time.

## What is relevant
THE WORK paragraph is excellent — close to self-sufficient. It carries the design decision, the predicate spec, the divergence rationale, and the rendering constraint. If the whole brief were just that plus WRITE-SET, RETURN SPEC, and PROTOCOL, I could build from it.
From READ-FIRST, only a handful of entries earn their bytes for this task: dc-nnf5 (the surfacing atom — my hit lines must ride atom_line), cl-89b5 (find-areas — find already renders via atom_line), cl-3qwu (surface-atom — one module owns rendering), cl-syj7 (atom-lint — I must not touch front.title outside src/surface.rs). Maybe dc-wwnk/cl-qxxp as context for why bodies are full of ids that a substring match can trip on.

## What reads as noise
Roughly 80% of the payload. The dispatch-machinery corpus (join-as-fetch, multi-held dispatch, fire-time routing, kind-shaped wake, boundary-harvests-all, work-only-stamping, charter-at-wake...) is the history of the *tooling that delivered me here*, not of the work. Sixteen doc-report lines are pure ballast — none will be opened. Seven threads all marked "NOT yours to settle" — if they're not mine to settle and none of them is about find matching, why enumerate them? The spine shelf is dumped whole (25 claims) when perhaps 4 are load-bearing for this item. The brief even cites th-uacu ("the brief is a dump") in its own noise — the graph knows.

## What I wish it told me
- **Where find lives.** No file path. The write-set says `src/**` but the brief never says which module implements `q find` or where the current matching loop is. First minutes of the work will be re-deriving what the dispatcher already knows.
- **Where the lexicon's `contains_word` lives**, since I must write a divergence comment "beside its definition" referencing it — a file:line would have made that a one-touch edit.
- **What "id substring" means at tier one, precisely.** The query matching inside an id (e.g. `cli` inside some random id charset)? Or only prefix/id-field matches? The settled sentence reads "id substring plus title and body word-boundary hits" — I'll take it literally (substring anywhere in the id stays tier one), but a one-line example of a tier-one vs loose hit would have removed the ambiguity.
- **The desired loose-tail label text/register.** "The loose label belongs in the atom's register, not a raw print" — fine, but is there a settled wording ("loose:", "substring only", "matched loosely")? I'll have to pick one and flag it.
- **Whether existing find tests exist** and where — a pointer to the test file would beat me globbing for it.

## Net
High-quality core (THE WORK + RETURN SPEC are the best-written parts), buried in an area dump that scales with the area's history, not with the item's needs. The signal-to-noise problem is exactly the one th-uacu already names. What's genuinely missing is code geography: two file paths would have saved more time than the 60 lines of decisions cost.
