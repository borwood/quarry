//! The brief framings (do-g8r4): ratified prose for every section, shipped
//! as engine defaults in these exact words — customizable at setup
//! (th-qv27), never optional, never edited here without ratification
//! (dc-dsdm: depth verbiage ships ratified, never invented silently).
//! Lineage: dc-xfgz (species framing), dc-83nk (the floor line, grown into
//! the preamble), dc-hjad (backdrop framings), dc-ez67 (quest repair).
//! Craft constraints, per dc-dsdm: each framing's first sentence stands
//! alone as the one-line disposition (the area-open shelf renders exactly
//! that); every framing survives rendering above a nearly-empty section.

/// Opens every brief; carries the floor line (dc-83nk).
pub const PREAMBLE: &str = "You woke into this task with no memory of how it came to be — that is the condition of every mind that works here, and this machine is built around it. The graph is a stigmergic memory: durable artifacts — decisions, claims, threads, reports — left by the design conversations and builders before you, linked with earnest effort to be found, in the hope that where links are missing, a smart agent going looking closes the gap. This brief is a derived view of that graph — a floor, not the whole interface. It may read complete; it is not. Reading past it (`q open`, `q find`, `q query`) is expected work, not a detour. Where construction ends, discipline carries: what you build and what you record become the ground the next mind stands on.";

/// The vein species framing, carrying the assay sentence (dc-drr6; landed
/// with the assay office slice, it-swsy).
pub const VEINS: &str = "In a human codebase, continuity lives in people — someone carries the DRY systems, the utilities, the pipelines, and feels \"we already have something for that\" before writing a twin. Here every builder is ephemeral, and a codebase takes the shape of the minds that build it: left uncountered, fragmented and short-lived. Veins are the counterforce — the gold previous sessions won, marked so that what proved durable keeps being developed instead of rediscovered. Build from them as much as possible; the list is ranked programmatically and may be incomplete — scrutinize, `q open`, dig while you plan, before you execute. Where no vein honestly covers you, the other mode is minting one: if extracting a utility or mechanism would serve the agents after you, do it — and make your labor and hopes loud in your report, so the harvester can help your discoveries endure. The one forbidden move is quiet invention — novelty that dies with your session. Each vein carries its assay: ratified means a landing's judge verified it against the diff; asserted means one mind wrote it down and no one has stood behind it since — weigh accordingly, and let load-bearing-but-unassayed sharpen your scrutiny, not stop your build.";

pub const RECEIPTS: &str = "Receipts are the index of what already exists — one per landed capability: what it is, does, and why. This machine grows by accretion, and the fastest way to build wrong is to build a twin of something standing. If your work touches anything a receipt names, find that capability before building its sibling.";

pub const MEASURED: &str = "Measured claims are living facts with an instrument — usually a test that keeps them honest by construction. While the instrument stands green and unchanged, the claim is current; one reading as drifted means the instrument itself changed — re-read it before leaning on the number, because a tuned threshold rewrites what the claim may promise.";

pub const READINGS: &str = "Readings are values taken at a moment to inform a decision that may already be made — sediment, not signal: true of their date, dated by design. Weigh a reading's age like a geologist, not an auditor — its drift is expected stratification, never an alarm. If your decision needs the number current, take a fresh reading; never lean on sediment for load.";

pub const CONTRACTS: &str = "Contracts are promises named symbols have made — behavior a piece of code committed to keep. Others build against these promises without re-reading the code that keeps them. If your change touches a symbol under contract, honor it, or say loudly in your report why it could not be honored.";

pub const DECISIONS: &str = "These are the rulings in force where you're working — the settled will of the user and the sessions before you. Build within them; the matched ones likely govern your task. A ruling that reads wrong for your case is Chesterton's fence: built in a conversation you weren't in, for a reason you may not see. Queue the conflict (`q new thread \"<the conflict>\" --about <area>`), name it in your report — the queue is how this machine changes its mind.";

pub const THREADS: &str = "Open questions around this area — not yours to settle, but they show where thinking is unfinished, and your work may be exactly the evidence one is waiting for. Cite a thread's id in any claim you mint or in your report; the backlink carries your evidence to its next reader. Settling is the user's; noticing is yours.";

pub const DOCS: &str = "Reports and prose left by sessions before you, exactly as they wrote them. Treat them as hints, not law: nothing refreshes a report after it lands, and where one contradicts a live claim or decision, the live node wins completely. Their where-things-live maps can save you archaeology; their conclusions may already be overruled. Take the directions; re-verify the claims.";

/// The your-writes framing splits at do-g8r4's placeholder — "[the expected
/// acts render here, per item: the claim with its species, its source, its
/// legal edges]" — which the brief renderer derives per item between OPEN
/// and CLOSE.
pub const YOUR_WRITES_OPEN: &str = "Your code lands once; what you record about it is read forever. At landing you owe the graph its bones — ";

pub const YOUR_WRITES_CLOSE: &str = "And if your questing found two truths with no recorded relationship: mention what relates, link only what leans — a mention costs nothing and is never wrong; an edge is a load-bearing claim, judged at harvest like the rest of your work (dc-ez67: depends-on, about, source, supports only; settling rels are never yours). These writes are how your work becomes ground instead of history. Claims you mint will be assayed at harvest against your diff — write them to survive that reading. A claim minted to satisfy a prompt is fool's gold; a claim the next builder can stand on is the point.";

/// The mint species prompt (dc-6gn9; ratified 2026-08-17 on it-nmzn):
/// fires when a method-carrying claim mints kindless — the species
/// question asked at the choke point, a prompt, never a gate.
pub const SPECIES_PROMPT: &str = "this claim carries a method - taken once to inform a decision (--kind reading, settles as sediment), or relied on to stay true (--kind measured, lives by its instrument)?";

/// The vein species prompt (dc-grrb shape; composed on it-pgn9, for
/// ratification at its harvest): fires when a claim leading with a
/// registered name mints kindless — species is what the assay ladder
/// rides, so a kindless vein never ratifies. A prompt, never a gate.
pub const VEIN_PROMPT: &str = "this claim leads with a registered name - a mechanism or mandate read off landed code (--kind vein), or the receipt of a landed capability (--kind feature)? Kindless claims never ride the assay ladder.";

/// The harvest ask (composed on it-pgn9, for ratification at its harvest):
/// kindless backtick-titled badge mints enumerate at the judgment seat with
/// the settle command in hand — the silent-repair path dies loud, before
/// ratification passes them by asserted. A prompt, never a gate: landing
/// proceeds regardless (dc-grrb).
pub fn vein_ask_line(n: usize) -> String {
    format!("{} kindless mint(s) under this badge lead with a registered name - the assay ladder rides species, so landing now passes these by asserted. Settle each, then land:", n)
}

/// Species-shaped affirm, instrument-backed (dc-6gn9; ratified 2026-08-17
/// on it-nmzn): a measured claim whose source is a test file — the living
/// method is the instrument; affirm on drift means re-read it.
pub const AFFIRM_INSTRUMENT: &str = "living measurement, instrument drifted - re-read the test: does this claim still describe what it asserts? Your affirm records that reading.";

/// Species-shaped affirm, manual method (dc-6gn9; ratified 2026-08-17 on
/// it-nmzn): the irreducible remainder — liveness probes, field
/// observations, user-facts. Affirm records a re-run, never a re-read.
pub const AFFIRM_MANUAL: &str = "living measurement, manual method - affirm records a re-run, not a re-read: run it, then affirm what you saw.";

/// The behind sediment collapse (dc-6gn9; ratified 2026-08-17 on it-nmzn):
/// drift over readings is expected stratification — it collapses to this
/// count while rot enumerates loud. The lint's signal returns: when behind
/// shows an entry, it means something.
pub fn sediment_line(n: usize) -> String {
    format!("sediment: {} dated reading(s) drifted with their sources - expected stratification, not rot", n)
}

/// The harvest assay line (dc-drr6; ratified 2026-08-17 on it-swsy): the
/// dispatcher's landing judgment already verifies badge-minted claims
/// against the diff — ratification records that act. Shown at the harvest
/// seat and again at the landing act itself.
pub fn assay_harvest_line(n: usize) -> String {
    format!("assay: {} claim(s) minted under this badge ratify with your landing - the diff is the evidence, your judgment is the act. Ratified never means true; refute and blast stand.", n)
}

/// The solo assay line (dc-drr6; ratified 2026-08-17 on it-swsy): a session
/// landing its own work ratifies its own mints — the ratifier is stamped,
/// so one mind stays legible against two-minds-against-evidence.
pub fn assay_solo_line(n: usize) -> String {
    format!("assay: {} claim(s) from this arc self-ratify with your landing - stamped to your hand; one mind, on the record.", n)
}

/// The claim ladder taught at the claim verb (dc-drr6; ratified 2026-08-17
/// on it-swsy).
pub const ASSAY_CLAIM_HELP: &str = "Claims mint asserted. Veins ratify when a landing's judge verifies them against the diff - at harvest, or your own solo landing. Ratified records who assayed, never truth.";

/// The load-bearing-but-never-assayed warning header (dc-drr6; ratified
/// 2026-08-17 on it-swsy): the prospector's warning — visibility as the
/// only pressure. No verify queue, no gates.
pub const ASSAY_WARNING: &str = "load-bearing but never assayed - builds stand on these and no judge has: fool's gold risk rises with weight. Assay on next touch, or refute.";

/// The lean prompt at ready (dc-ez67, dc-grrb; lands with it-6349, for
/// ratification at its harvest): the flip enumerates body-cited decisions
/// and claims with no edge from the item, each beside its ready-made link
/// command — a presence prompt at the station where shaping completes and
/// the design session still holds context; never a gate.
pub const LEAN_HEADER: &str =
    "the body cites these with no edge recorded — a mention references, an edge leans:";

/// The why under the list (it-6349): what an edge does that a mention
/// cannot — the doctrine, not just the enumeration.
pub const LEAN_WHY: &str = "an edge is a load-bearing claim: leaned nodes pin the dispatch brief's READ-FIRST, so the working agent reads them before building. Mention-only is often correct — the judgment is yours, here, while the design context still holds.";

/// The open-ended close (it-6349): the enumeration only knows body
/// citations; the shaper may know leans the body never named — reflection
/// reaches edges the recommendation cannot see.
pub const LEAN_CLOSE: &str = "and past this list: what else does the item stand on that no line surfaced? The enumeration sees only body citations — leans the body never named are yours to record.";

/// The spend-down header at wrap (dc-hzrm; composed on it-gwj7, for
/// ratification at its harvest): opens the sweep over the session's
/// deep-touch list — derived from the event log's per-session acts,
/// ranked by touch depth. The wrap-event cursor makes it once per
/// boundary by construction; an actless session renders nothing.
pub fn spend_down_header(n: usize) -> String {
    format!(
        "spend-down — {} node(s) this session touched deepest, ranked; the boundary is the last reader of what this context holds:",
        n
    )
}

/// The spend-down prompt at wrap (dc-hzrm; composed on it-gwj7, for
/// ratification at its harvest): the boundary itself is the waiter
/// (dc-dty5) — a prompt, never a gate. Direct authorship is the vehicle
/// (the dying context is the best-positioned judge of what it holds);
/// the enrichment attitude compresses the do-dspv 2026-08-18 entry; the
/// Goodhart phrase names volume as the temptation (dc-dsdm).
pub const SPEND_DOWN: &str = "this context dies at the boundary; the graph is what survives it. Over the nodes above, ask: what does this session uniquely hold that the graph lacks? Author it directly, by your own hand, while the context is warm — append the state paragraph, the evidence note, the dogear (q edit <id> --body-file <the full body plus your paragraph>); link what leans. Think stigmergically: what would the next cold session have to re-derive, at what cost and consequence? Hold your enrichments lightly — honest about scope and gaps, skeptical of this session's own perspective as one active slice of experience of the graph — and write for the future cold reader: user reasoning and intent, warnings, the connections that make disparate nodes a constellation, patterns that will return for a repo searching priors rather than inventing. Volume is the temptation (Goodhart): a spend-down is measured by what the next session no longer re-derives, never by how much was written.";

/// The counter-voice, first encounter (dc-hzrm; composed on it-gwj7, for
/// ratification at its harvest): the one ambient line that speaks for the
/// prose channel — every other surface speaks the graph's frame. Fired on
/// graph silence, never the clock; any graph act resets it; a filing
/// session never sees it. Dense once per session (machine-local encounter
/// state), a light phrase after.
pub const COUNTER_VOICE_DENSE: &str = "every other line quarry speaks is the graph's frame; this one speaks for the conversation. Anything floated here — a proposal, a concern, a half-formed concept — that has no node yet? Conversation dies with the session; the graph is what the next mind finds. File it while it is cheap (q new thread \"<it>\" --about <area>; a claim; a dogear on the node it belongs to), or let it go deliberately. This line fires on graph silence, never the clock; any graph act quiets it.";

/// The counter-voice, later encounters (dc-hzrm): the light phrase — the
/// teaching already landed once; repetition at density is habituation fuel.
pub const COUNTER_VOICE_LIGHT: &str =
    "graph silence — anything floated in conversation with no node yet?";

/// The claim species in shelf order (dc-yd9s, dc-6gn9): frontmatter kind,
/// section label, ratified framing. The engine stays kindless-but-kind-aware
/// — kinds outside this table still render, grouped under their own string,
/// with no framing (nothing ratified to say).
pub const SPECIES: [(&str, &str, &str); 5] = [
    ("vein", "VEINS", VEINS),
    ("feature", "RECEIPTS", RECEIPTS),
    ("measured", "MEASURED", MEASURED),
    ("reading", "READINGS", READINGS),
    ("contract", "CONTRACTS", CONTRACTS),
];

/// The one-line disposition: a framing's first sentence, standing alone —
/// the register the area-open shelf affords (dc-dsdm craft constraint:
/// every framing's first sentence must stand alone).
pub fn first_sentence(framing: &str) -> &str {
    match framing.find(". ") {
        Some(i) => &framing[..=i],
        None => framing,
    }
}
