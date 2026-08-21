---
id: cl-p4k2
type: claim
title: '`heredoc-opaque`: a bash heredoc body is one opaque token, taken at the opener''s newline so the redirect beside it survives'
v: 4
status: ratified
provenance: assistant
created: 2026-08-21T11:22:36Z
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
  at: cdb043c6d64a
- rel: supports
  to: it-dt68
  at: 5
---

`heredoc-opaque`: a bash heredoc body is one opaque token, taken at the opener''s newline so the redirect beside it survives

The second half of the tokenizer-edge floor under cl-zj2c, one channel over (it-dt68). teach::shell_tokens tracked no heredoc delimiter at all: the `<<` pushed a boundary, so nothing desynced the way the PowerShell here-string did, but every line of the body still tokenized as ordinary words — and prose that reads like a command mints a path nothing downstream can question. Measured before the fix, `cat <<EOF` + "touch src/ghost.rs" + `EOF` accrued ["src/ghost.rs", "EOF"], and the repo''s own commit road (`git commit -F - <<''EOF''`) accrued "EOF" through a `>` inside a Co-Authored-By address — the it-ap3x incident replayed in the channel this repo actually uses for prose, since the it-ap3x landing is exactly what moved commits off the here-string and onto the heredoc.

THE OPERATOR IS READ AS A RUN. One `<` is a plain input redirect and three is a bash here-string (its word is a single token already, no body to swallow); exactly two is a heredoc opener. At an opener the tokenizer reads, in place, the optional `-`, any spaces, and the delimiter — quoted (`<<''EOF''`, `<<"EOF"`, the quotes belonging to the opener) or bare — and pushes them onto a pending list. The delimiter never reaches the token stream, so it can never stand as a command word or a target.

THE BODY IS TAKEN AT THE NEWLINE THAT ENDS THE OPENER LINE, not at the operator — bash''s own ordering, and the reason `cat <<EOF > out/real.txt` still hands out/real.txt to its redirect. Consuming at the operator would have been simpler and would have eaten the rest of the command line with the body. Each heredoc opened on the line takes its body in turn (two openers, two bodies, in order), running to a line whose whole content is the delimiter — trailing whitespace and a CRLF''s `\r` trimmed, leading whitespace only under the `<<-` form — or to end of input when it has none. A partial match never terminates: a body line reading "EOF is the delimiter here" is body.

Each body is pushed as ONE token followed by a boundary. The token is opaque by construction — it carries the `<<` it opened with, and opaque_target refuses any token holding `<`, so even a dangling `>` in front of it cannot mint it as a target. The trailing boundary is load-bearing in the other direction: without it the body token becomes a command word whose argument scan swallows the real command after the terminator, and `... EOF` + "touch src/real.rs" loses src/real.rs.

Pinned by write_shapes_never_mints_a_path_from_a_bash_heredoc_body in tests/basic.rs — the commit-road shape, all four openers (quoted, double-quoted, bare, `<<-` with an indented terminator, plus the spaced `<< EOF`), the redirect on the opener line, the command after the terminator, two heredocs on one line, an unterminated body, whole-line delimiter matching, the CRLF terminator, and the `<<<` and plain-`<` positive controls — and by an added arm of observe_shell_accrues_only_resolvable_targets_marked_shell, which carries why the drop belongs at the PARSE: src/ghost.rs resolves store-relative by mere path joining, so no resolution step downstream could tell it from a real write. Probed against the regression it guards: dropping the pending push fails the suite with ["EOF"] in the observed set.

This closes the residue cl-zj2c named in its closing paragraph — that paragraph now describes ground that is covered. What remains open in this parser is a different hole, filed as it-dprv: a newline is not a command boundary, so the next line''s command word reads as the previous line''s argument (a false positive both ways — "touch" minted as a path, a second line''s real write unseen).
