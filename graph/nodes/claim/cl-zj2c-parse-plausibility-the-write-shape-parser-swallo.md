---
id: cl-zj2c
type: claim
title: '`parse-plausibility`: the write-shape parser swallows here-strings whole and refuses targets that cannot be filenames'
v: 7
status: ratified
provenance: assistant
created: 2026-08-21T09:16:17Z
actor: claude-fable-5
kind: vein
ratified:
  by: claude-fable-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/teach.rs
  at: b4c33066b79b
- rel: supports
  to: it-ap3x
  at: 4
- rel: source
  to: file:tests/basic.rs
  at: 8de5dd671593
- rel: supports
  to: it-dt68
  at: 5
---

`parse-plausibility`: the write-shape parser swallows here-strings whole and refuses targets that cannot be filenames

The tokenizer-edge floor under the shell channel of cl-up6s (it-ap3x): the write-shape parse is a guess made over text the tokenizer can read WRONG, so the guess is now filtered at both ends. Neither half denies anything — the channel stays observation-only in the commit-sweep guard's posture.

AT THE TOKENIZER: teach::shell_tokens consumes a PowerShell here-string — @ then a quote, through its line-initial terminator, the @" form as well as @' — WHOLE, as one opaque token. This repo hands git its multi-line commit messages that way (its own build notes teach the form), and the body is prose: an apostrophe in it, "main.rs's Join arm", closed the quote the opener began, and the terminator re-opened it, so every character after the literal was read inside a quote that never closes. The command's real tail arrived as one bogus word, and the > that closes a Co-Authored-By <address> inside the message pushed that word as a redirect target. That is how a literal "@ 2>&1 | tail -20" reached OBSERVED vs LEASED at the it-csm3 harvest. An unterminated literal now ends the parse quietly instead of desyncing everything behind it, and the writer arguments beside a here-string value stay readable.

AT THE TARGET: teach::opaque_target refuses two shapes no real write target carries, beside the streams, devices, globs and substitutions it already refused. First, a token holding a control character, a quote, or a shell metacharacter the tokenizer would have split on had it been reading a command (<>|;&) — its presence is the tell that the token is a fragment of something else. Second, a sigil-only token with no alphanumeric character in it at all: a bare @, --, {}. Plausible paths are untouched — spaces and Windows separators are ordinary in a path and cost nothing to keep.

Dropping is the cheap direction here, the posture cl-up6s already set (a parse is a guess; only the tool-write channel records unresolved paths, because there the write is a certainty) and framings::SIGHT_BOUNDARY states the residue honestly. A false positive is the expensive direction: it reads at the judgment seat as a write nobody made, and the harvester cannot tell it from a real one.

The two halves are independent — either alone kills this incident — and each was probed against the regression it guards: disabling the here-string arm fails the assert that a real redirect beside a here-string still parses; disabling the plausibility clause fails the bare-sigil assert.

Pinned by write_shapes_drops_parser_debris_and_sigils (the incident command verbatim in shape, the sigil and metacharacter shapes, the unterminated literal, and the plausible targets that must survive) and by an added arm of observe_shell_accrues_only_resolvable_targets_marked_shell, which carries the reason the drop must happen at the PARSE and not at resolution: debris resolves store-relative by mere path joining, so nothing downstream would have caught it.

AMENDED AT THE it-dt68 LANDING (2026-08-21, the dispatcher's hand): the heredoc residue this paragraph used to name as open is closed. cl-p4k2 tracks the delimiter from the opener and consumes a bash heredoc body — bare, quoted, and tab-stripped forms alike — as one opaque token, taken at the newline that ends the opener line so a redirect beside the delimiter still belongs to its command. It was the same class one channel over, and worse than predicted: measured pre-fix, the repo's own commit road (git commit -F - with a heredoc) minted the terminator EOF as a touched path, the it-ap3x incident replayed verbatim in the channel this repo moved to BECAUSE of it-ap3x.

The residue that remains is narrower and named: it-dprv — a newline is not a command boundary in shell_tokens, so a multi-line command reads as one command. Both directions are measured there: a later line's command word mints as a plausible path (the floor cannot catch it, "touch" being a legal filename), and a real write on a later line goes unseen. The cure wants the newline boundary plus a line-continuation exception, which is why it was filed rather than folded in here.
