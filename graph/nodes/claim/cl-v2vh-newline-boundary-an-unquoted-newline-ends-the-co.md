---
id: cl-v2vh
type: claim
title: '`newline-boundary`: an unquoted newline ends the command before it; the continuation exception is one word wide'
v: 4
status: ratified
provenance: assistant
created: 2026-08-21T11:38:33Z
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
  at: d15f00a82b95
- rel: supports
  to: it-dprv
  at: 4
---

`newline-boundary`: an unquoted newline ends the command before it; the continuation exception is one word wide

The last tokenizer-edge hole under cl-zj2c and cl-p4k2 (it-dprv). teach::shell_tokens read a newline as ordinary whitespace — the `c if c.is_whitespace()` arm stood ahead of the separator arm, so `\n` flushed a word and never pushed a boundary. A multi-line command was therefore ONE command, and only its first word was ever read as a command word. Both error directions followed, measured on write_shapes before the fix.

FALSE POSITIVE: `touch src/a.rs` + newline + `touch src/b.rs` parsed to ["src/a.rs", "touch", "src/b.rs"] — line two's command word read as an argument of line one and minted as a touched path. cl-zj2c's plausibility floor cannot catch it ("touch" is a legal filename) and neither can resolution (a bare word joins store-relative), which is why the drop belongs at the parse. Worse in the shape end to end: `touch src/one.rs` + newline + `cp fixtures/a.rs src/two.rs` accrued THREE phantom paths, "cp" and "fixtures/a.rs" beside the real one, every one of them resolving. And a redirect left dangling at a line end took the next line's command word outright — `cargo build >` + newline + `touch src/a.rs` parsed to ["touch"] and nothing else.

FALSE NEGATIVE: `cargo build > build.log` + newline + `cp fixtures/a.rs src/a.rs` parsed to ["build.log"] alone — the cp was consumed as an argument of line one and never scanned as a command, so a real write went unseen and the sight boundary widened by a whole line.

THE BOUNDARY. An unquoted newline now flushes and pushes a boundary exactly as `;` does; the arm sits ahead of the whitespace arm and behind the heredoc arm that already claimed its own newline (cl-p4k2's ordering is untouched — a body is still taken at the newline ending its opener line). A quoted newline is not a boundary at all: the quote branch runs before the match, so prose inside an argument stays one token and the it-ap3x floor keeps holding. The boundary can only REDUCE false positives, never add one: prose reaches this parser only inside quotes, a here-string, or a heredoc body, and all three are consumed whole before the newline arm sees them — so a line whose first word is `touch` is a touch.

THE CONTINUATION EXCEPTION IS ONE WORD WIDE. This tokenizer does no escape handling, so a naive boundary loses a wrapped command's tail: `cp fixtures/a.rs \` + newline + `  src/b.rs` parses to [] with the destination gone. A bash trailing `\` or a PowerShell trailing backtick STANDING AS ITS OWN WORD is therefore deleted, exactly as bash removes a backslash-newline pair, joining the tail to the command in front of it.

Standing as its own word is the whole of the narrowing, and it was settled by measurement, not taste: `\` also ends a Windows directory path and both shells share this parser. A rule reading ANY trailing `\` turns `Copy-Item a.rs B:\dest\` + newline + `touch src/b.rs` into the single target ["B:\desttouch"] — a plausible path nobody wrote, the expensive direction cl-zj2c named — and loses line two's real write with it, where the word rule reads both correctly. It also buys nothing for the glued form it would cover: `cp a.rs\` + newline + `src/b.rs` parses to [] under both rules, because bash's own joining makes that destination "a.rssrc/b.rs". The glued continuation is the residue, and it costs a lost target, never an invented one.

A CRLF's `\r` is skipped when a newline follows it, so the continuation check reads the real last character of the line. Without that, `\r` flushed the lone `\` as a token of its own and every CRLF-terminated continuation lost its tail — measured, [] where ["src/b.rs"] is right.

Pinned by a_newline_ends_the_command_before_it_unless_the_line_continues in tests/basic.rs — both incident shapes verbatim, three lines, the CRLF line end, the dangling redirect, both shells' continuations in LF and CRLF, the Windows-path negative, the glued-form residue, the quoted-prose control, and cl-p4k2's heredoc control read through the new boundary — and by an added arm of observe_shell_accrues_only_resolvable_targets_marked_shell carrying the end-to-end accrual: two lines, two real paths, no command word among them. Each half probed against the regression it guards: removing the boundary fails both tests (the observed set showing "cp" and "fixtures/a.rs" as touched paths), removing the continuation arm fails with the destination gone, removing the `\r` skip fails the CRLF continuation.

This closes the residue cl-zj2c and cl-p4k2 both name as open in their closing paragraphs; those paragraphs now describe covered ground. Amending a ratified claim's body is not an agent's hand — flagging it is, and this arc's report does.
