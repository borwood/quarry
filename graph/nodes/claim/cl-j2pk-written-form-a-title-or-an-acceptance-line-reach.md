---
id: cl-j2pk
type: claim
title: '`written-form`: a title or an acceptance line reaches the graph as it was typed — a control character or an unpaired backtick refuses at every station'
v: 6
status: ratified
provenance: assistant
created: 2026-08-21T15:08:36Z
actor: claude-opus-5
kind: vein
ratified:
  by: claude-opus-5
  date: 2026-08-21
edges:
- rel: about
  to: ar-c7f5
  at: 1
- rel: source
  to: file:src/ops.rs
  at: 2c2276685d23
- rel: source
  to: file:src/main.rs
  at: dfb475f7fdc2
- rel: source
  to: file:tests/basic.rs
  at: df4237b7ba73
- rel: supports
  to: dc-qvtz
  at: 3
---

`written-form`: a title or an acceptance line reaches the graph as it was typed — a control character or an unpaired backtick refuses at every station

THE HOUSE SHELL EATS THE HOUSE STYLE, MEASURED. dc-qvtz makes a backticked name the vein register's form and PowerShell is this machine's shell, where the backtick is the ESCAPE character inside a DOUBLE-quoted argument. Probed live 2026-08-21: "`replacement-clear`: a fire" arrives as U+000D + "eplacement-clear: a fire", 25 characters against the 27 that were typed — BOTH backticks gone, the leading r gone with the first of them, and nothing syntactically wrong left in the string for a reader to notice. cl-gy6q was minted through exactly that hole; its create event in graph/log/2026-08.jsonl carries the corrupted title verbatim, and slugify of that string is character-for-character the filename it wore until this landing.

TWO TELLS, ONE PREDICATE, EVERY STATION. ops::check_written_form judges the written form of a title or an acceptance line and refuses ahead of every mutation. A CONTROL CHARACTER: each of PowerShell's escape letters — r n t a b f v 0 e, the whole class the brief enumerates — produces a C0 control character exactly where the register name began, and a title or an acceptance line is one line of prose where no control character is ever intended. An ODD BACKTICK COUNT: the register form pairs, so a lone backtick is a segment whose other half was eaten or never typed. Measured at filing across the whole store, both tells have zero false positives: 429 node titles and 147 acceptance lines, swept.

The stations are ops::new_node (title and mint-time acceptance lines — every mint road funnels through it: q new, q claim with --title and derived alike, q rule, the dispatch report registration) and ops::set (title= and acceptance+=). ops::truncate_title checks the RAW first line before the trim and before the cut, because `.trim()` erases exactly the leading control character that is the evidence, and a truncation can orphan a backtick that was paired when it was typed. The retitle station is not optional: a retitle is where a corrupted name gets repaired, so it is the one place the corruption must never enter a second time.

REFUSE, NOT WARN, AND TEACH THE ROAD. Each refusal names the tell, renders the received value with its control characters made visible (a control character is invisible in a terminal echo — which is why the corruption goes unnoticed at all), states the mechanism, and teaches SINGLE quotes as the RULE rather than a fallback, with the here-string (@'…'@) beside it for long values. q claim --help and q set --help now carry the same teaching where titles are authored.

THE RESIDUE IS STATED, NOT HIDDEN, and it is why the teaching is the rule. A backticked name whose first letter is NOT an escape letter loses its backticks and NOTHING else: measured, "`slug-follows`: y" arrives as "slug-follows: y". What reaches q is then a legal plain title, and no check at this end can tell it from one that was typed that way — the name is simply absent from the register, so dc-qvtz's backticked join never fires and the vein prompt keyed on the same shape (cl-sv7z, cl-vuda) stays silent. Only the authoring road cures that half.

Pinned by a_shell_eaten_register_name_never_lands_silently in tests/basic.rs: the incident literal verbatim (Rust reads \r exactly as PowerShell does), the pre-fix slug measured on slugify itself, both refusal arms, every escape letter, both stations plus the mint's acceptance lines, a refused act bumping and writing nothing, the register form passing as the positive control, and the residue asserted as the boundary it is. Each arm probed against the regression it guards: disabling the control-character arm reproduces the incident end to end (the mint lands wearing cl-gy6q's damaged filename), and disabling the parity arm lets the unpaired form through.
