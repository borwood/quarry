# Dispatch report — the lexicon trio (it-b5tq, it-nuw5, it-sc2u), one agent, one matcher zone

Agent report, delivered 2026-08-15, registered by the dispatcher for all three items.

## it-b5tq — lands `backtick-floor`

`backticked_spans` filters 2-60 (was 4-60) — the single-line change dc-qvtz ruled. Bare-token floors stand untouched: `sig_tokens` at five, the reverse pass at six. Test `lexicon_backtick_floor_two_chars` measures all three edges: extraction (`cli` joins at three chars, `q` stays below two), relatedness (a body backticking `cli` reaches a cli-titled node — invisible before), and the noise gate (the same words unbackticked join nothing). Vein claim: cl-baxa. One companion change, flagged and dispatcher-endorsed at harvest: relatedness's backtick-vs-title check was raw substring; at a two-char floor that re-admits the cli-inside-click debris species the find-hygiene work evicted — tightened to the word-boundary scan, one line, defending the ruling from itself.

## it-nuw5 — lands `plural-fold`

`contains_word` folds s/es at compare time by trying the word's own +s/+es variants and stripped stems under the boundary scan, both directions: test `lexicon_plural_fold_compare_time` proves watches meets watch (forward) and renderer meets renderers (reverse). Find's predicate stays exact, asserted both directions in the same test. Lexicon side only: intent-delta's name joins stay exact equality — deliberate names don't inflect. Vein claim: cl-rbp9. Noted: the lease/leases census pair cannot transit the reverse pass at five chars (the six-char reverse floor stands by ruling); it is covered on the compare side by the same fold.

## it-sc2u — lands `compound-halves`

`sig_tokens` emits each hyphen compound whole plus its halves of five-plus chars (generic-token and dedup filters apply to halves); `contains_word` adopts find's hyphen-as-boundary edge rule — the two predicates now share one boundary rule, the plural fold the sole remaining documented divergence; relatedness weighs a whole-name hit double a fragment hit in both passes, and the "why" names the compound over the half. Test `lexicon_compound_halves_join_and_outrank` measures: core sample (spaced) meets `core-sample`, the compound-titled node outranks the fragment-titled node, the why says `core-sample`. Live confirmation at mint: cl-2dj3's own touches surfaced the shelf-sees-before-it-ranks decision via a reverse compound hit the old matcher could not see. Vein claim: cl-2dj3.

## Shared

- cl-en99 re-grounded once after all three landed, per dc-qvtz: method re-run green over the new matcher, source edge restamped clean.
- The three vein claims minted kindless (the it-pgn9 hole, met live) and were explicitly kinded vein.
- q serve stopped/rebuilt/relaunched for the exe lock; probe answered 200 — second occurrence of the serve-lock snag.

## Reflections (agent), with dispatcher notes

- Multi-join stamping: one agent bound three badges in sequence; every file write and all 11 graph acts stamped under the last-joined badge (it-sc2u) — harvest confirmed the first two badges observed nothing but their joins. The one-agent-many-dispatches shape wants a stamping story; filed as a bug at harvest.
- The reverse pass's six-char floor makes the lease/leases pair untestable through relatedness in one direction — floors interact with the fold in ways the census framing didn't surface; worth knowing when reading future misses.
- leases never meets leaseless: derivational morphology, out of s/es scope — if that miss mattered, it silently still does.
- The doubled compound weight is a chosen constant, not measured; magnitude is taste until a real ranking failure calibrates it.

## Dispatcher verification at harvest (2026-08-15)

Full suite re-run by the dispatcher's own hand: raw test result lines read 69 passed and 1 passed, zero failed. Diff spot-checked: the 2-60 filter, the fold variants under find_word, the shared boundary rule and documented divergence comments, the doubled compound weight, the tick check on the word-boundary scan. Claim kinds and the cl-en99 restamp confirmed in the graph.
