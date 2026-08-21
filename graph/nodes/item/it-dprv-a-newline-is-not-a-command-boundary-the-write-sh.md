---
id: it-dprv
type: item
title: 'a newline is not a command boundary: the write-shape parser reads the next line''s command word as the previous line''s argument'
v: 4
status: done
provenance: assistant
created: 2026-08-21T11:22:04Z
actor: claude-fable-5
kind: bug
acceptance:
- 'a multi-line command is read as the commands it is: an unquoted newline ends the command before it, so no later line''s command word mints as a path and no write on a later line goes unseen, while a line continuation keeps its tail'
witness:
- line: 'a multi-line command is read as the commands it is: an unquoted newline ends the command before it, so no later line''s command word mints as a path and no write on a later line goes unseen, while a line continuation keeps its tail'
  by: chat:b8214cb4-bf6e-42fa-9ceb-317d5660ca2d
  session: dispatcher
  kind: dispatch
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
---

Noticed at the it-dt68 heredoc landing and measured on teach::write_shapes directly — filed rather than fixed there, because the cure is not the heredoc cure and the arc was scoped to the heredoc.

teach::shell_tokens treats a newline as ordinary whitespace: the `c if c.is_whitespace()` arm precedes the separator arm, so `\n` flushes a word and never pushes a boundary. A multi-line command therefore tokenizes as ONE command, and only its first word is ever read as a command word. Both error directions follow, measured on write_shapes:

FALSE POSITIVE — `touch src/a.rs\ntouch src/b.rs` parses to ["src/a.rs", "touch", "src/b.rs"]. The second line''s command word is read as an argument of the first `touch` and minted as a touched path. It is the it-ap3x / it-dt68 false-positive class again, and the plausibility floor cannot catch it: "touch" is a perfectly legal filename.

FALSE NEGATIVE — `cargo build > build.log\ncp fixtures/a.rs src/a.rs` parses to ["build.log"] alone. The `cp` on line two is consumed as an argument of line one and never scanned as a command, so a real write goes unseen — the sight boundary widens by a whole line.

The cure is a boundary at an unquoted newline, but not a naive one: this tokenizer does no escape handling, so a line continuation (a trailing `\` in bash, a trailing backtick in PowerShell) would lose its tail — `cp a.rs \<newline> src/b.rs` would drop its destination. The fix wants the newline boundary PLUS the continuation exception, and a positive control for each shape.

Note the function''s doc comment claimed newlines already pushed a boundary; it-dt68 corrected the comment to match the code and left the behaviour here.
