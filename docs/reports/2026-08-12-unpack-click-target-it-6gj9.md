# Dispatch report: unpack click-target (it-6gj9)

Agent report, registered verbatim by the dispatcher at harvest.

## What changed

**src/mention.rs** — `unpack_html` no longer delegates to `surface::atom_unpack_html` (which anchored only the bare id). It now takes the surface-owned text register `atom_unpack` (so all content composition — id, label, title, foreign-areas — stays in src/surface.rs per dc-nnf5), HTML-escapes it, transcodes the backticked title to a `<code>` span via backtick parity (replacement carries no backticks back into the scanner), and wraps the ENTIRE expansion in one anchor: `<a class="unpack" href="#/n/{id}">{id} [{type} {status}: <code>{title}</code>{ — areas: …}]</a>`. All cl-qxxp behavior preserved: status always, dead/archived labels, current title, code spans skipped, dangling shapes verbatim.

**src/view.rs** — TEMPLATE CSS: `a.unpack` rule (muted tint from the page's own `--muted` family, radius, padding, `box-decoration-break: clone` so wrapped lines keep the pill shape) and a hover deepening. Stale comment beside the `body_html` call updated.

No other files touched. CLI/brief registers untouched.

## Build and tests

`cargo build --release`: Finished `release` profile in 23.64s — clean, no warnings.

`cargo test --no-fail-fast` raw result lines:

```
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (lib unittests)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (main unittests)
test result: FAILED. 51 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s   (tests/basic.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (tests/surface_lint.rs)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (doc-tests)
```

The single failure is `view_carries_hyperlinked_unpack_and_mention_index` (tests/basic.rs:1439), which pins the OLD id-only anchor markup that this item's acceptance supersedes. tests/basic.rs was outside the lease; the assertion update is dispatcher landing homework.

## Acceptance outcome

Verified in the regenerated graph/view/index.html (131 unpack anchors rendered):

- Clicking anywhere navigates: whole expansion inside one anchor, dead/archived labels and foreign-areas announcements ride inside it.
- Reads as a marked span: `a.unpack` styling present in the emitted page.

## Reflections (agent's own words)

- The dispatcher's orientation placed the change in `mention.rs::unpack_html`, but the old markup actually lived in `surface::atom_unpack_html` — outside the lease. Resolved by rebuilding the HTML register in mention.rs on top of the text-register atom (content ownership stays in surface.rs; mention.rs does only escape/transcode/anchor). Consequence: **`atom_unpack_html` in src/surface.rs is now orphaned** — zero callers, still emits the superseded id-only-anchor markup. Should be deleted or become the whole-anchor renderer; flagged as follow-up.
- The one failing test was foreseeable from the brief itself (it pins exactly the behavior the user asked to change) and the lease didn't cover tests/. Chose to complete the change and report rather than stop, since the RETURN spec accepts by rendered outcome and the failure is a stale pin, not a regression — but a green suite was not achievable within the lease.
- Degenerate case: a node title containing a literal backtick would split oddly in the backtick→`<code>` transcode. The text register is already ambiguous for such titles; if titles ever legitimately carry backticks, the atom's title delimiting deserves a real escape.
- Small pleasure: the replacement string carrying no backticks means the outer scanner's backtick parity is undisturbed by construction — the invariant fell out of the transcode rather than needing a guard.
