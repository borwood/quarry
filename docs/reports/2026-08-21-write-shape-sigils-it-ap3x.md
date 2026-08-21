# Dispatch report — it-ap3x: the write-shape parser accrues sigils

*Registered by the dispatcher on the agent's behalf: `docs/reports/**` sat
outside the arc's lease, so the report could not be filed from the agent seat.
The words below are the agent's. See the friction note at the end — this is a
recurring gap in the dispatch shape, not a lapse.*

**Outcome: acceptance met.** *"no bare sigil or non-path token from command
text accrues as a touched path"* — pinned by a new instrument and probed both
ways.

## The mechanism (it was not a bare `@`)

The actual debris in `graph/.touched.jsonl` was
`{"path":"@ 2>&1 | tail -20","via":"shell"}` — the whole command tail, not a
lone sigil. The chain:

1. This repo hands git multi-line commit messages through a PowerShell
   here-string (`git commit -m @'…'@`), which its own build notes teach.
2. `shell_tokens` had no here-string concept. The apostrophe in the message
   prose (`main.rs's Join arm`) **closed** the quote the `@'` opened; the
   terminating `'@` **re-opened** it. Everything after the literal was then
   read inside a quote that never closes.
3. The `>` closing the `Co-Authored-By: … <noreply@anthropic.com>` address
   inside the message emitted a redirect token, and the single swallowed word
   behind it — `@ 2>&1 | tail -20` — was pushed as its target.
4. `opaque_target` had no plausibility test, and `observe_shell` resolves
   store-relative by path joining alone (nothing on disk answers for it), so it
   accrued clean.

## The fix (`src/teach.rs`, two independent halves)

- **`shell_tokens`**: a here-string opener at the start of a word (`@'` or
  `@"`) is consumed whole through its line-initial terminator as one opaque
  token. Unterminated literals end the parse quietly instead of desyncing
  everything behind them; writer args beside a here-string value stay readable.
- **`opaque_target`**: refuses two shapes no write target carries — a token
  holding a control char, a quote, or a metacharacter the tokenizer would have
  split on in a command (`<>|;&`), and a sigil-only token with no alphanumeric
  at all (`@`, `--`, `{}`). Plausible paths untouched: spaces and Windows
  separators still pass.

Dropping is the cheap direction and matches cl-up6s's stated posture; a false
positive is the expensive one, since the harvester cannot tell it from a real
write.

## Instruments

- `tests/basic.rs::write_shapes_drops_parser_debris_and_sigils` — the incident
  command verbatim in shape, the sigil/metachar/control shapes, unterminated
  literals, plus the plausible targets that must survive.
- An added arm on `observe_shell_accrues_only_resolvable_targets_marked_shell`
  carrying *why* the drop must happen at the parse: debris resolves
  store-relative by joining, so nothing downstream would catch it.
- **Probed both ways**: disabling the here-string arm fails the "a real
  redirect beside a here-string still parses" assert (got `[]`, wanted
  `["build.log"]`); disabling the plausibility clause fails the bare-sigil
  assert.

`test result: ok. 130 passed; 0 failed` (basic) — plus 2, 1, 1, 1, 3 across
broken_pipe, observed_set, store_lint, surface_lint, worktree_proof. All
zero-failure, read raw. Release rebuilt so the hook path is live. No
`q init --claude` needed: no guide/skill/hook-wiring text changed.

*Dispatcher's note at landing: the suite was re-run from the judge seat and
read raw — nine `test result: ok` lines, basic at 130 passed.*

## Graph writes

- **cl-zj2c** — `parse-plausibility`, kind vein, about ar-c7f5, sourced on
  `file:src/teach.rs` and `file:tests/basic.rs`, `supports → it-ap3x`.
- **Affirmed cl-up6s** `--to file:src/teach.rs` after re-reading it. Its
  accounting invariant holds unchanged; this is a floor under its shell
  channel, not a correction to it. Its `src/render.rs` and `src/coord.rs`
  source edges remain drifted — **not mine, not reviewed, deliberately not
  affirmed.**

## user-owned calls

none.

## Reflections

- **The stale entry survives.** `graph/.touched.jsonl` still carries the
  `@ 2>&1 | tail -20` row under `session:dispatcher` at 09:03:27. It is
  gitignored machine-local state and outside the write-set, so it was left. It
  will render at the next dispatcher-session surface until pruned.
- **One residue that could not be closed, same class, one channel over.** A
  bash heredoc body (`<<'EOF' … EOF`) still tokenizes as ordinary command text
  — the delimiter is never tracked, so a `<` boundary drops in and the prose
  becomes command words. The plausibility floor catches its debris *shapes*,
  but a heredoc line reading `touch foo` would still mint `foo` as a plausible
  false positive. Closing it wants the same delimiter-tracking move made here
  for the PowerShell form. No thread was filed for it (it is a defect, not a
  user-owned call, and a stray badge thread would skew the harvest
  reconciliation) — flagged here for the dispatcher to file.
- **Friction: no report could be registered.** `docs/reports/**` is outside the
  lease, so the doc node that `latest_report_doc` joins to the harvest seat
  cannot come from the agent seat — the "user-owned calls: none" declaration
  lives only in the returned message, where `declared_user_owned_calls` cannot
  parse it. Harvest shows `user_owned_await`. Either the lease should include
  the report path on bug dispatches, or the reconciliation should have a second
  source.
- **Friction: quoting a claim body.** Vein bodies want backticks *and*
  apostrophes, which no single shell quoting form passes cleanly. The title and
  body were staged as scratchpad files and passed via `"$(cat …)"`. That
  worked, but it put two `unresolved:true` scratchpad rows in the badge's
  observed set — correct behavior per cl-up6s, and harmless, but it means the
  accounting mechanism reliably logs noise for anyone who mints a well-formed
  vein. A `--body-file` flag on `q claim`/`q edit` would remove both problems.
- **Surprise worth noting:** the plausibility filter alone would have caught
  this incident, and so would the here-string arm alone. Both were kept because
  they fail differently — the filter is a backstop against debris *shapes*, the
  tokenizer arm prevents the desync that manufactures them. A desync can still
  produce a perfectly plausible-looking target, which is the failure mode the
  filter cannot see.
