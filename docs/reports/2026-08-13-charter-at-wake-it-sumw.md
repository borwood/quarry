# Dispatch report — it-sumw: orientation consumes the charter

## Against the RETURN spec — `charter-at-wake` lands

Both wake choke points now render the registered charter, same text, one renderer:

- **`q session resume`** prints `charter: <text>` beneath the purview line. The charter previously rode inline on the `resuming session` header; it moved beneath the purview per the item text, so the kind reads as its own line at wake.
- **SessionStart orient** renders the same `charter:` line beneath the `session <name> purview (...)` line. To make this true *for a bound chat* (not just launcher-env chats), the orient hook now resolves its wake session env-first, then via the chat binding — an adopted chat previously got **no session line at all** at SessionStart; now it orients as its session, charter included. This is the one structural change beyond a print statement.
- **One capability**: both surfaces call the new `coord::charter_line(&Purview)` (`src/coord.rs`), so the text cannot drift between renderers.
- **No-charter sessions render exactly as today** — verified for resume, orient-with-env, orient-unbound (multi-session warning intact), and orient-unregistered (`no registered purview` message intact), all demonstrated in a scratch store.

Verification: full suite read off the raw result lines — `60 passed; 0 failed` (basic, including new `charter_at_wake_line`), `1 passed; 0 failed` (surface lint), `0 failed` everywhere else. Release binary rebuilt. Live proof on this repo's own graph: the quarry session's orient now prints its design-session charter — the exact session whose 2026-08-13 wake demonstrated the gap.

Graph: spine claim cl-mk8b (`charter-at-wake`) minted under the badge, sourced `file:src/main.rs`, `supports dc-ydvb` with a note naming it the consumption. Diff confined to `src/coord.rs`, `src/main.rs`, `tests/basic.rs` plus q-verb graph writes; the `graph/sessions.json` charter and pre-join log lines in the working tree are the dispatcher's, not the agent's.

## Provisional calls (none contradict rulings)

1. **Bound-chat orient scope**: resolving the binding means an adopted chat's orient now also renders the purview counts, foreign-lease, and new-in-purview lines — the whole session block, not just the charter. Judged inseparable from "the orient line for a bound chat carries the same text"; flagged in case the dispatcher wanted charter-only. Dispatcher's judgment at harvest: accepted — an adopted chat orienting as its full session is the intent of adoption, not scope creep.
2. Orient's charter line indents two spaces under the session line, matching resume's register, though other orient lines are flush-left. Accepted.

## Reflections (agent's own)

The item read like two println!s and mostly was — the real work was noticing the bound-chat hole: `coord::purview` was env-only, so the adopt path never met the orient session block at all, charter or not. If the delivery had been "add a line where the session line already prints," an adopted design session would still wake kind-blind and the demonstrated gap would only half-close. Friction worth mining: resume and orient each hand-roll session resolution slightly differently now (resume demands env, orient takes env-or-binding) — a `wake_session` resolver in coord would unify them, but resume's env-only strictness looked deliberate (launcher-owned identity, dc-8mf2) so it was left. Doubt: `charter_line` prefixes `charter:` inside coord while indentation stays per-surface — a half-owned render that would irritate the surface-atom lint if charters ever become node fields rather than registry state.

## Harvest verification (dispatcher)

Test suite re-run at harvest by the dispatcher, result lines read raw: `60 passed; 0 failed` and `1 passed; 0 failed`, all other harnesses `0 failed`.
