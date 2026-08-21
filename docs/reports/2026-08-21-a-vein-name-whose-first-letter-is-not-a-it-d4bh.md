# Dispatch report — it-d4bh: a vein name whose first letter is not a shell escape letter loses its backticks silently

**Outcome: acceptance met.** *"a vein or feature claim minted with no
backticked register name in its title draws the register-form question at the
same choke point that already carries the species and vein prompts, naming the
form and the single-quote road, and prompting only — never a gate"* — built,
pinned by a new instrument, probed against the regression it guards, and
measured live in the house shell against the release binary.

## What the defect actually is

cl-j2pk's floor catches the loud half of the shell damage. The quiet half
leaves nothing to catch. Re-measured live 2026-08-21 in PowerShell, against
`target/release/q.exe` with the fix in place, in a scratch store:

```
> q claim "`slug-follows`: the slug follows the title" --kind vein ...
✔ "slug-follows: the slug follows the title" — claim·vein [asserted] v2
```

Both backticks gone, every character between them kept, the mint legal. Where
the name's first letter is one of `r n t a b f v 0 e` the escape leaves a
control character and `ops::check_written_form` refuses; where it is any other
letter — `s` here — what reaches `q` is a plain title that no predicate at the
receiving end can distinguish from one typed that way. The single-quoted
control in the same session mints `` `slug-leads`: … `` intact, so the road
cl-j2pk teaches is real and the hole is only in the double-quoted one.

The cost is the whole register: dc-qvtz's backticked join never fires, so the
vein is invisible to relatedness and to the intent delta, and both prompts
keyed on `backtick_titled` (cl-sv7z at mint, cl-vuda at harvest) stay silent —
such a mint is not kindless, it is **kinded with nothing to register**.

## The fix — one arm, at the same choke point

`main.rs::print_mint_surfaces` carried two arms: `SPECIES_PROMPT` (kindless +
method) and `VEIN_PROMPT` (kindless + `backtick_titled`). The third arm is the
complement of the second — `ty == "claim"`, kind one of the two register
species (`vein`, `feature`, the pair dc-yd9s names and `queries::assayable`
rides), and `backtick_titled` **false**:

```rust
} else if node.front.ty == "claim"
    && matches!(node.front.kind.as_deref(), Some("vein") | Some("feature"))
    && !quarry::queries::backtick_titled(quarry::surface::title_raw(node))
{
    outln!("  register: {}", quarry::framings::REGISTER_FORM_PROMPT);
    outln!("    name it: q set {} 'title=`name`: what it provides'", node.front.id);
}
```

One predicate point is preserved: the arm calls cl-sv7z's own
`queries::backtick_titled`, negated — the mint prompt, the harvest ask and this
question can never disagree about what a registered name is. Every mint road
funnels through this point, so `q claim` and `q new claim` both ask.

`framings::REGISTER_FORM_PROMPT` (new, dc-dsdm channel — composed here, for
ratification at this harvest) names the form, states the mechanism, teaches
SINGLE quotes and the here-string as the road, and — because the two cases are
genuinely indistinguishable — says out loud that **a deliberately plain title
is a fine answer**. Only the author knows, which is exactly why it is a
question.

The settle line beside it is a **retitle**, not a species set:
`q set <cl> 'title=`name`: what it provides'`. It rides it-8k3p's retitle
station, so the node's file moves to the new slug. Verified live end to end:
the taught command landed the name and `q find slug-follows` then joined it.

A prompt, never a gate (dc-grrb): the mint lands on the asserted rung *before*
the question prints — asserted in the instrument and seen in the live run
above.

## Instruments

`tests/basic.rs::a_register_species_mint_with_no_registered_name_draws_the_form_question`
— end to end through the spawned binary, because the choke point lives in
`main.rs` and `Command` hands argv through untouched (so the eaten shape is
written as the literal the shell produces, exactly as the it-8k3p instrument
does). It carries: the eaten shape asserted to be a legal plain title first;
the mint standing asserted while the question prints; the settle line with the
minted id in it and the taught retitle actually landing the name in the
register; `--kind feature` drawing the same question; the register form as the
positive control; cl-sv7z's lane intact (a kindless backtick-titled mint still
draws `VEIN_PROMPT` and not this one); `measured` and kindless-plain drawing
nothing at all; both mint roads; and the verbiage pinned verbatim.

**Probed against the regression it guards.** Disabling the species half of the
arm reproduces the defect end to end — the test fails printing the silent mint
verbatim:

```
✔ "slug-follows: what it provides" — claim·vein [asserted] v2 · geology
```

no question, nothing else. Restored, green.

**Suite:** `cargo test` — `test result: ok. 150 passed; 0 failed` in
`tests/basic.rs` (149 before), and `2 / 1 / 1 / 1 / 3` passed with 0 failed
across the other suites. Read off the raw `test result:` lines, no filter on
the counts.

## Graph writes

- **cl-2dhg** (vein, `--source file:src/main.rs`, plus `source
  file:src/framings.rs`) — `` `register-form-prompt` ``. `supports` → cl-j2pk
  (the residue it closes) and → cl-sv7z (the predicate and choke point it
  shares). `depends-on` was refused between claims by the edge matrix;
  `supports` is the legal direction and is the honest one here.
- **Affirms**, scoped with `--to` to the files this work actually re-read:
  cl-j2pk on `src/main.rs` (its `q claim`/`q set` help teaching, unchanged and
  still correct) and on `tests/basic.rs` (its instrument, green); cl-sv7z on
  `src/main.rs` (the prompt point, extended beside it, lane asserted intact).
  cl-vuda is behind on `src/render.rs`, which this arc never opened — **not**
  affirmed.

## Residue, stated

**The scope is the mint, deliberately.** `q set <cl> kind=vein` after the fact
reaches no prompt — exactly as cl-sv7z's arm reaches none. The mint is the
choke point both were built at. cl-vuda is the land-time backstop for the
*kindless* shape; there is no land-time backstop for the *nameless register
species* shape, so a vein whose name the shell ate at mint and whose author
dismissed the question will still ratify nameless at harvest. Whether that
deserves a `render::harvest` arm beside cl-vuda's is a real open question and
is not built here; `src/render.rs` was in the lease, but the ask is a second
mechanism with its own framing and its own ratification, not a line of this
one.

**A shared `register_species(kind)` predicate belongs in `queries.rs`** beside
`backtick_titled` — `matches!(kind, Some("vein") | Some("feature"))` now
appears at two points (`queries::assayable:623` and the new arm).
`src/queries.rs` sat outside this arc's lease, so it stayed inline. This is the
same shape that stopped it-8k3p from building this prompt at all, one file
over.

## user-owned calls:

none.

No fork here contradicts or overturns a standing ruling. dc-grrb (prompts,
never gates) governed the instrument choice and was followed rather than
tested; dc-qvtz supplied the form; dc-dsdm's channel means the new framing
ships composed-for-ratification, which is the dispatcher's judgment at harvest
and not a user call.

## Reflections

The honest discomfort in this one is that the acceptance is a **prompt whose
own premise is that it cannot know if it is right**. Every other floor this
codebase has built refuses something it can prove is wrong. This one fires on a
shape that is very often correct — plenty of veins could legitimately want a
plain title — and the best it can do is ask and say so. I wrote "a deliberately
plain title is a fine answer" into the framing because without that sentence
the prompt reads as an accusation, and an agent under prompt pressure will
invent a backticked name to make it stop. That is exactly the fool's-gold move
the brief warns about, manufactured by the prompt itself. I think the sentence
is load-bearing and would resist trimming it.

The second doubt: the prompt fires on **every** nameless vein mint forever,
while the defect it exists for happens only when someone typed a name and lost
it. The signal-to-noise depends entirely on how often veins are honestly
plain-titled. Scanning the backdrop, essentially every vein in this graph is
backtick-titled, so the noise floor looks near zero today — but that is a
convention holding, not a mechanism, and if the convention ever loosens this
prompt becomes chatter at the busiest surface in the tool. Worth a reading at
some future harvest rather than an assumption now.

A smaller surprise: `depends-on` is illegal claim→claim. I reached for it
reflexively because the brief's legal-rel list reads as a flat menu, but the
edge matrix is typed underneath it. The refusal message named the legal set and
the fix was immediate — good error, no complaint — but a builder reading only
the brief will keep reaching for the wrong verb first.

Finally, the lease boundary cost the cleanest version of this fix twice over,
in the same direction as the arc it inherited from: it-8k3p could not build
this prompt because `src/framings.rs` was outside its write-set, and this arc
could not extract the two-call-site predicate because `src/queries.rs` was
outside its own. Each time the leased set was drawn around the *symptom's*
file rather than the mechanism's neighborhood. I do not think that is a defect
in any single fire — it is what it-8vsm ("the lease shape biases the
instrument") already names, and this arc is one more data point under it.
