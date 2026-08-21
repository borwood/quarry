# dispatch report: a bash heredoc body tokenizes as command text

item: it-dt68 · area: cli · 2026-08-21 · files: `src/teach.rs`, `tests/basic.rs`

## the outcome

Acceptance line: *prose inside a bash heredoc never mints a touched path: the
body is consumed whole to its line-initial terminator, in the quoted, bare, and
tab-stripped forms alike, and a real write beside the heredoc still parses.*

Met. `teach::shell_tokens` now tracks the heredoc delimiter and takes the body
as one opaque token; the full suite is green (144 passed, 0 failed — `basic`
136, `broken_pipe` 2, `observed_set` 1, `store_lint` 1, `surface_lint` 1,
`worktree_proof` 3, doc-tests 0).

## what the parser did before, measured

Measured on `write_shapes` directly, with the heredoc arm disabled and
everything else in place:

| command | parsed targets, pre-fix |
| --- | --- |
| `cat <<EOF` ⏎ `touch src/ghost.rs` ⏎ `EOF` | `["src/ghost.rs", "EOF"]` |
| `cat <<"EOF"` / `cat << EOF` / `cat <<-EOF` (same body) | `["src/ghost.rs", "EOF"]` |
| `cat <<EOF > out/real.txt` ⏎ `touch src/ghost.rs` ⏎ `EOF` | `["out/real.txt", "src/ghost.rs", "EOF"]` |
| `git commit -F - <<'EOF'` ⏎ *(commit prose)* ⏎ `EOF` | `["EOF"]` |
| `git commit -F - <<'EOF'` ⏎ `touch src/ghost.rs` ⏎ `EOF` ⏎ `touch src/real.rs` | `["src/ghost.rs", "EOF", "touch", "src/real.rs"]` |
| `cat <<EOF` ⏎ `touch src/ghost.rs` ⏎ *(no terminator)* | `["src/ghost.rs", "never", "closes"]` |

Two things in that table are worth pausing on.

First, the commit-road row is the it-ap3x incident replayed verbatim, one
channel over. The dispatcher's `git commit -F - <<'EOF'` body carries a
`Co-Authored-By: … <noreply@anthropic.com>` line; the `>` that closes that
address is read as a redirect, and the next word — the terminator `EOF`, sitting
on its own line — becomes its target. A plausible-looking path, minted from a
commit message, in the exact channel this repo moved to *because* the here-string
form produced the it-ap3x debris.

Second, the last rows show the parse minting the word `touch` itself as a path.
That is not the heredoc — see the user-owned/filed section below.

Post-fix, every heredoc row above parses to `[]` or to the real write alone
(`["out/real.txt"]`, `["src/real.rs"]`).

## the mechanism

`src/teach.rs`, `shell_tokens` — the cure the brief named, delimiter tracking,
with two shape decisions worth stating:

**The operator is read as a run.** One `<` is a plain input redirect; three is a
bash here-string (`<<< "word"`), whose word is already a single token and has no
body to swallow; exactly two is a heredoc opener. At an opener the tokenizer
reads the optional `-`, any spaces, and the delimiter — quoted (`<<'EOF'`,
`<<"EOF"`) or bare — in place, and pushes them onto a pending list. The
delimiter never enters the token stream, so it can never stand as a command word
or a redirect target.

**The body is taken at the newline that ends the opener line, not at the
operator.** That is bash's own ordering and it is the reason
`cat <<EOF > out/real.txt` still hands `out/real.txt` to its redirect —
consuming at the operator would have been three lines shorter and would have
eaten the rest of the command line along with the body. Each heredoc opened on
the line takes its body in turn (two openers, two bodies, in order), running to
a line whose *whole* content is the delimiter — trailing whitespace and a
CRLF's `\r` trimmed, leading whitespace only under `<<-` — or to end of input
when there is no terminator. A partial match never terminates: a body line
reading `EOF is the delimiter here` is body.

Each body is pushed as one token **followed by a boundary**. The token is opaque
by construction: it carries the `<<` it opened with, and `opaque_target` already
refuses any token holding `<`, so even a dangling `>` in front of it cannot mint
it. The trailing boundary is load-bearing in the other direction — without it
the body token becomes a command word whose argument scan swallows the real
command after the terminator, and `EOF` ⏎ `touch src/real.rs` loses
`src/real.rs`.

The function's doc comment claimed newlines already pushed a boundary. They do
not, and never did; the comment now describes what the code does.

## how it is pinned

- `write_shapes_never_mints_a_path_from_a_bash_heredoc_body` (tests/basic.rs) —
  the commit-road shape, all the openers (quoted, double-quoted, bare, spaced,
  `<<-` with an indented terminator), the redirect on the opener line, the
  command after the terminator, two heredocs on one line, an unterminated body,
  whole-line delimiter matching, a CRLF terminator, and the `<<<` and plain-`<`
  positive controls.
- an added arm of `observe_shell_accrues_only_resolvable_targets_marked_shell` —
  end to end, carrying *why the drop belongs at the parse*: `src/ghost.rs`
  resolves store-relative by mere path joining, nothing on disk is consulted, so
  no step downstream could have told it from a real write. The real
  write beside the heredoc (`> out.log`) still accrues.
- **probed against the regression it guards**: with the pending push disabled,
  `write_shapes_never_mints_a_path_from_a_bash_heredoc_body` fails on its first
  assert with `["EOF"]` in the observed set. The other two `write_shapes` tests
  pass under that probe, which is the point — nothing already standing covered
  this.

## graph writes

- `cl-p4k2` — vein, ``heredoc-opaque``: the mechanism, `--source
  file:src/teach.rs`, `supports` → it-dt68.
- `q affirm cl-zj2c --to it-dt68` — the homework the link raised; the claim's
  two halves are re-verified by this arc (its own test stayed green across a
  rewrite of the function it pins).
- `it-dprv` — a bug filed, not fixed, see below.

**cl-zj2c wants an amendment at harvest.** Its closing paragraph names this
heredoc hole as open residue ("Closing it wants delimiter tracking in
shell_tokens, the same move made here for the PowerShell form"). That paragraph
now describes covered ground. Amending a ratified claim's body is not an agent's
hand — flagging it is.

## user-owned calls:

none.

Two calls came up and neither is the user's. The first — refuse or split a
comma-joined value, refuse or swallow a heredoc — was already settled by
standing doctrine (cl-zj2c: a parse is a guess, dropping is the cheap direction,
a false positive is the expensive one), so swallowing needed no new ruling. The
second is scope: I found a second defect in the same function and filed it
instead of fixing it (below). That is the dispatcher's call to schedule, not the
user's to rule, and CLAUDE.md's standing ruling — *defects are bugs on sight, no
second-instance thresholds* — is satisfied by filing it as a bug rather than a
watch.

## the defect I filed instead of fixing: it-dprv

**A newline is not a command boundary.** In `shell_tokens` the
`c if c.is_whitespace()` arm precedes the separator arm, so `\n` flushes a word
and never pushes a boundary: a multi-line command tokenizes as ONE command and
only its first word is ever read as a command word. Measured, post-fix:

- false positive — `touch src/a.rs` ⏎ `touch src/b.rs` → `["src/a.rs", "touch",
  "src/b.rs"]`. The second line's command word is minted as a touched path, and
  the plausibility floor cannot catch it: `touch` is a legal filename.
- false negative — `cargo build > build.log` ⏎ `cp fixtures/a.rs src/a.rs` →
  `["build.log"]` alone. The `cp` is consumed as an argument of line one and
  never scanned, so a real write goes unseen.

I left it because the cure is not this arc's cure and is not one line: the
tokenizer does no escape handling, so a naive newline boundary breaks line
continuations (`cp a.rs \` ⏎ `  src/b.rs` would lose its destination). It wants
the boundary plus the trailing-`\` (and PowerShell backtick) exception, with a
positive control for each. Filed as `it-dprv`, kind bug, about `ar-c7f5`, with
both measurements in the body.

## reflections

**The false positive I expected is not the one the incident produces.** The
brief predicted a heredoc line reading `touch foo` minting `foo`. It does — but
the commit road's actual failure is subtler and nastier: the `>` inside a
`Co-Authored-By: … <address>` line reaches across the newline and grabs the
*terminator* as its target. Same root, and the fix covers both, but I would not
have written the test that caught it if I had only reasoned from the brief. The
probe table is what showed it. I would build the throwaway probe again.

**Where the body is taken mattered more than I expected.** The obvious reading
of "consume the body to its line-initial terminator as one opaque token" is to
do it at the `<<`. That is wrong in a way that only shows up on
`cat <<EOF > out.txt`, and it fails the second half of the acceptance line while
passing the first. Deferring to the newline is a handful of extra lines and one
`pending` vec, and it made the two-heredocs-on-one-line case fall out for free.

**Doubt: `<<-` leniency.** Bash strips only tabs from a `<<-` terminator; I
trim all leading whitespace. A space-indented terminator under `<<-` is not a
terminator to bash, so on that input my parse ends the body where bash would
not. I took it deliberately — the writer's intent on such a line is
unambiguous, and ending the body early only resumes ordinary parsing rather than
minting anything — but it is a divergence from the shell, and if this parser
ever grows a stricter purpose that choice should be revisited.

**Friction: the graph would not let me record the lean I wanted.**
`cl-p4k2 depends-on cl-zj2c` is refused (depends-on takes item/thread →
item/thread/decision), and the legal claim → claim rels are supports, refutes,
supersedes. My claim *stands on* cl-zj2c; `supports` points the load the other
way, and refutes/supersedes are settling rels that are never mine. So a real
lean between two claims has no honest edge, and I left it as a mention. If the
edge matrix is meant to carry "this claim extends that one", it is missing a
rel; if mentions are meant to carry it, the doctrine line ("link only what
leans") reads as though an edge would be available.
