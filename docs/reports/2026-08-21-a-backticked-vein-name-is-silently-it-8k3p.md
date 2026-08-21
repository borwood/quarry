# dispatch report — it-8k3p

**a backticked vein name is silently eaten by the house shell, and a title fix never re-slugs its file**

Arc: it-8k3p · agent badge `agent:a5ab80f9b951475e8` · 2026-08-21 · areas cli, process

---

## Against the RETURN spec

> a vein name reaches the graph as it was written or the mint refuses: a title whose
> backticked register segment is malformed never lands silently, and a corrected title
> either carries its filename with it or says plainly that it will not

**Met, on both clauses, with one stated residue.** A malformed register segment now refuses at
every station that writes a title or an acceptance line, and a corrected title carries its
filename with it *and* says so — in the echo, in the log, and in `q set --help`. The residue is
one shell behaviour no check at the receiving end can see; it is measured, stated in the vein
body, and filed as its own defect (it-d4bh) because its cure sits outside this arc's lease.

---

## What the mechanism actually is, measured

The brief proposed the tell would be "a backticked segment with an unbalanced or missing opening
backtick". **That is not what happened, and building to it would have missed the incident.**

Probed live in this machine's shell, 2026-08-21:

| typed | arrives as | length |
|---|---|---|
| `"` + `` `replacement-clear` ``+ `: a fire"` (double quotes) | `U+000D` + `eplacement-clear: a fire` | 25 (from 27) |
| `'` + `` `replacement-clear` `` + `: a fire'` (single quotes) | unchanged, backtick = `U+0060` | 27 |
| `"` + `` `slug-follows` `` + `: y"` | `slug-follows: y` | backticks gone, nothing else |

**Both** backticks are eaten, not one. `` `r `` becomes a carriage return; `` `: `` becomes a bare
colon. The string that reached `q` had **zero** backticks and perfect parity, so a
balance check alone would have passed it. Confirmed against the incident's own record —
`graph/log/2026-08.jsonl`, the cl-gy6q create event, carries `"\replacement-clear: a fire…"`
verbatim, and `slugify` of that string is character-for-character the filename cl-gy6q wore.

The tell that *is* exact: **a control character**. Every one of PowerShell's escape letters —
`r n t a b f v 0 e`, the entire class the brief enumerates — produces a C0 control character
exactly where the register name began. A title and an acceptance line are each one line of prose,
so no control character in one is ever what anybody typed.

---

## What landed

### 1. `written-form` — the floor (cl-j2pk)

`ops::check_written_form` refuses on two tells: **a control character**, and **an odd backtick
count** (the brief's tell, kept — it catches an unpaired segment from any cause, and cost
nothing). Each refusal names the tell, renders the received value with its control characters made
visible (a control character is invisible in a terminal echo, which is exactly why the corruption
goes unnoticed), states the mechanism, and teaches **single quotes as the rule, not a fallback**,
with the here-string beside it.

Stations, one predicate:

- `ops::new_node` — title and mint-time acceptance lines. Every mint road funnels through it:
  `q new`, `q claim` (`--title` and derived alike), `q rule`, the dispatch report registration.
- `ops::truncate_title` — checks the **raw first line**, before the trim and before the cut.
  This is load-bearing, not belt-and-braces: `.trim()` erases exactly the leading control
  character that is the evidence, so the derived-title road would otherwise pass a corrupted
  name through silently. Truncation can also orphan a backtick that was paired when typed.
- `ops::set` — `title=` and `acceptance+=`. The retitle station is where a corrupted name gets
  *repaired*, so it is the one place the corruption must never enter a second time.

**Refuse, not warn.** The acceptance line says "or the mint refuses"; dc-ygzz says defects are
bugs on sight; and the false-positive cost was measured at zero (below). A node already holding a
corrupted title is not trapped by this — the check judges the *new* value, so the repair road
stays open.

**False positives, measured across the whole real store at filing:** 429 node titles and 147
acceptance lines (YAML continuations joined) swept for both tells — **zero** hits on either.

### 2. `slug-follows-title` — the filename follows (cl-h9d6)

`q set <id> title=…` now moves the node's file to `<id>-<slugify(title)>.md`.

- The old slug was **already** recorded in `front.aliases` by every retitle — written by one line
  and read by nobody since it was added. `Store::find` gained an alias rung between the live-slug
  rung and the title-fragment rung, so a name the node has left still answers while a live slug
  outranks it. Moving the file is what makes that dead field load-bearing.
- **Placement**: the rename is decided in the field loop and committed *past the last refusal* —
  past the acceptance gate and the it-ds6b demotion — so a refused `q set` moves no file. That is
  cl-gy6q's own rule applied to its own repair.
- **Ordering**: rename-then-save, never save-then-delete. Whichever step fails, exactly one file
  wears the node's id; two files with one id is the one state `load_all` cannot read straight.
  `Store::move_node_file` refuses outright if anything stands at the target.
- **Never silent**: the move rides `SetOutcome.moved` to an echo naming both names and the alias,
  and onto the set event as `renamed{from,to}`.
- **`q set --help` states it** — the other half of the brief's fork. The verb now carries the
  filename *and* says plainly that it does, including that re-setting a title to the value it
  already carries is the repair road for a file left behind by a pre-fix retitle.

### 3. The damaged artifact, repaired

`cl-gy6q-eplacement-clear-….md` → `cl-gy6q-replacement-clear-….md`, through the new mechanism
(not by hand), with `eplacement-clear-a-fire-that-displaces-a-live-ar` kept as a resolving alias.

### 4. The safer authoring road, named where titles are taught

`q claim --help` and `q set --help` both now carry it: single quotes for any backticked value,
here-string (`@'…'@`) for a long one, `--body-file` for a body longer still.

---

## Measurement the brief asked for, and one correction to its method

The brief's sweep compared **first words only** and found cl-gy6q as the single mismatch, noting
that a corruption deeper into a title would escape it. Re-run comparing the **full** file slug
against `slugify(current title)` across all 429 node files: **12 mismatches**.

- 1 is the corruption (cl-gy6q) — repaired.
- 11 are honest historical retitles whose files name titles that no longer exist: `cl-gu5f`,
  `cl-p6aj`, `dc-crea`, `dc-grrb`, `it-3pab`, `it-6ekf`, `it-csm3`, `it-kkde`, `it-pgn9`,
  `it-qxjx`, `th-4w3a` (several are the spine→vein rename).

**The 11 were deliberately left alone.** Each is repairable with one `q set <id> title=<its own
title>` under the new mechanism, but each repair bumps its node and sends its citers behind for a
name nothing downstream reads. That trade is the dispatcher's call, not mine to spend at a bug
landing; the road is recorded in cl-h9d6's body so it is not lost.

---

## The residue, stated

A backticked name whose first letter is **not** an escape letter loses its backticks and nothing
else: `` "`slug-follows`: y" `` arrives as `slug-follows: y`, measured. What reaches `q` is a
legal plain title and **no check at this end can distinguish it from one typed that way**. The
name is then simply absent from the register: dc-qvtz's backticked join never fires, and the vein
prompts keyed on the same shape (cl-sv7z at mint, cl-vuda at harvest) stay silent.

The cure that is left is a prompt, never a gate (dc-grrb): a `--kind vein` claim whose title
carries no backticked name draws the register-form question at the choke point that already
carries `SPECIES_PROMPT` and `VEIN_PROMPT`. Its line is a composed register string and those ride
`src/framings.rs` (dc-vzvf's flag channel), which sat outside this arc's write-set — the refusals
that landed are inline bails, which is why the floor was buildable here and the prompt was not.
**Filed as it-d4bh** (item·bug, sketch, cli) with the measurement and the proposed shape.

---

## Instruments

Both in `tests/basic.rs`; full suite green — **149 passed; 0 failed** (basic), plus 2 / 1 / 1 / 1 / 3
across broken_pipe, observed_set, store_lint, surface_lint, worktree_proof. Result lines read raw.

**`a_shell_eaten_register_name_never_lands_silently`** — the incident literal verbatim (Rust reads
`\r` exactly as PowerShell does), the pre-fix damage measured on `slugify` itself, both refusal
arms, every escape letter, both stations plus the mint's acceptance lines, a refused act bumping
and writing nothing, the register form passing as the positive control and joining
`backtick_titled`, and the residue asserted as the boundary it is.

**`a_retitle_carries_the_nodes_file_with_it`** — the move and the vacated name, exactly one file
per id, the alias rung answering the old slug while the live slug outranks it, the pre-fix state
rebuilt on disk and repaired by re-setting the title it already carries, a refused act moving
nothing, and the echo end to end through the spawned binary.

**Each half probed against the regression it guards** (disable, run, restore):

| disabled | what fails |
|---|---|
| control-character arm | the incident reproduces end to end — the mint lands wearing cl-gy6q's damaged filename, `cl-…-eplacement-clear-a-fire-that-displaces-a-live-ar.md` |
| parity arm | the unpaired `` `half-eaten `` form lands |
| the file move | `the retitle moved the file` |
| the alias rung | `no node matches 'the-wrogn-title'` — **and no other test in the suite notices**, which is why the rung is pinned in the new instrument |
| rename committed at the decision point instead of past the gates | the refused act has already destroyed the file's name |

---

## Graph writes

- **cl-j2pk** `written-form` — vein, about ar-c7f5, sourced on src/ops.rs, src/main.rs,
  tests/basic.rs; supports dc-qvtz (the deliberate-name floor only reaches the graph if the shell
  does not eat it first).
- **cl-h9d6** `slug-follows-title` — vein, about ar-c7f5, sourced on src/ops.rs, src/store.rs,
  src/main.rs, tests/basic.rs.
- **it-d4bh** — the residue, filed as a defect (above).
- **cl-gy6q** — repaired: file moved, alias kept, v8.
- **Affirms** (scoped, `--to`, each on a ref I actually reviewed): cl-erm8 and cl-up6s on
  `file:src/store.rs` (diff reviewed and confined to `find`'s new rung and the new
  `move_node_file` — neither claim's subject is touched); cl-2kxr, cl-644e, cl-b2z2, cl-c34g,
  cl-cv92, cl-dqt4, cl-en99, cl-gy6q, cl-qm6t, cl-zj2c on `file:tests/basic.rs` (the diff is
  **221 insertions, 0 deletions** — measured with `git diff --numstat`, so every named instrument
  is byte-identical and every one ran green); cl-gy6q on `file:src/ops.rs`.
- **Left standing, deliberately**: the remaining `src/ops.rs` and `src/main.rs` drift on claims I
  did not review (a large pre-existing behind backlog stands beside it, most of it against
  src/teach.rs, src/queries.rs and src/render.rs, which this arc never touched). Affirming those
  would be attesting to a reading I did not do.

## user-owned calls:

none.

### forks met, and why none of them was the user's

Two forks were met and both were explicitly left to build time by the brief itself, not reserved
to the user:

- refuse vs. warn on a malformed register segment — settled to **refuse** by the acceptance line
  ("or the mint refuses"), by dc-ygzz (defects are bugs on sight), and by a measured zero
  false-positive rate across the whole store.
- re-derive the filename vs. state plainly that it will not — **both** were done, which is what
  the brief's own framing said was required to avoid leaving half the bug standing.

Checked and found *not* in conflict: dc-grrb's "never gates" governs vein-landmark presence
checks at landing ("a blocked done breeds Goodhart claims"), not input-integrity refusals at a
mint; this floor is the family of cl-74qt's glob-shape floor and dc-p6z4's acceptance gate.

## reflections

**The brief's diagnosis was right about the disease and wrong about the symptom, and that was the
most valuable thing to find.** It named an unbalanced backtick as the tell. Building to that would
have shipped a check that passes the exact string that caused the incident. The correction came
from two cheap measurements I nearly skipped as ceremony: reading the create event out of the log,
and running the escape in the live shell to count characters. Both took under a minute. I would
have felt equally confident having done neither, which is the uncomfortable part.

**The dead `aliases` field was the surprise.** Every retitle had been recording the old slug for
who knows how long, and nothing ever read it — and because the file never moved, what it recorded
was the slug the file *still had*, so the record was simultaneously unread and meaningless.
Moving the file made the field true and the rung obvious. It felt less like adding a feature than
like finding one already half-built and finishing it. I am slightly suspicious of that feeling.

**Where I stopped short and it nags.** The residue (it-d4bh) is the half of this defect that
actually costs the register something — a vein whose name never joins is invisible to exactly the
machinery the vein philosophy is built on — and I could not build it because a *prompt string*
lives in a file outside the lease while a *refusal string* lives inline. That is a real seam: the
composed-register discipline (dc-vzvf) makes prompts lease-expensive and refusals lease-cheap,
which quietly biases an arc toward refusing where prompting would be the better instrument. I do
not think that is anybody's intent, and it is worth someone's attention beyond this item.

**Uncertainty I want on the record.** The odd-backtick arm is the weaker of the two tells: it is
zero-false-positive across 429 titles and 147 acceptance lines today, but it is a *style* rule
enforced as a *refusal*, and the day someone legitimately wants one backtick in a title it will
refuse them with a lecture about a shell. The control-character arm has no such flavour — it
refuses only strings nobody could have typed. If one of the two ever has to go, it is the parity
arm.

**The eleven legacy filenames.** I left them and I am about 70/30 on it. The argument for
repairing is that a filename naming a title that no longer exists is a small lie the next reader
trips over; the argument against is eleven version bumps and eleven sets of citers going behind
for a name nothing downstream reads. I chose the reversible option and recorded the road, but if
the dispatcher's instinct is to sweep them, the mechanism is there and the sweep is eleven
commands.
