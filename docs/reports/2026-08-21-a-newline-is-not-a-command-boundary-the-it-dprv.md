# dispatch report: a newline is not a command boundary

**it-dprv** · area `cli` · agent `a211dc6b7d04e1a62` · 2026-08-21

## the outcome

The acceptance line is met. An unquoted newline now ends the command before it
in `teach::shell_tokens`, so no later line's command word mints as a path and no
write on a later line goes unseen — while a line continuation keeps its tail.

`cargo test`: **137 passed, 0 failed** in `tests/basic.rs`, and every other
target green (2, 1, 1, 1, 3, 0 across `broken_pipe`, `observed_set`,
`store_lint`, `surface_lint`, `worktree_proof`, doc-tests). Release rebuilt, so
the hooks run the fixed parser.

## what the parser did before, measured

I built a throwaway probe (`tests/zz_prefix_measure.rs`, deleted) and ran every
shape through `write_shapes` before touching `shell_tokens`, then again after.
Nothing below is reasoned.

| command | before | after |
| --- | --- | --- |
| `touch src/a.rs` ⏎ `touch src/b.rs` | `["src/a.rs", "touch", "src/b.rs"]` | `["src/a.rs", "src/b.rs"]` |
| `cargo build > build.log` ⏎ `cp fixtures/a.rs src/a.rs` | `["build.log"]` | `["build.log", "src/a.rs"]` |
| `touch src/a.rs` ⏎ `touch src/b.rs` ⏎ `touch src/c.rs` | `["src/a.rs", "touch", "src/b.rs", "src/c.rs"]` | all three |
| `touch src/a.rs` **CRLF** `touch src/b.rs` | `["src/a.rs", "touch", "src/b.rs"]` | `["src/a.rs", "src/b.rs"]` |
| `cargo build >` ⏎ `touch src/a.rs` | `["touch"]` | `["src/a.rs"]` |
| `Copy-Item a.rs B:\dest\` ⏎ `touch src/b.rs` | `["B:\dest\"]` | `["B:\dest\", "src/b.rs"]` |

Two findings the brief did not predict:

**The false positive is bigger than one word.** The brief's example mints
`touch`. It is worse in the shape end to end: `touch src/one.rs` ⏎
`cp fixtures/a.rs src/two.rs` accrued **three** phantom paths — `cp`,
`fixtures/a.rs`, and `src/two.rs` all read as arguments of line one's `touch`,
every one of them resolving store-relative. That measurement now stands as the
probe assert in `observe_shell_accrues_only_resolvable_targets_marked_shell`.

**A dangling redirect reaches across the newline.** `cargo build >` ⏎
`touch src/a.rs` parsed to `["touch"]` — nothing but the phantom. Same root
(the `>` scan takes the next token, and the next token was the next line's
command word), and the same class as the it-dt68 incident where a `>` inside a
`Co-Authored-By` address grabbed the heredoc terminator.

## the mechanism

Three arms in `shell_tokens`, all ahead of the `c if c.is_whitespace()` arm that
used to swallow `\n`, and all behind the heredoc arm that already claims its own
newline (cl-p4k2's ordering is untouched — a body is still taken at the newline
that ends its opener line).

1. **`'\n' => flush + boundary`**, exactly as `;` does. A quoted newline is not
   a boundary at all: the quote branch runs before the `match`, so prose inside
   an argument stays one token and the it-ap3x floor keeps holding.
2. **a newline whose current word is a lone `\` (bash) or a lone backtick
   (PowerShell)** clears that word instead — the line continuation, deleted
   exactly as bash removes a backslash-newline pair, so the tail joins the
   command in front of it.
3. **`'\r' if chars.peek() == Some(&'\n') => {}`** — a CRLF's `\r` belongs to
   the newline behind it, not the word in front, so the continuation check reads
   the real last character of the line.

**The boundary can only reduce false positives.** A line's first word is now
read as a command word, and on a real multi-line command a line starting with
`touch` *is* a touch. Prose reaches this parser only inside quotes, a
here-string, or a heredoc body — all three consumed whole before the newline arm
sees them. There is no third way in.

**The continuation exception is one word wide, and that was settled by
measurement, not taste.** `\` also ends a Windows directory path, and both
shells share this parser. I flipped the arm to the broad rule
(`cur.ends_with('\\')`) and measured:

- `Copy-Item a.rs B:\dest\` ⏎ `touch src/b.rs` → `["B:\desttouch"]` — a
  plausible path nobody wrote (the expensive direction cl-zj2c named), *and*
  line two's real write lost with it. The word rule reads both correctly.
- `cp a.rs\` ⏎ `src/b.rs` → `[]` under the broad rule **and** `[]` under the
  word rule. The broad rule buys nothing for the glued form it exists to cover,
  because bash's own joining makes that destination `a.rssrc/b.rs`, which the
  parse drops either way.

So the narrowing costs nothing measurable and removes a minting shape. The glued
continuation is the named residue, pinned as such — a lost target, never an
invented one.

## how it is pinned

`a_newline_ends_the_command_before_it_unless_the_line_continues` in
`tests/basic.rs`: both incident shapes verbatim, three lines, the CRLF line end,
the dangling redirect, both shells' continuations in LF and CRLF, the
Windows-path negative with its measured alternative in the comment, the
glued-form residue, a quoted-prose control (a commit message whose line reads
`touch src/ghost.rs` mints nothing), and cl-p4k2's heredoc control re-read
through the new boundary.

Plus an added arm of `observe_shell_accrues_only_resolvable_targets_marked_shell`
carrying why the drop belongs at the **parse**: a bare `touch` resolves
store-relative by mere path joining, so no resolution step downstream could tell
it from a real write, and no plausibility floor can refuse it — `touch` is a
legal filename.

Each half probed against the regression it guards, one at a time:

- boundary removed → **both** tests fail; the observed set shows `cp` and
  `fixtures/a.rs` as touched paths (135 passed, 2 failed).
- continuation arm removed → `cp fixtures/a.rs \` ⏎ `src/b.rs` parses to `[]`,
  destination gone.
- `\r` skip removed → the CRLF continuation parses to `[]`.

## graph writes

- **`cl-v2vh`** — vein, `` `newline-boundary` ``: an unquoted newline ends the
  command before it; the continuation exception is one word wide.
  `--source file:src/teach.rs`, `--about ar-c7f5`, `supports` → it-dprv.
- **`cl-p4k2 -[supports]-> cl-v2vh`** — the lean, recorded in the direction the
  matrix allows (claim → claim `depends-on` is refused; the lean runs the way
  `weight_held` counts load). It is a real lean, not a mention: without
  heredoc-opaque, a newline boundary would *actively* mint prose from heredoc
  bodies — worse than the state it replaces.
- **`q affirm cl-p4k2`** (1 ref) and **`q affirm cl-zj2c`** (2 refs) — my edit
  drifted both against `src/teach.rs`, and cl-zj2c against `tests/basic.rs` too.
  I re-read both bodies whole and re-ran their instruments across a change to
  the function they pin. Note cl-zj2c was *already* behind on `src/teach.rs`
  from the it-dt68 arc; my affirm stamps it current against the whole file, and
  I did read both of its halves (the here-string arm and `opaque_target`) in
  their present form.
- **`q affirm cl-up6s --to file:src/teach.rs`** (1 ref) — scoped deliberately.
  I re-read its SHELL CHANNEL paragraph and it stays exactly true; its
  `src/render.rs` and `src/coord.rs` drift is not mine and is left standing for
  the source-drift pass.

**cl-zj2c and cl-p4k2 both want an amendment at harvest.** Each closes with a
paragraph naming this hole as open residue — cl-zj2c: *"The residue that remains
is narrower and named: it-dprv…"*; cl-p4k2: *"What remains open in this parser
is a different hole, filed as it-dprv…"*. Both paragraphs now describe covered
ground. Amending a ratified claim's body is not an agent's hand — flagging it
is, exactly as the it-dt68 report flagged cl-zj2c for this same reason.

**Three files in the observed set are not in the diff.** `tests/zz_prefix_measure.rs`
and `tests/zz_broad_probe.rs` were the throwaway probes, created and deleted
inside the lease; the scratchpad `claim.txt` is recorded unresolved because I
was badged (cl-up6s's no-silent-discard invariant working as designed) and held
the vein body while I handed it to `q claim` as an argument.

## the defect I filed instead of fixing: it-awhz

**A scoped affirm prints the unscoped all-clear.** main.rs's Affirm arm has one
message for a zero count — `if count == 0 { outln!("nothing behind — no restamp
needed.") }` — and that line is the *unscoped* statement (every ref on this node
is current). It also prints when the zero comes from `--to` naming a target the
node has no behind edge toward. Measured, 2026-08-21:
`q affirm cl-up6s --to it-dprv` printed the all-clear while `q query behind`
showed cl-up6s behind on two refs in the same second.

I hit it by copying the it-dt68 report's recipe verbatim (`q affirm cl-zj2c --to
it-dt68`), got the line twice, and would have reported both affirms done had I
not gone to `q query behind` and grepped for the ids — where both claims were
genuinely drifted against the file my own diff had just changed. With 110 behind
entries, a silently skipped affirm is invisible. That is the it-bj3b species of
sight failure one surface over: the accounting reporting the opposite of the
truth.

Filed rather than fixed: the cure is in `src/main.rs`, outside this lease, and
the message wants a real wording call (name the scope, and ideally say whether
the node is behind elsewhere with the unscoped command in hand). Filed as
`it-awhz`, kind bug, about `ar-c7f5`, with the measurement in the body — plus a
second, smaller note there: `--to` accepts a file ref
(`q affirm cl-up6s --to file:src/teach.rs` restamped 1), which is exactly the
affordance an agent wants after reviewing one file of a multi-file claim, and
nothing in the verb's help says so.

## user-owned calls:

none.

Two judgment calls came up and neither is the user's. The first — narrow the
continuation exception to a standalone word, or read any trailing backslash —
is settled by standing doctrine (cl-zj2c: a parse is a guess, dropping is the
cheap direction, a false positive is the expensive one), and I measured both
rules rather than reasoning about them, so no new ruling was needed. The second
is the `cl-p4k2 -[supports]-> cl-v2vh` edge: dc-ez67 puts edges in the agent's
hand and has them judged at harvest, which is where it belongs, not with the
user.

## reflections

**I nearly shipped the broad continuation rule.** My first instinct was
`cur.ends_with('\\')` — it reads more "correct", it is what bash actually does,
and it would have introduced a minting shape on the very machine this parser
runs on (Windows paths end in `\`). What saved it was flipping the arm and
measuring instead of arguing. The measurement also killed the argument *for* the
broad rule: it does not even rescue the glued form, because the parse drops the
joined destination anyway. Two minutes of probe beat a paragraph of reasoning,
and I would spend them again.

**The `\r` arm is the kind of thing that would have shipped broken.** On this
box every file is CRLF and the heredoc code already trims `\r` — but the
continuation check reads `cur`, and a `\r` had already flushed the lone `\` into
the token stream before the newline arm ever saw it. I only caught it because I
put a CRLF continuation in the test table on general principle, not because I
reasoned my way to it. A tokenizer with no escape handling has more of these.

**Doubt: the glued continuation residue is real but I could not make it matter.**
`cp a.rs\` ⏎ `src/b.rs` loses its destination. I pinned it as a residue assert
rather than fixing it, because every fix I could see (look back at the previous
token, or accept any trailing `\`) either re-opens the Windows-path minting or
needs real escape handling — which this tokenizer deliberately does not have.
If someone later gives `shell_tokens` proper escape handling, that assert is the
first thing to revisit, and it should flip to the real bash reading rather than
be deleted.

**Design friction, now filed as it-awhz: the affirm output nearly cost this arc
its homework.** I copied the sibling report's `q affirm <cl> --to <item>` shape
because a landed, harvested report is the most trustworthy recipe available to
an agent with no memory — and it returned an all-clear on two claims my own diff
had drifted. What is worth noticing beyond the defect itself is *why* I recovered:
not from the verb, but from reading `q query behind` whole because I wanted the
blob hashes for this report. Diligence unrelated to the task is a bad backstop.

**Doubt about my own affirm on cl-zj2c.** It was behind on `src/teach.rs` from
before my arc (the it-dt68 landing), and my restamp stamps it current against
the whole present file, not just my hunk. I did read both of its halves in their
present form and its instrument passes, so I stand behind it — but an affirm has
no way to say *"I reviewed this against the part I changed"*, and the honest
alternative was to leave a ratified claim reading as drifted for a reason that no
longer existed. If the judge disagrees, the fix is to un-affirm and re-read; I
flagged it in the graph-writes section rather than letting it pass silently.
