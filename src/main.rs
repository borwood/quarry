use anyhow::Result;
use clap::{Parser, Subcommand};

// Non-fatal output (it-8tcy): every print rides the swallowing macros, so
// a closed pipe ends output quietly and no state work sequenced after a
// print can be skipped by an output failure.
use quarry::{errln, out, outln};

use quarry::coord;
use quarry::model::Node;
use quarry::ops::{self, NewArgs};
use quarry::queries;
use quarry::render;
use quarry::store::Store;

const TOP_HELP: &str = "\
GETTING ORIENTED (session start):
  q query queue      threads awaiting the user, answerable now
  q query ready      items dispatchable right now
  q query shaping    upcoming work and its blockers
  q guide            the judgment layer: when to mint what, provenance, session shape

Run `q <verb> --help` for flags and examples. Files under graph/ are never
edited by hand — every write goes through a verb, which bumps versions,
stamps refs, and enforces the constraints.";

#[derive(Parser)]
#[command(name = "q", version, about = "quarry — a work graph for AI-native development", after_help = TOP_HELP)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Bootstrap graph/ in the current directory
    Init {
        /// Also install the Claude teaching surfaces: the generated skill and
        /// the graph/ guard hook (re-run after upgrading the tool)
        #[arg(long)]
        claude: bool,
    },
    /// Mint a node
    #[command(after_help = "EXAMPLES:
  q new area \"hydrology\"
  q new thread \"finite or pinned oceans?\" --provenance user --status queued --about hydrology
  q new item \"water body graph\" --kind slice --about hydrology --acceptance \"bodies persist\"
  q new doc \"S11 results\" --path docs/spikes/S11-results.md --about hydrology")]
    New {
        /// area | item | thread | decision | claim | doc
        ty: String,
        title: String,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        status: Option<String>,
        /// user | assistant | measured | external (default: derived from actor)
        #[arg(long)]
        provenance: Option<String>,
        /// Subject attachments (node key or file:path) — repeatable
        #[arg(long = "about")]
        about: Vec<String>,
        /// Host-repo path this doc wraps (doc nodes; stamps its blob)
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        method: Option<String>,
        /// Mark ratified by (e.g. user); stamps today's date
        #[arg(long)]
        ratified: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<String>,
        #[arg(long = "acceptance")]
        acceptance: Vec<String>,
        /// Project-declared field, k=v (protocol layer) — repeatable
        #[arg(long = "field")]
        fields: Vec<String>,
        /// Mint-time supports edge (claim/doc → decision/item/claim) — one
        /// command registers a dispatch report against its item
        #[arg(long)]
        supports: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
    /// Execute a gated intent by its one-time token
    Resume { token: String },
    /// Add an edge (stamps the target's version / file blob automatically)
    #[command(after_help = "EXAMPLES:
  q link it-4k7f depends-on th-j9uu       # item waits on a thread
  q link th-j9uu depends-on it-8m2x       # thread waits on its spike
  q link cl-9x2m supports dc-my9w         # evidence a decision leans on
  q link do-3fk2 builds-on cl-9x2m        # a spec stands on this capability (lineage, never status)
  q link it-4k7f about file:src/water/body.rs:42   # blob-stamped file ref")]
    Link {
        src: String,
        /// about | part-of | depends-on | settles | supports | refutes | supersedes | source | builds-on
        rel: String,
        dst: String,
        /// Required to cite a refuted/superseded target (C5)
        #[arg(long)]
        acknowledge: bool,
        #[arg(long)]
        note: Option<String>,
    },
    /// Retire an edge as a logged act (no version bump on either node)
    #[command(after_help = "EXAMPLES:
  q unlink dc-4k7f builds-on dc-9x2m --note \"mislink: minted against the wrong decision\"
Retirement is a logged act, never an erasure: the edge leaves the source's
frontmatter, the event records actor/session/badge and the --note why, and
NEITHER node bumps — bookkeeping, not content (the affirm rationale), so
citers never go behind over housekeeping. Retiring an edge that does not
exist refuses and lists what the source carries.")]
    Unlink {
        src: String,
        /// The rel of the edge to retire (about | part-of | depends-on | …)
        rel: String,
        dst: String,
        /// Why the edge retires (logged on the unlink event)
        #[arg(long)]
        note: Option<String>,
    },
    /// Mutate fields: status=… title=… kind=… method=… path=… acceptance+=…
    /// acceptance-=… write-set+=… ratified=…
    #[command(after_help = "acceptance-= removes a line (it-ds6b): the value matches the exact line,
or any substring matching exactly one — zero or multiple matches refuse,
listing candidates, and the echo and log carry the full resolved line
removed, never what was typed. Replace is both fields in one act:
  q set <item> \"acceptance-=<old>\" \"acceptance+=<new>\"
Stripping a readied item's last line loudly demotes it to shaped
(dc-p6z4). From a non-design seat, subtraction is authoring: the removal
rides the witness review channel until the user ratifies (dc-mpg8).
Unlike unlink, the item BUMPS — the contract is content, not bookkeeping.")]
    Set {
        node: String,
        #[arg(required = true)]
        fields: Vec<String>,
        #[arg(long)]
        note: Option<String>,
    },
    /// Replace the body
    Edit {
        node: String,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        body_file: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
    /// Resolve a thread with a decision (--by user records the user's ruling)
    #[command(after_help = "EXAMPLES:
  q rule th-j9uu \"reservations are per-session leases\" --by user
The decision inherits the thread's subject attachments; the thread resolves
and anything depending on it unblocks. C3: an assistant may not settle a
user-provenance thread without --by user.")]
    Rule {
        thread: String,
        text: String,
        #[arg(long)]
        by: Option<String>,
        #[arg(long)]
        title: Option<String>,
    },
    /// Extract a claim (extraction-on-citation)
    #[command(after_help = format!("EXAMPLES:
  q claim \"halo is 4-11 cells\" --about hydrology --source s11-results --method \"ring differencing\"
  q claim \"`body-graph`: water bodies keep identity across chunk regen\" --about hydrology --source file:src/water/body.rs
Extract a claim only when something depends on the statement or kills it —
never while writing prose. C1: at least one --about. C2: non-user claims
name their --source doc. A landing counts as dependence: register a landed
capability as a VEIN claim (--source file:<the code>), titled name-first
in the project's register (`name`: what it provides) — systematic,
intention-revealing names. Titles feed the relatedness lexicon, so a
well-named vein surfaces itself to future work.
{}", quarry::framings::ASSAY_CLAIM_HELP))]
    Claim {
        text: String,
        /// Full title when the derived first-line cut would truncate it (register-length vein names)
        #[arg(long)]
        title: Option<String>,
        /// Material species (dc-yd9s, dc-6gn9): vein | feature | measured |
        /// reading | contract — any string renders; nothing validates. A
        /// method-carrying claim minted kindless draws the species prompt.
        #[arg(long)]
        kind: Option<String>,
        #[arg(long = "about", required = true)]
        about: Vec<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        method: Option<String>,
        #[arg(long)]
        provenance: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Refute a claim and show its blast radius
    Refute {
        claim: String,
        /// The evidence node (claim or doc)
        #[arg(long)]
        by: String,
        #[arg(long)]
        note: Option<String>,
    },
    /// Re-stamp behind edges after review (--to limits to one target)
    Affirm {
        node: String,
        #[arg(long)]
        to: Option<String>,
    },
    /// Archive a settled node (a flag, never a removal; --undo restores).
    /// Only settled statuses; parents with live children refuse — archive
    /// the leaves and find them through the parent.
    Archive {
        node: String,
        #[arg(long)]
        undo: bool,
    },
    /// Render a node's neighborhood brief (--all includes archived neighbors)
    Open {
        node: String,
        #[arg(long)]
        all: bool,
    },
    /// Render an item's dispatch brief (derived, never hand-written) and log
    /// the render — reserving that item requires a same-session brief (C8)
    #[command(after_help = "EXAMPLES:
  q brief \"water body graph\"
The brief is DERIVED: read-first neighborhood at pinned versions, write-set
from the live lease (complement is do-not-touch), acceptance as the RETURN
spec, protocol riders (on=brief), and the actor rules. If it reads wrong,
fix the graph and re-render — never hand-compose dispatch context.")]
    Brief { item: String },
    /// Per-node history from the event log
    Log { node: String },
    /// Canned queries
    #[command(alias = "q")]
    Query {
        #[command(subcommand)]
        which: Query,
    },
    /// The single-thread topic queue: threads the user works one at a time
    /// (push / pop / front / drop; no subcommand lists it). Working state,
    /// machine-local — ownership itself stays in the graph's queued threads.
    Queue {
        #[command(subcommand)]
        which: Option<QueueCmd>,
    },
    /// Boundary-time lint: what is owed, dangling, in flight, or unrecorded
    Wrap,
    /// Search nodes by id, title, or body text
    Find { text: String },
    /// Session purviews (the committed registry of who covers which areas)
    Session {
        #[command(subcommand)]
        which: SessionCmd,
    },
    /// Lease a write-set for an item at dispatch time (C7)
    #[command(after_help = "EXAMPLES:
  q reserve \"water body graph\" --files \"crates/dc-worldgen/**\"
  q reserve \"sdk docs pass\" --files \"docs/sdk/**\" --shared
--files is REPEATABLE and takes one glob per flag — for a multi-glob
write-set, repeat it (--files \"src/**\" --files \"tests/**\"). A comma-joined
value is refused (it-x4bb): it would lease one dead pattern matching only
the first path in it, denying the rest at write time.
Exclusive by default: an overlapping foreign lease denies, naming the
holder. --shared marks a co-write zone (shared leases coexist, with mutual
visibility). --steal overrides loudly and is logged. Release explicitly
when the arc lands; sessions start leaseless and reserve at dispatch.
A lease follows a brief: reserve refuses unless this session rendered
`q brief <item>` first (C8) — no lease on unbriefed work.")]
    Reserve {
        item: String,
        /// Write-set globs for the lease, one glob per flag, repeatable
        #[arg(long = "files", required = true)]
        files: Vec<String>,
        #[arg(long)]
        shared: bool,
        #[arg(long)]
        steal: bool,
        /// Required with --steal: why the override is justified (logged)
        #[arg(long)]
        reason: Option<String>,
    },
    /// Release an item's lease
    Release { item: String },
    /// Dispatch an item: brief logged + lease + in-flight + a single-use
    /// join token in a one-line spawn prompt, as one act
    #[command(after_help = "EXAMPLES:
  q dispatch \"water body graph\" --files \"crates/dc-worldgen/**\"
  q dispatch \"the parser fix\" --files \"src/**\" --files \"tests/**\"
--files is REPEATABLE and takes one glob per flag — never a comma-joined
list, which is refused (it-x4bb): leased whole it is one dead pattern
matching only the first path in it, and the agent's writes to the rest are
denied mid-arc by its own badge, in a seat that cannot extend a lease.
One act: logs the brief (C8), reserves the write-set (--files, falling
back to the item's recorded write-set), sets in-flight, records the badge
machine-locally with a single-use join token, and prints the ONE-LINE
spawn prompt. The hand-off is a fetch (dc-zbxj): the agent runs
q join <token>, which binds its identity to the badge and renders the
brief fresh from the graph — nothing is hand-carried. The agent reports
and stops; YOU judge and land: q harvest <item>.
Multi-held (dc-qyr5): a chat dispatches any number of items in parallel;
an item belongs to one chat. Re-dispatching your own item mints a fresh
token (the old dies); an item another chat holds live refuses — take it
over whole with --steal --reason \"why\" (loud, logged).
Fire-time routing (dc-crea): from a non-dispatch session, when a live
dispatch-kind session covers the item, nothing fires — the item stays
ready (ready IS the dispatcher feed) and the live session(s) are named;
none awake offers which dispatcher to wake. Advisory, stateless, never a
gate: --solo fires from anywhere, no reason demanded.")]
    Dispatch {
        item: String,
        /// Write-set globs for the lease, one glob per flag, repeatable
        /// (falls back to the item's recorded write-set)
        #[arg(long = "files")]
        files: Vec<String>,
        /// Mark the lease as a co-write zone
        #[arg(long)]
        shared: bool,
        /// Take over another chat's live dispatch of this item, whole (loud, logged)
        #[arg(long)]
        steal: bool,
        /// Required with --steal: why the take-over is justified (logged on the steal event)
        #[arg(long)]
        reason: Option<String>,
        /// Fire from this session even when a dispatcher covers the item
        /// (routing is advisory — fire solo stays legitimate, dc-crea)
        #[arg(long)]
        solo: bool,
    },
    /// Join a dispatch: consume the spawn-prompt token, bind this agent's
    /// identity to the badge, and render the brief fresh from the graph
    #[command(after_help = "The fetch half of the hand-off (dc-zbxj): the spawn prompt is one line —
q join <token> --store <root> — and everything else derives here. The token
is single-use (re-join by the same identity re-prints the brief; a second
identity refuses). One agent, one badge (it-tanf): an identity bound to a
live badge cannot join a second — acts would stamp the last join, not the
item served — so multi-item work is separate dispatches; the refusal leaves
the token live and names the roads out. Identity is hook-injected
(QUARRY_AGENT/QUARRY_CHAT) — outside hook coverage, export
QUARRY_DISPATCH=<item> instead and skip join.
--store is the store pin (dc-g5x5): the canonical graph root, stamped into
the spawn line at dispatch. Join consumes it and plants it for your
identity, so every later q act and observed write lands at that graph
wherever your cwd sits — a worktree fork's own graph/ copy is never
written. Omitted, the store resolves as usual (env pin, then discovery).")]
    Join {
        token: String,
        /// The canonical graph root (the store pin, stamped into the spawn
        /// line at dispatch — dc-g5x5)
        #[arg(long)]
        store: Option<String>,
    },
    /// Harvest a dispatch: observed-vs-leased, badge-stamped acts, report
    /// homework — the dispatcher judges acceptance and lands by hand
    #[command(after_help = "An agent's \"done\" is a stop signal, never a transition: the item stays
in-flight and the lease held until YOU land it (q set <item> status=done ·
q release <item>). Harvest prints the judgment surface and clears the
machine-local badge; a partial or stop report harvests the same way.")]
    Harvest { item: String },
    /// The witness review channel (dc-mpg8): witness-authored acceptance
    /// lines awaiting the user's ratify-or-amend
    #[command(after_help = "A session whose registered kind is not design holds the witness pen:
acceptance it authors is transcription — marked at authoring by
construction and carried on the design wake's review channel until the
user ratifies or amends it. Bare `q witness` lists the channel;
`q witness <item>` shows one item's marks, ratified stamps included.
--ratify records the USER'S word only (--by user — the q rule --by user
channel). --mark is the transcription road for lines that predate the pen:
it marks an existing acceptance line with its true authoring seat
(--author-session), adds review pressure, and can clear nothing.")]
    Witness {
        /// Item to inspect or act on; omitted lists the whole channel
        item: Option<String>,
        /// Mark an existing acceptance line (verbatim) as witness-authored
        #[arg(long)]
        mark: Option<String>,
        /// The session whose seat authored the line (with --mark)
        #[arg(long)]
        author_session: Option<String>,
        /// Ratify the item's witness-authored lines — the user's word only
        #[arg(long)]
        ratify: bool,
        /// Who ratifies (must be user; the agent transcribes, never decides)
        #[arg(long)]
        by: Option<String>,
    },
    /// Render the whole graph as one self-contained HTML page (graph/view/index.html)
    View {
        /// Open the rendered page in the default browser
        #[arg(long)]
        open: bool,
    },
    /// Serve the view over HTTP — every request renders the live graph
    #[command(after_help = "A std-only loop on 127.0.0.1: each request re-runs the view render over
the live store, so a long-lived tab's refresh is always current — no baked
file to go stale, no regeneration act (dc-f79h). Ctrl-C stops it.")]
    Serve {
        /// Port to listen on (default 7171)
        #[arg(long)]
        port: Option<u16>,
    },
    /// Print the judgment-layer primer (when to mint what, provenance, session shape)
    Guide,
    /// Hook entry points (wired by `q init --claude`)
    Hook {
        #[command(subcommand)]
        which: HookCmd,
    },
}

#[derive(Subcommand)]
enum QueueCmd {
    /// Append a thread to the topic queue
    Push { thread: String },
    /// Take the front topic off the queue (does not resolve the thread —
    /// rulings still go through q rule)
    Pop,
    /// Move a queued thread to the front
    Front { thread: String },
    /// Remove a thread from the queue without taking it up
    Drop { thread: String },
}

#[derive(Subcommand)]
enum HookCmd {
    /// PreToolUse guard: denies freehand Write/Edit under graph/ (C6)
    Guard,
    /// SessionStart orientation: one-line graph summary for a fresh session
    Orient,
    /// PreToolUse on Bash|PowerShell: inject the bound session's env
    Session,
}

#[derive(Subcommand)]
enum SessionCmd {
    /// Register or update a session's purview
    Set {
        name: String,
        #[arg(long = "areas", required = true)]
        areas: Vec<String>,
        /// Session kind: design | dispatch (dc-ad8b — the set is open,
        /// validated in one place; surfaces render whatever kind they find)
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        charter: Option<String>,
        /// Also write a <name>-session.cmd launcher at the repo root
        #[arg(long)]
        launcher: bool,
        /// This chat only — not expected to be re-entered (persistence is
        /// the default; ephemeral is the marked case)
        #[arg(long)]
        ephemeral: bool,
    },
    /// List registered sessions and their areas
    List,
    /// The derived wake brief: holdings, your recent acts, arrivals, owed
    Resume,
    /// Bind THIS chat to a session when no launcher env is set (takes effect
    /// on the next tool call, via the session hook)
    Adopt { name: String },
    /// Retire a session: registry entry removed, leases released, last
    /// rites logged (the default close act for an ephemeral session)
    Retire { name: String },
}

#[derive(Subcommand)]
enum Query {
    /// Items dispatchable right now
    Ready {
        /// Restrict to the current session's purview (QUARRY_SESSION)
        #[arg(long)]
        mine: bool,
    },
    /// Upcoming work (sketch/shaped) and what blocks each piece
    Shaping {
        #[arg(long)]
        mine: bool,
    },
    /// Threads awaiting the user, answerable now
    Queue {
        #[arg(long)]
        mine: bool,
    },
    /// Stale edges by severity
    Behind,
    /// Who leans on this node (transitively)
    Blast { node: String },
    /// A node's homework, re-derived now: citers behind it (each with the
    /// affirm command that clears it) and work it unblocks — the same
    /// derivation every mutating verb prints at act time; that print is a
    /// delivery, never the only copy (it-8tcy)
    Homework { node: String },
    /// Assistant writes against user provenance (should be empty — C3)
    Contested,
    /// Nodes nothing points at
    Idle {
        #[arg(long, default_value_t = 14)]
        days: i64,
    },
    /// Assistant claims never verified
    Unverified,
    /// Load-bearing but never assayed: builds stand on these and no judge
    /// has — the prospector's warning (dc-drr6), heaviest first
    #[command(visible_alias = "unassayed")]
    Load,
    /// Items with no acceptance lines — derived at read, never stored
    /// (dc-p6z4): authoring acceptance clears it by construction. Any live
    /// status; the gate holds shaped ones from ready
    #[command(visible_alias = "awaiting")]
    AwaitingAcceptance,
    /// Live kind=bug items, the shaping stratum leading — defects are
    /// bugs on sight and fix with urgency (dc-ygzz); the design wake
    /// counts the shaping stratum whenever nonzero
    #[command(visible_alias = "bugs")]
    Defects,
    /// The uncommitted graph split by owner — the log is the ownership
    /// oracle: a node file belongs to the session with the last event on
    /// it since the prior commit. Foreign sets carry the owner's last-seen
    /// age; a stale owner's set flips to an explicit-path adoption offer.
    /// The log shard is exempt ledger and rides along with any commit
    #[command(name = "commit-set")]
    CommitSet,
    /// What a dispatch wrote: badge-stamped events and guard-observed files
    Dispatch { item: String },
    /// Intent vs reality by name: backtick-named acceptance lines of live
    /// items against vein claim titles in shared areas — intended-but-
    /// unlanded and landed-but-unintended, both directions
    IntentDelta {
        /// Scope to one area (default: every area)
        area: Option<String>,
    },
}

fn read_body(body: Option<String>, body_file: Option<String>) -> Result<String> {
    match (body, body_file) {
        (Some(b), _) => Ok(b),
        (None, Some(f)) => Ok(std::fs::read_to_string(f)?),
        (None, None) => Ok(String::new()),
    }
}

/// The full `new` intent, serializable so a gate can save and resume it.
#[derive(serde::Serialize, serde::Deserialize)]
struct NewCliArgs {
    ty: String,
    title: String,
    kind: Option<String>,
    status: Option<String>,
    provenance: Option<String>,
    about: Vec<String>,
    path: Option<String>,
    method: Option<String>,
    ratified: Option<String>,
    body: Option<String>,
    body_file: Option<String>,
    acceptance: Vec<String>,
    fields: Vec<String>,
    #[serde(default)]
    supports: Option<String>,
    note: Option<String>,
}

/// Mint-time surfaces: the bodyless-sketch nudge and the relatedness
/// touches-line — an index for judgment, silence the default state.
fn print_mint_surfaces(store: &Store, node: &Node) {
    if node.front.ty == "item" && node.body.trim().is_empty() {
        outln!(
            "  note: bodyless sketch — a title-only node leaves the next reader nothing to open; one sentence of intent is the floor: q edit {} --body \"...\"",
            node.front.id
        );
    }
    // The species prompt (dc-6gn9, ratified on it-nmzn): a method-carrying
    // claim minted kindless gets the species question at the choke point —
    // a prompt, never a gate; the mint already stands.
    if node.front.ty == "claim" && node.front.kind.is_none() && node.front.method.is_some() {
        outln!("  species: {}", quarry::framings::SPECIES_PROMPT);
        outln!("    settle it: q set {} kind=reading (or kind=measured)", node.front.id);
    } else if node.front.ty == "claim"
        && node.front.kind.is_none()
        && quarry::queries::backtick_titled(quarry::surface::title_raw(node))
    {
        // The vein prompt (it-pgn9, dc-grrb shape): a kindless mint leading
        // with a registered name is vein-shaped — the species question at the
        // same choke point, a prompt, never a gate. The method prompt above
        // keeps its lane: a method-carrying claim is measurement-shaped, and
        // the two questions never stack.
        outln!("  species: {}", quarry::framings::VEIN_PROMPT);
        outln!("    settle it: q set {} kind=vein (or kind=feature)", node.front.id);
    }
    let Ok(all) = store.load_all() else { return };
    let touches = quarry::queries::relatedness(&all, node);
    if !touches.is_empty() {
        outln!("  touches — review and judge; link only what genuinely relates:");
        for (t, why) in touches {
            outln!("    {} — {}", line(&all, t), why);
        }
    }
    print_mention_surfaces(&all, node);
}

/// The body-citation echo (dc-wwnk): every resolved id's current title beside
/// it — the echo IS the verification; a wrong-but-real id reads wrong here,
/// which no dangling warning can catch. Id-shapes resolving to nothing ask a
/// question, never gate (hyphenated prose false-positives exist); the
/// real-edge upgrade is offered as judgment, never auto-linked.
fn print_mention_surfaces(all: &[Node], node: &Node) {
    let cited = quarry::mention::cited_ids(&node.body);
    if cited.is_empty() {
        return;
    }
    let mut resolved: Vec<&Node> = Vec::new();
    let mut dangling: Vec<String> = Vec::new();
    for id in &cited {
        match all.iter().find(|n| &n.front.id == id) {
            Some(t) if t.front.id != node.front.id => resolved.push(t),
            Some(_) => {} // self-mention: nothing to verify
            None => dangling.push(id.clone()),
        }
    }
    if !resolved.is_empty() {
        outln!("  body cites — read the echo; a wrong-but-real id reads wrong here:");
        for t in &resolved {
            outln!("    {}", aref(all, t));
        }
        outln!(
            "    (render unpacks these; a mention references, an edge leans — if this stands on one, record it: q link {} <rel> <id>)",
            node.front.id
        );
    }
    for d in &dangling {
        outln!(
            "  {} is id-shaped but resolves to nothing — a citation to fix, or hyphenated prose to leave as is? (wrap lints danglers)",
            d
        );
    }
}

/// The lean prompt at ready (dc-ez67, dc-grrb; it-6349): fires when an item
/// reaches ready — the station where shaping completes and the design
/// session still holds context. Enumerates body-cited decisions and claims
/// with no edge from the item, each beside its ready-made link command,
/// teaches the why (a mention references, an edge leans, leaned nodes pin
/// the brief's READ-FIRST), and closes open-ended — the enumeration only
/// knows body citations; the shaper may know leans the body never named.
/// A presence prompt, never a gate: mention-only is often correct, and the
/// judgment is the shaper's. Silence when nothing un-edged is cited.
fn print_lean_prompt(store: &Store, item: &Node) {
    let Ok(all) = store.load_all() else { return };
    let unleaned = quarry::queries::unleaned_citations(&all, item);
    if unleaned.is_empty() {
        return;
    }
    outln!("  {}", quarry::framings::LEAN_HEADER);
    for (t, cmd) in &unleaned {
        outln!("    · {} — if this stands on it: {}", aref(&all, t), cmd);
    }
    outln!("    {}", quarry::framings::LEAN_WHY);
    outln!("    {}", quarry::framings::LEAN_CLOSE);
}

/// The reader scope phrase for attention surfaces: badge-scoped attention
/// belongs to the dispatch arc, session attention to the session (dc-pwyd).
fn attention_scope(reader: &str) -> &'static str {
    if reader.starts_with("badge:") { "this dispatch arc" } else { "this session" }
}

/// The per-area watermark surface, run after a mutating verb touched a node.
/// Keyed on the acting identity's attention key (dc-pwyd: a joined agent
/// spends badge-scoped attention, never the holding session's). First touch
/// of an unread area nudges once; foreign drift since the recorded read
/// prints inline (the delta IS the delivery); own writes and quiet checks
/// advance the cursor silently.
fn area_watermarks(store: &Store, node_id: &str) {
    let reader = coord::attention_key(store);
    let Ok(all) = store.load_all() else { return };
    let Ok(node) = store.find(&all, node_id) else { return };
    let areas: Vec<(String, String)> = node
        .front
        .edges
        .iter()
        .filter(|e| e.rel == "about")
        .filter_map(|e| {
            all.iter()
                .find(|n| n.front.id == e.to && n.front.ty == "area")
                .map(|n| (n.front.id.clone(), aref(&all, n)))
        })
        .collect();
    for (aid, area_ref) in areas {
        match coord::touch_area(store, &all, &reader, &aid) {
            coord::AreaTouch::FirstTouch => {
                outln!(
                    "  note: first touch of area {} {} without a read — the read-first: q open {}",
                    area_ref,
                    attention_scope(&reader),
                    aid
                );
                coord::record_area_read(store, &reader, &aid);
            }
            coord::AreaTouch::Drift(lines) => {
                outln!("  since your last read of {} (other hands):", area_ref);
                for l in lines {
                    outln!("    · {}", l);
                }
            }
            coord::AreaTouch::Current => {}
        }
    }
}

/// Land-time landmark check (ratified 2026-08-09): when an item lands, does
/// anything in the graph cite the files it held? PRESENCE of citation only,
/// and a prompt, never a gate — a blocked done breeds Goodhart claims.
fn vein_check(store: &Store, item: &Node, globs: Option<Vec<String>>) {
    if item.front.ty != "item" {
        return;
    }
    let Ok(all) = store.load_all() else { return };
    // Observed files win when present: what the guard actually saw touched
    // is truer than what the lease predicted.
    let observed = coord::touched_for(store, &format!("item:{}", item.front.id));
    let globs = if !observed.is_empty() {
        observed
    } else {
        globs.unwrap_or_else(|| {
            coord::load_leases(store)
                .iter()
                .find(|l| l.item == item.front.id)
                .map(|l| l.globs.clone())
                .unwrap_or_else(|| item.front.write_set.clone())
        })
    };
    if globs.is_empty() || quarry::queries::files_cited(&all, &globs) {
        return;
    }
    outln!(
        "  landed uncited: no claim or doc cites {:?}. If this work left a durable capability, register its vein while the diff is warm:",
        globs
    );
    outln!("    q claim \"`capability-name`: what it now provides\" --about <area> --source file:<path>");
    outln!("  Name it — a `named` capability reasons better than a description, and every build that cites it surfaces by blast.");
    outln!("  Skip freely if nothing durable landed — a claim minted to silence this line is Goodhart, worse than silence. Presence is checked; quality is judged at review.");
}

/// The area-first-touch gate (user-agreed 2026-08-09): minting into an area
/// this reader has never read intercepts once, delivers the area's derived
/// read-first, and saves the intent for q resume. A prior same-reader
/// `q open <area>` passes silently — the gate is the backstop, not the
/// path. The reader is the acting identity's attention key (dc-pwyd): a
/// joined agent gates on its own eyes — the brief already recorded its
/// item's areas at join — and a session gates on its own, however many
/// arcs it dispatched into the area meanwhile.
fn area_gate_if_needed(store: &Store, a: &NewCliArgs) -> Result<bool> {
    let reader = coord::attention_key(store);
    let all = store.load_all()?;
    let mut unread: Vec<(String, String)> = Vec::new();
    for key in &a.about {
        if key.starts_with("file:") {
            continue;
        }
        let Ok(n) = store.find(&all, key) else { continue };
        if n.front.ty == "area" && !coord::has_area_read(store, &reader, &n.front.id) {
            unread.push((n.front.id.clone(), aref(&all, n)));
        }
    }
    if unread.is_empty() {
        return Ok(false);
    }
    let token = quarry::protocol::save_intent(store, "new", serde_json::to_value(a)?)?;
    outln!(
        "⏸ gated: first write into area(s) unread {} — the read-first arrives now.",
        attention_scope(&reader)
    );
    for (aid, area_ref) in &unread {
        outln!("\n── area {} ──", area_ref);
        out!("{}", render::open(store, aid, false)?);
        coord::record_area_read(store, &reader, aid);
    }
    outln!("\nYour intent is saved. Read the above, then run: q resume {}", token);
    outln!(
        "(args are remembered; delivery is recorded — {} will not be gated on these areas again)",
        attention_scope(&reader)
    );
    Ok(true)
}

fn do_new(store: &Store, a: NewCliArgs) -> Result<()> {
    if area_gate_if_needed(store, &a)? {
        std::process::exit(2);
    }
    let supports = a.supports.clone();
    let body = read_body(a.body, a.body_file)?;
    let node = ops::new_node(
        store,
        NewArgs {
            ty: a.ty,
            title: a.title,
            kind: a.kind,
            status: a.status,
            provenance: a.provenance,
            about: a.about,
            path: a.path,
            method: a.method,
            ratified_by: a.ratified,
            body,
            acceptance: a.acceptance,
            fields: a.fields,
            note: a.note,
        },
    )?;
    outln!("✔ {}", line(&store.load_all().unwrap_or_default(), &node));
    if let Some(target) = supports {
        let edge = ops::link(store, &node.front.id, "supports", &target, false, None)?;
        outln!("  ✔ {} -[supports]-> {} (at {})", node.front.id, edge.to, edge.at);
    }
    let filed = node.front.ty == "area"
        || node.front.edges.iter().any(|e| e.rel == "about" && e.to.starts_with("ar-"));
    if !filed {
        outln!(
            "  note: unfiled — the map cannot place it. Attach it: q link {} about <area>",
            node.front.id
        );
    }
    print_mint_surfaces(store, &node);
    // Mint-to-ready is the other construction path to ready (dc-p6z4's
    // inventory): the lean prompt fires wherever ready is reached (it-6349).
    if node.front.ty == "item" && node.front.status == "ready" {
        print_lean_prompt(store, &node);
    }
    area_watermarks(store, &node.front.id);
    if let Ok(all) = store.load_all() {
        for (title, text) in quarry::protocol::inline_texts(
            &all,
            "new",
            Some(node.front.ty.as_str()),
            node.front.kind.as_deref(),
        ) {
            outln!("\nprotocol — {}:", title);
            for l in text.lines() {
                outln!("  {}", l);
            }
        }
    }
    Ok(())
}

fn print_gate(g: &quarry::protocol::Gate) {
    outln!("⏸ gated: this act carries project protocol, delivered once per session.");
    for (title, body) in &g.rules {
        outln!("\n── {} ──", title);
        for l in body.lines() {
            outln!("{}", l);
        }
    }
    outln!("\nYour intent is saved. Do the work under the protocol, then run: q resume {}", g.token);
    outln!("(args are remembered; to change them, re-run the original command — this session is now cleared for this rule)");
}

fn human_age(secs: i64) -> String {
    if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// atom_line for a list entry — the thin bridge onto the surface module
/// (dc-nnf5): registers differ in how much of the atom they render, never
/// in where they are built.
fn line(all: &[Node], n: &Node) -> String {
    quarry::surface::atom_line(&quarry::surface::atom(all, n))
}

/// atom_ref for a sentence — the floor every refusal, confirmation, and
/// composed line stands on.
fn aref(all: &[Node], n: &Node) -> String {
    quarry::surface::atom_ref(&quarry::surface::atom(all, n))
}

/// After a mutation, report the homework it created: citers now behind (with
/// the command that clears each after review) and work the change unblocked.
/// A DELIVERY of a derivable surface, never the only copy (it-8tcy): the
/// derivation lives in render::homework, and `q query homework <node>`
/// re-derives it on demand — a pipe that dies mid-print loses nothing.
fn print_homework(store: &Store, touched: &[&str]) {
    let Ok(all) = store.load_all() else { return };
    let lines = render::homework(&all, touched);
    if !lines.is_empty() {
        outln!("homework (re-derivable any time: q query homework <node>):");
        for l in lines {
            outln!("  {}", l);
        }
    }
}

/// Archive-on-consumption (dc-6gn9): run after any settling act — a reading
/// whose last live consumer just settled archives itself, and this surface
/// says what was hidden. Automation replaces agent discipline where only
/// one end state exists.
fn sweep_readings(store: &Store) {
    let Ok(swept) = ops::consume_readings(store) else { return };
    if swept.is_empty() {
        return;
    }
    let all = store.load_all().unwrap_or_default();
    for n in &swept {
        outln!(
            "  ⚑ reading archived on consumption — its last live consumer settled: {}",
            aref(&all, n)
        );
    }
}

/// Presence: warn when another session touched this node recently (24h).
fn presence_note(store: &Store, id: &str) {
    let Some(mine) = coord::current_session() else { return };
    let Ok(log) = store.read_log() else { return };
    let cutoff = {
        use time::format_description::well_known::Rfc3339;
        let t = time::OffsetDateTime::now_utc() - time::Duration::days(1);
        t.format(&Rfc3339).unwrap_or_default()
    };
    if let Some(ev) = log.iter().rev().find(|ev| {
        ev.get("node").and_then(|v| v.as_str()) == Some(id)
            && ev.get("ts").and_then(|v| v.as_str()).map_or(false, |ts| ts > cutoff.as_str())
            && ev
                .get("session")
                .and_then(|v| v.as_str())
                .map_or(false, |s| s != mine)
    }) {
        let ts = ev.get("ts").and_then(|v| v.as_str()).unwrap_or("?");
        let sess = ev.get("session").and_then(|v| v.as_str()).unwrap_or("?");
        let op = ev.get("op").and_then(|v| v.as_str()).unwrap_or("?");
        outln!("  note: session {} touched this node ({} at {})", sess, op, ts);
    }
}

/// Purview-filter a node list when --mine is set; explain if unset/unregistered.
fn scope_mine<'a>(
    store: &Store,
    all: &'a [Node],
    nodes: Vec<&'a Node>,
    mine: bool,
) -> Vec<&'a Node> {
    if !mine {
        return nodes;
    }
    match coord::purview(store, all) {
        Some((_, areas)) => {
            let ids: Vec<&str> = areas.iter().map(|a| a.front.id.as_str()).collect();
            nodes
                .into_iter()
                .filter(|n| coord::in_purview(n, &ids))
                .collect()
        }
        None => {
            errln!("(--mine ignored: set QUARRY_SESSION and register it with q session set)");
            nodes
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init { claude } => {
            let cwd = std::env::current_dir()?;
            Store::init(&cwd)?;
            outln!("✔ graph/ initialized at {}", cwd.display());
            outln!("  suggested .gitignore lines: graph/.index/  graph/view/  graph/.*  (machine-local state — leases, cursors, the touched-set, the dispatch badge)");
            if claude {
                for a in quarry::teach::install_claude(&cwd)? {
                    outln!("  ✔ {}", a);
                }
                outln!("  note: the hook names this q binary by absolute path — re-run `q init --claude` if the binary moves.");
            }
        }
        Cmd::View { open } => {
            let store = Store::resolve()?;
            let path = quarry::view::write(&store)?;
            outln!("✔ rendered {}", path.display());
            if open {
                // If a serve loop is up on the default port, open that
                // instead — the served page is always current; the baked
                // file stays as the offline courtesy (dc-5pb3, dc-f79h).
                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], quarry::view::DEFAULT_PORT));
                let served =
                    std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(200))
                        .is_ok();
                let target: std::ffi::OsString = if served {
                    let url = format!("http://127.0.0.1:{}/", quarry::view::DEFAULT_PORT);
                    outln!("  a serve loop is up — opening {} instead of the baked file", url);
                    url.into()
                } else {
                    path.clone().into_os_string()
                };
                #[cfg(windows)]
                std::process::Command::new("cmd")
                    .args(["/C", "start", ""])
                    .arg(&target)
                    .spawn()?;
                #[cfg(not(windows))]
                std::process::Command::new("open").arg(&target).spawn()?;
            }
        }
        Cmd::Serve { port } => {
            let store = Store::resolve()?;
            let port = port.unwrap_or(quarry::view::DEFAULT_PORT);
            let listener = std::net::TcpListener::bind(("127.0.0.1", port))?;
            outln!(
                "✔ serving the view at http://127.0.0.1:{} — every request renders the live graph; Ctrl-C stops",
                port
            );
            quarry::view::serve(&store, listener)?;
        }
        Cmd::Guide => out!("{}", quarry::teach::GUIDE),
        Cmd::Hook { which } => match which {
            HookCmd::Guard => {
                use std::io::Read as _;
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input)?;
                if let Some(msg) = quarry::teach::guard(&input) {
                    errln!("{}", msg);
                    std::process::exit(2);
                }
                // The lease layer (dispatch chain): best-effort, never fails
                // a session over a missing graph. Identity is parsed FIRST:
                // the store resolves through the one resolver (dc-g5x5) with
                // the hook input's identity and cwd in hand — a hook process
                // never sees the shell injection's env, so the identity pin
                // is its road to a pinned store.
                let parsed = serde_json::from_str::<serde_json::Value>(&input).ok();
                let chat = parsed.as_ref().and_then(|v| {
                    v.get("session_id").and_then(|x| x.as_str()).map(String::from)
                });
                // The agent id rides the hook input in subagents — the
                // subagent's only distinguishing identity (its session_id
                // matches the parent chat's).
                let agent = parsed.as_ref().and_then(quarry::teach::hook_agent_id);
                let hook_keys: Vec<String> = [
                    agent.as_ref().map(|a| format!("agent:{}", a)),
                    chat.as_ref().map(|c| format!("chat:{}", c)),
                ]
                .into_iter()
                .flatten()
                .collect();
                let hook_cwd = parsed.as_ref().and_then(|v| {
                    v.get("cwd").and_then(|x| x.as_str()).map(std::path::PathBuf::from)
                });
                if let (Some(path), Ok(store)) = (
                    quarry::teach::write_target(&input),
                    quarry::store::resolve_store(None, &hook_keys, hook_cwd.as_deref()),
                ) {
                    // The lease layer judges REPO-RELATIVE paths only: a
                    // write outside the host repo (scratchpads, temp files)
                    // is never scope creep, never contract material.
                    // Resolution rides the ONE point (store::store_relative,
                    // it-bj3b): case-folded, separator-normalized, work root
                    // stripped first (dc-g5x5 — a fork mirrors the layout).
                    let rel = store.relative(&path);
                    let session = coord::current_session().or_else(|| {
                        chat.as_deref().and_then(|cid| coord::chat_binding(&store, cid))
                    });
                    // The badge resolves for the ACTING identity, never
                    // the machine and never the held entry: env, then the
                    // association q join bound (agent, chat, session
                    // keyed) — stamping and the guard follow the WORK
                    // (dc-zbxj); another chat's dispatch is not this
                    // chat's badge.
                    let dispatch =
                        coord::badge_for(&store, agent.as_deref(), chat.as_deref(), session.as_deref());
                    if let Some(rel) = rel {
                        let dispatched: Vec<String> = coord::load_dispatches(&store)
                            .held
                            .values()
                            .map(|d| d.item.clone())
                            .collect();
                        let leases = coord::load_leases(&store);
                        let mut context: Vec<String> = Vec::new();
                        match quarry::teach::lease_check(
                            &leases,
                            session.as_deref(),
                            dispatch.as_deref(),
                            &dispatched,
                            &rel,
                        ) {
                            quarry::teach::LeaseCheck::Deny(msg) => {
                                errln!("{}", msg);
                                std::process::exit(2);
                            }
                            quarry::teach::LeaseCheck::Warn(msg) => context.push(msg),
                            quarry::teach::LeaseCheck::Allow => {}
                        }
                        // Observation, never denial: accrue the touch, echo
                        // the contract on the first badged write, notice
                        // drift, nudge the leaseless arc at threshold.
                        context.extend(quarry::teach::observe_write(
                            &store,
                            &leases,
                            session.as_deref(),
                            dispatch.as_deref(),
                            &rel,
                        ));
                        if !context.is_empty() {
                            outln!(
                                "{}",
                                serde_json::json!({"hookSpecificOutput": {
                                    "hookEventName": "PreToolUse",
                                    "additionalContext": context.join("\n")
                                }})
                            );
                        }
                    } else if dispatch.is_some() {
                        // NO SILENT DISCARD UNDER A BADGE (it-bj3b): a tool
                        // write whose absolute path failed store-relative
                        // resolution is still a certain write under this
                        // badge — observe_write records the raw path marked
                        // unresolved, and harvest renders it. The lease
                        // layer stays out (out-of-repo paths were never
                        // contract material); relative-shaped strays skip,
                        // exactly as before.
                        let p = path.replace('\\', "/");
                        if p.starts_with('/') || p.contains(':') {
                            quarry::teach::observe_write(
                                &store,
                                &[],
                                session.as_deref(),
                                dispatch.as_deref(),
                                &p,
                            );
                        }
                    }
                }
            }
            HookCmd::Session => {
                use std::io::Read as _;
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input)?;
                // The one resolver, hook identity in hand (dc-g5x5): a
                // joined agent's session hook must consult the PINNED store
                // for bindings and alerts even when its cwd sits in a
                // worktree fork.
                let parsed = serde_json::from_str::<serde_json::Value>(&input).ok();
                let agent = parsed.as_ref().and_then(quarry::teach::hook_agent_id);
                let hook_keys: Vec<String> = [
                    agent.as_ref().map(|a| format!("agent:{}", a)),
                    parsed
                        .as_ref()
                        .and_then(|v| v.get("session_id").and_then(|x| x.as_str()))
                        .map(|c| format!("chat:{}", c)),
                ]
                .into_iter()
                .flatten()
                .collect();
                let hook_cwd = parsed.as_ref().and_then(|v| {
                    v.get("cwd").and_then(|x| x.as_str()).map(std::path::PathBuf::from)
                });
                if let Ok(store) =
                    quarry::store::resolve_store(None, &hook_keys, hook_cwd.as_deref())
                {
                    // The commit-sweep guard (it-4q6t), ahead of injection:
                    // bulk staging that would capture another session's
                    // uncommitted node files denies with the asker's own
                    // git add line in hand. The asking session resolves
                    // like the write guard's: launcher env, then the chat
                    // binding — the hook process never sees the shell
                    // injection's env.
                    let tool = parsed
                        .as_ref()
                        .and_then(|v| v.get("tool_name").and_then(|x| x.as_str()))
                        .unwrap_or("");
                    let cmd = parsed
                        .as_ref()
                        .and_then(|v| v.get("tool_input"))
                        .and_then(|ti| ti.get("command"))
                        .and_then(|c| c.as_str());
                    if let Some(cmd) = cmd {
                        let chat = parsed
                            .as_ref()
                            .and_then(|v| v.get("session_id").and_then(|x| x.as_str()));
                        let session = coord::current_session()
                            .or_else(|| chat.and_then(|cid| coord::chat_binding(&store, cid)));
                        if let Some(msg) = quarry::teach::staging_guard(
                            &store,
                            tool,
                            cmd,
                            hook_cwd.as_deref(),
                            session.as_deref(),
                        ) {
                            errln!("{}", msg);
                            std::process::exit(2);
                        }
                        // The shell half of the sight boundary (it-bj3b):
                        // parse the command for common write shapes and
                        // accrue targets that resolve store-relative,
                        // marked shell-parsed. Observation only — a parse
                        // is best-effort and never denies, never speaks.
                        if matches!(tool, "Bash" | "PowerShell") {
                            let dispatch = coord::badge_for(
                                &store,
                                agent.as_deref(),
                                chat,
                                session.as_deref(),
                            );
                            quarry::teach::observe_shell(
                                &store,
                                dispatch.as_deref(),
                                session.as_deref(),
                                cmd,
                                hook_cwd.as_deref(),
                            );
                        }
                    }
                    if let Some(out) = quarry::teach::session_hook_output(&store, &input) {
                        outln!("{}", serde_json::to_string(&out)?);
                    }
                }
            }
            HookCmd::Orient => {
                // Best-effort: a hook must never fail a session over a missing graph.
                let (chat_id, model, agent_id, hook_cwd) = {
                    use std::io::{IsTerminal, Read as _};
                    let mut s = String::new();
                    if !std::io::stdin().is_terminal() {
                        let _ = std::io::stdin().read_to_string(&mut s);
                    }
                    let v = serde_json::from_str::<serde_json::Value>(&s).ok();
                    (
                        v.as_ref()
                            .and_then(|v| v.get("session_id").and_then(|x| x.as_str()).map(String::from)),
                        v.as_ref()
                            .and_then(|v| v.get("model").and_then(|x| x.as_str()).map(String::from)),
                        v.as_ref().and_then(quarry::teach::hook_agent_id),
                        v.as_ref().and_then(|v| {
                            v.get("cwd").and_then(|x| x.as_str()).map(std::path::PathBuf::from)
                        }),
                    )
                };
                // The one resolver with the wake's identity in hand
                // (dc-g5x5): a pinned identity orients over the pinned
                // graph, wherever this session's cwd sits.
                let hook_keys: Vec<String> = [
                    agent_id.as_ref().map(|a| format!("agent:{}", a)),
                    chat_id.as_ref().map(|c| format!("chat:{}", c)),
                ]
                .into_iter()
                .flatten()
                .collect();
                if let Ok(store) =
                    quarry::store::resolve_store(None, &hook_keys, hook_cwd.as_deref())
                {
                    if let (Some(cid), Some(m)) = (&chat_id, &model) {
                        coord::record_chat_actor(&store, cid, m);
                    }
                    if let Ok(all) = store.load_all() {
                        let queue = queries::queue(&all);
                        let ready = queries::ready(&all);
                        let behind = queries::behind(&store, &all);
                        // The wake session (it-sumw): env identity first
                        // (launcher-owned), then the chat binding (adopt) —
                        // a bound chat orients AS its session, so the
                        // charter meets every wake, not only launcher-env
                        // chats. Resolved before anything prints: the wake
                        // SHAPE follows the session's kind (it-wub5), and
                        // this surface renders the shape it is handed —
                        // coord::wake_shape is the one kind match point.
                        let reg = coord::load_sessions(&store);
                        let wake = coord::current_session().or_else(|| {
                            chat_id.as_deref().and_then(|cid| coord::chat_binding(&store, cid))
                        });
                        let wake_reg =
                            wake.as_ref().and_then(|s| reg.get(s).map(|p| (s.as_str(), p)));
                        let shape =
                            coord::wake_shape(wake_reg.and_then(|(_, p)| p.kind.as_deref()));
                        // Sediment stays out of the wants-action count
                        // (dc-6gn9): behind means rot and breakage; the
                        // strata report themselves beside it.
                        let rot = behind.iter().filter(|b| !b.sediment).count();
                        let sed = behind.len() - rot;
                        outln!(
                            "quarry: {} nodes · owed to the user: {} · ready to dispatch: {} · behind: {}{}",
                            all.len(),
                            queue.len(),
                            ready.len(),
                            rot,
                            if sed > 0 { format!(" · sediment: {}", sed) } else { String::new() }
                        );
                        // Omitted under the dispatch shape: threads are not
                        // a dispatch session's to settle (dc-wngq).
                        if shape.owed_threads {
                            for n in &queue {
                                outln!("  owed: {}", line(&all, n));
                            }
                            // The gate's pressure surface (dc-p6z4), beside
                            // owed threads: a gate-demoted item has a real
                            // waiter — a dispatcher tried to fire it.
                            // Pressure cuts at shaped; sketches stay quiet.
                            let awaiting = queries::awaiting_acceptance(&all)
                                .iter()
                                .filter(|n| n.front.status == "shaped")
                                .count();
                            if awaiting > 0 {
                                outln!(
                                    "  awaiting acceptance: {} shaped item(s) with no acceptance lines — the gate holds them from ready (q query awaiting-acceptance)",
                                    awaiting
                                );
                            }
                            // The assay's pressure surface (dc-drr6,
                            // it-fwn3), beside owed threads: judging a
                            // claim is a design-session act, and load-
                            // bearing is the waiter test (dc-p6z4) —
                            // builds stand on the claim. Zero-holds
                            // unassayed stays off the wake by
                            // construction: the query is weight-filtered.
                            let unassayed = queries::load_bearing_unassayed(&all).len();
                            if unassayed > 0 {
                                outln!(
                                    "  load-bearing unassayed: {} claim(s) builds stand on with no judge on record — fool's gold risk rises with weight (q query load)",
                                    unassayed
                                );
                            }
                            // The standing-ruling earner (dc-dty5, dc-ygzz):
                            // defects are bugs on sight and fix with urgency
                            // — every kind=bug in shaping is waitered by
                            // fiat. Zero renders nothing.
                            let defects = queries::defects(&all)
                                .into_iter()
                                .filter(|n| queries::in_shaping(n))
                                .count();
                            if defects > 0 {
                                outln!(
                                    "  defects in shaping: {} bug(s) below ready — defects fix with urgency, the standing ruling waiters them (q query defects)",
                                    defects
                                );
                            }
                            // The witness review channel (dc-mpg8), beside
                            // owed threads: witness-authored acceptance
                            // waits on the user's ratify-or-amend, and the
                            // design session is the one liaison the user
                            // attends. Zero renders nothing.
                            let witness = queries::witness_flags(&all).len();
                            if witness > 0 {
                                outln!(
                                    "  witness-authored acceptance under review: {} line(s) authored from a non-design seat await the user's ratify-or-amend (q witness)",
                                    witness
                                );
                            }
                            // The hunger earner (dc-dty5): when ready is
                            // empty while shaping holds work, the feed
                            // itself is the waiter on the whole pool.
                            // Anything in ready and the line is absent
                            // entirely; ranking is fully derived.
                            let pool = queries::promotion_candidates(&all);
                            if ready.is_empty() && !pool.is_empty() {
                                outln!(
                                    "  the feed is dry: nothing in ready while {} item(s) shape — the feed itself waiters the shaping pool; top promotion candidates, ranked derived (q query shaping):",
                                    pool.len()
                                );
                                for n in pool.iter().take(3) {
                                    outln!("    {}", line(&all, n));
                                }
                            }
                        }
                        if let Some((sess, p)) = wake_reg {
                            let areas: Vec<&Node> = all
                                .iter()
                                .filter(|n| n.front.ty == "area" && p.areas.contains(&n.front.id))
                                .collect();
                            let ids: Vec<&str> = areas.iter().map(|a| a.front.id.as_str()).collect();
                            let mine_q = queue.iter().filter(|n| coord::in_purview(n, &ids)).count();
                            let mine_r = ready.iter().filter(|n| coord::in_purview(n, &ids)).count();
                            let names: Vec<&str> =
                                areas.iter().map(|a| quarry::surface::title_raw(a)).collect();
                            outln!(
                                "session {} purview ({}): {} answerable thread(s), {} ready item(s) — scope with --mine",
                                sess, names.join(", "), mine_q, mine_r
                            );
                            // Kind and charter beneath the purview line
                            // (it-sumw, it-skpa): the same texts q session
                            // resume renders — the kind field first, the
                            // charter prose on top of it (dc-ad8b).
                            if let Some(k) = coord::kind_line(p) {
                                outln!("  {}", k);
                            }
                            if let Some(c) = coord::charter_line(p) {
                                outln!("  {}", c);
                            }
                            // The dispatch-kind wake leads with what a
                            // dispatcher owes (it-wub5): ready in purview,
                            // in-flight with harvest commands, homework
                            // residue — one render, both wake surfaces.
                            if shape.dispatcher_lead {
                                for l in quarry::render::dispatch_wake(&store, &all, &ids) {
                                    outln!("  {}", l);
                                }
                            }
                            let leases = coord::load_leases(&store);
                            for l in leases.iter().filter(|l| l.session != sess) {
                                let what = all
                                    .iter()
                                    .find(|n| n.front.id == l.item)
                                    .map(|n| aref(&all, n))
                                    .unwrap_or_else(|| format!("\"{}\" ({})", l.item_title, l.item));
                                outln!(
                                    "live lease elsewhere: session {} holds {:?} — {}",
                                    l.session, l.globs, what
                                );
                            }
                            if let Ok(log) = store.read_log() {
                                for n in all.iter().filter(|n| coord::in_purview(n, &ids)) {
                                    if let Some(ev) = log.iter().find(|ev| {
                                        ev.get("op").and_then(|v| v.as_str()) == Some("create")
                                            && ev.get("node").and_then(|v| v.as_str()) == Some(n.front.id.as_str())
                                            && ev.get("session").and_then(|v| v.as_str()).map_or(false, |s| s != sess)
                                    }) {
                                        let from = ev.get("session").and_then(|v| v.as_str()).unwrap_or("?");
                                        if !matches!(n.front.status.as_str(), "done" | "dropped" | "resolved") {
                                            outln!(
                                                "new in your purview from session {}: {}",
                                                from,
                                                line(&all, n)
                                            );
                                        }
                                    }
                                }
                            }
                        } else if let Some(sess) = &wake {
                            outln!(
                                "session '{}' has no registered purview — q session set <name> --areas <area>...",
                                sess
                            );
                        } else if !reg.is_empty() {
                            let names: Vec<&str> = reg.keys().map(|s| s.as_str()).collect();
                            outln!(
                                "unbound chat in a multi-session repo (sessions: {}). Before substantive work, ask the user: adopt one of these (q session adopt <name>), or define a new session — and if new, is it meant to persist across chats and be re-entered, or is it ephemeral, for this chat only?",
                                names.join(" · ")
                            );
                        }
                        outln!("orient with: q query queue · q query ready · q query shaping · q guide");
                    }
                }
            }
        },
        Cmd::Find { text } => {
            let store = Store::resolve()?;
            let all = store.load_all()?;
            let q = text.to_lowercase();
            // Tiered output (it-hjed): word-boundary and id hits first,
            // substring-only hits trailing as the labeled loose tail —
            // always shown, never hidden, no flag. Every hit line rides
            // atom_line; the tier label is a suffix in the same register
            // as the matched-in-body one.
            let hits = queries::find_hits(&all, &q);
            if hits.strong.is_empty() && hits.body.is_empty() && hits.loose.is_empty() {
                outln!("no node matches \"{}\".", text);
            }
            for n in hits.strong {
                outln!("{}", line(&all, n));
            }
            for n in hits.body {
                outln!("{}  (matched in body)", line(&all, n));
            }
            for n in hits.loose {
                outln!("{}  (loose: substring only)", line(&all, n));
            }
        }
        Cmd::Session { which } => {
            let store = Store::resolve()?;
            match which {
                SessionCmd::Set { name, areas, kind, charter, launcher, ephemeral } => {
                    let all = store.load_all()?;
                    let ids = coord::resolve_area_ids(&store, &all, &areas)?;
                    // One validation point (dc-ad8b): the kind string is
                    // judged in coord::parse_kind and nowhere else.
                    let kind = kind.as_deref().map(coord::parse_kind).transpose()?;
                    let titles: Vec<String> = ids
                        .iter()
                        .filter_map(|id| all.iter().find(|n| &n.front.id == id))
                        .map(|n| quarry::surface::title_raw(n).to_string())
                        .collect();
                    let overlaps = coord::purview_overlaps(&store, &ids);
                    coord::save_session(&store, &name, ids, kind.clone(), charter, ephemeral)?;
                    outln!(
                        "✔ session {}{} covers: {}{}",
                        name,
                        kind.map(|k| format!(" [{}]", k)).unwrap_or_default(),
                        titles.join(" · "),
                        if ephemeral { "  (ephemeral — this chat only)" } else { "" }
                    );
                    for (other, shared) in overlaps {
                        if other == name {
                            continue;
                        }
                        let shared_titles: Vec<String> = shared
                            .iter()
                            .filter_map(|id| all.iter().find(|n| &n.front.id == id))
                            .map(|n| quarry::surface::title_raw(n).to_string())
                            .collect();
                        outln!(
                            "  ⚠ purview overlaps session {} on: {} — legal (shared areas exist), but confirm it is deliberate; co-writes there want --shared leases.",
                            other,
                            shared_titles.join(", ")
                        );
                    }
                    outln!("  if THIS chat is to be the session: q session adopt {}", name);
                    if launcher {
                        let path = store.root.join(format!("{}-session.cmd", name));
                        std::fs::write(
                            &path,
                            format!("@echo off\r\nset QUARRY_SESSION={}\r\nclaude %*\r\n", name),
                        )?;
                        outln!("  ✔ launcher written: {}", path.display());
                        outln!("  run it to start a chat that IS this session; /clear keeps the identity, switching roles means relaunching.");
                    } else {
                        outln!(
                            "  launch with: cmd /c \"set QUARRY_SESSION={} && claude\"  (or --launcher to write a script)",
                            name
                        );
                    }
                }
                SessionCmd::Resume => {
                    // Boundary guard (it-ymsj): a dispatched agent waking a
                    // session identity is the same capture class as wrap.
                    if let Some(msg) = coord::boundary_refusal(&store, "q session resume") {
                        anyhow::bail!("{}", msg);
                    }
                    let all = store.load_all()?;
                    let Some(sess) = coord::current_session() else {
                        anyhow::bail!("no QUARRY_SESSION set — launch via a session launcher, or ask the user which session this chat is and relaunch")
                    };
                    let reg = coord::load_sessions(&store);
                    let Some(p) = reg.get(&sess) else {
                        anyhow::bail!("session '{}' is not registered — q session set {} --areas <area>...", sess, sess)
                    };
                    outln!("resuming session {}", sess);
                    let area_titles: Vec<String> = p
                        .areas
                        .iter()
                        .filter_map(|id| all.iter().find(|n| &n.front.id == id))
                        .map(|n| quarry::surface::title_raw(n).to_string())
                        .collect();
                    outln!("  purview: {}", area_titles.join(" · "));
                    // Kind and charter beneath the purview line (it-sumw,
                    // it-skpa): the kind field first, the charter prose on
                    // top of it (dc-ad8b) — the same texts the orient prints.
                    if let Some(k) = coord::kind_line(p) {
                        outln!("  {}", k);
                    }
                    if let Some(c) = coord::charter_line(p) {
                        outln!("  {}", c);
                    }
                    if let Some(ts) = coord::last_seen(&store, &sess) {
                        use time::format_description::well_known::Rfc3339;
                        let age_s = time::OffsetDateTime::parse(&ts, &Rfc3339)
                            .ok()
                            .map(|t| (time::OffsetDateTime::now_utc() - t).whole_seconds());
                        match age_s {
                            Some(a) if a < 120 => outln!(
                                "  ⚠ an incarnation of {} was active {}s ago — if another chat holds this identity, close one before writing.",
                                sess, a
                            ),
                            Some(a) => outln!("  last active: {}", human_age(a)),
                            None => {}
                        }
                    }
                    let ids: Vec<&str> = p.areas.iter().map(|s| s.as_str()).collect();
                    // The wake SHAPE follows the session's kind (it-wub5):
                    // this surface renders the shape it is handed —
                    // coord::wake_shape is the one kind match point.
                    let shape = coord::wake_shape(p.kind.as_deref());
                    let leases = coord::load_leases(&store);
                    let mine: Vec<_> = leases.iter().filter(|l| l.session == sess).collect();
                    if !mine.is_empty() {
                        outln!("  holdings:");
                        for l in mine {
                            let held = all.iter().find(|n| n.front.id == l.item);
                            let done = held
                                .map_or(false, |n| matches!(n.front.status.as_str(), "done" | "dropped"));
                            let what = held
                                .map(|n| aref(&all, n))
                                .unwrap_or_else(|| format!("\"{}\" ({})", l.item_title, l.item));
                            outln!(
                                "    {} holds {:?}{}",
                                what,
                                l.globs,
                                if done { format!(" — item done; release: q release {}", l.item) } else { String::new() }
                            );
                        }
                    }
                    if shape.dispatcher_lead {
                        // The dispatch-kind wake leads with what a
                        // dispatcher owes (it-wub5): ready in purview,
                        // in-flight with harvest commands, homework residue
                        // — one render, both wake surfaces.
                        for l in quarry::render::dispatch_wake(&store, &all, &ids) {
                            outln!("  {}", l);
                        }
                    } else {
                        let inflight: Vec<_> = all
                            .iter()
                            .filter(|n| n.front.ty == "item" && n.front.status == "in-flight" && coord::in_purview(n, &ids))
                            .collect();
                        if !inflight.is_empty() {
                            outln!("  in-flight in purview:");
                            for n in inflight {
                                outln!("    {}", line(&all, n));
                            }
                        }
                    }
                    let log = store.read_log()?;
                    let my_events: Vec<&serde_json::Value> = log
                        .iter()
                        .filter(|ev| ev.get("session").and_then(|v| v.as_str()) == Some(sess.as_str()))
                        .collect();
                    let my_last_ts = my_events.last().and_then(|ev| ev.get("ts").and_then(|v| v.as_str())).unwrap_or("");
                    if !my_events.is_empty() {
                        outln!("  your session's recent acts:");
                        for ev in my_events.iter().rev().take(8).rev() {
                            let node = ev.get("node").and_then(|v| v.as_str()).unwrap_or("?");
                            let what = all
                                .iter()
                                .find(|n| n.front.id == node)
                                .map(|n| aref(&all, n))
                                .unwrap_or_else(|| format!("({})", node));
                            let op = ev.get("op").and_then(|v| v.as_str()).unwrap_or("?");
                            let ts = ev.get("ts").and_then(|v| v.as_str()).unwrap_or("").split('T').nth(1).unwrap_or("");
                            outln!("    {} {} {}", ts, op, what);
                        }
                    }
                    let mut arrivals: Vec<&Node> = Vec::new();
                    for ev in log.iter().filter(|ev| {
                        ev.get("ts").and_then(|v| v.as_str()).map_or(false, |t| t > my_last_ts)
                            && ev.get("session").and_then(|v| v.as_str()).map_or(true, |s| s != sess)
                    }) {
                        if let Some(id) = ev.get("node").and_then(|v| v.as_str()) {
                            if let Some(n) = all.iter().find(|n| n.front.id == id) {
                                if coord::in_purview(n, &ids)
                                    && !matches!(n.front.status.as_str(), "done" | "dropped" | "resolved")
                                    && !arrivals.iter().any(|a| a.front.id == n.front.id)
                                {
                                    arrivals.push(n);
                                }
                            }
                        }
                    }
                    if !arrivals.is_empty() {
                        outln!("  arrived in your purview since your last act:");
                        for n in arrivals {
                            outln!("    {}", line(&all, n));
                        }
                    }
                    // Omitted under the dispatch shape: threads are not a
                    // dispatch session's to settle (dc-wngq).
                    if shape.owed_threads {
                        let owed: Vec<&Node> = queries::queue(&all)
                            .into_iter()
                            .filter(|n| coord::in_purview(n, &ids))
                            .collect();
                        if !owed.is_empty() {
                            outln!("  owed to the user in your purview:");
                            for n in owed {
                                outln!("    {}", line(&all, n));
                            }
                        }
                        // The gate's pressure surface (dc-p6z4), beside owed
                        // threads and scoped like them: a gate-demoted item
                        // has a real waiter — a dispatcher tried to fire it.
                        // Pressure cuts at shaped; sketches stay quiet.
                        let awaiting = queries::awaiting_acceptance(&all)
                            .into_iter()
                            .filter(|n| n.front.status == "shaped" && coord::in_purview(n, &ids))
                            .count();
                        if awaiting > 0 {
                            outln!(
                                "  awaiting acceptance: {} shaped item(s) in your purview with no acceptance lines — the gate holds them from ready (q query awaiting-acceptance)",
                                awaiting
                            );
                        }
                        // The assay's pressure surface (dc-drr6, it-fwn3),
                        // beside owed threads and scoped like them: judging
                        // a claim is a design-session act, and load-bearing
                        // is the waiter test (dc-p6z4) — builds stand on
                        // the claim. Zero-holds unassayed stays off the
                        // wake by construction: the query is
                        // weight-filtered.
                        let unassayed = queries::load_bearing_unassayed(&all)
                            .into_iter()
                            .filter(|(n, _)| coord::in_purview(n, &ids))
                            .count();
                        if unassayed > 0 {
                            outln!(
                                "  load-bearing unassayed: {} claim(s) in your purview builds stand on with no judge on record — fool's gold risk rises with weight (q query load)",
                                unassayed
                            );
                        }
                        // The standing-ruling earner (dc-dty5, dc-ygzz),
                        // scoped like its siblings: defects are bugs on
                        // sight and fix with urgency — every kind=bug in
                        // shaping is waitered by fiat. Zero renders
                        // nothing.
                        let defects = queries::defects(&all)
                            .into_iter()
                            .filter(|n| queries::in_shaping(n) && coord::in_purview(n, &ids))
                            .count();
                        if defects > 0 {
                            outln!(
                                "  defects in shaping: {} bug(s) in your purview below ready — defects fix with urgency, the standing ruling waiters them (q query defects)",
                                defects
                            );
                        }
                        // The witness review channel (dc-mpg8), beside
                        // owed threads and scoped like them: witness-
                        // authored acceptance waits on the user's
                        // ratify-or-amend at the design liaison. Zero
                        // renders nothing.
                        let witness = queries::witness_flags(&all)
                            .into_iter()
                            .filter(|(n, _)| coord::in_purview(n, &ids))
                            .count();
                        if witness > 0 {
                            outln!(
                                "  witness-authored acceptance under review: {} line(s) in your purview authored from a non-design seat await the user's ratify-or-amend (q witness)",
                                witness
                            );
                        }
                        // The hunger earner (dc-dty5), scoped like its
                        // siblings: ready empty while shaping holds work
                        // — the feed itself is the waiter on the whole
                        // pool. Anything in ready and the line is absent
                        // entirely; ranking is fully derived.
                        let fed = queries::ready(&all)
                            .into_iter()
                            .any(|n| coord::in_purview(n, &ids));
                        let pool: Vec<&Node> = queries::promotion_candidates(&all)
                            .into_iter()
                            .filter(|n| coord::in_purview(n, &ids))
                            .collect();
                        if !fed && !pool.is_empty() {
                            outln!(
                                "  the feed is dry: nothing in ready in your purview while {} item(s) shape — the feed itself waiters the shaping pool; top promotion candidates, ranked derived (q query shaping --mine):",
                                pool.len()
                            );
                            for n in pool.iter().take(3) {
                                outln!("    {}", line(&all, n));
                            }
                        }
                    }
                    outln!("  next: q query ready --mine · q query shaping --mine · q wrap before stopping");
                }
                SessionCmd::Adopt { name } => {
                    let reg = coord::load_sessions(&store);
                    if !reg.contains_key(&name) {
                        anyhow::bail!("session '{}' is not registered — q session set {} --areas <area>...", name, name);
                    }
                    coord::write_adopt_request(&store, &name)?;
                    outln!("✔ adopt request written for session {}.", name);
                    outln!("  the next shell tool call binds this chat and injects QUARRY_SESSION automatically (120s window).");
                    outln!("  prefer launcher-owned identity for new chats: the {}-session launcher.", name);
                }
                SessionCmd::Retire { name } => {
                    // The retire guard, scoped to the retiree (it-e6wq):
                    // refuse only when the retiree is implicated — the
                    // chat's own session under a live badge, or a retiree
                    // with a dispatch of its own in flight. A third
                    // session with no live dispatch retires clean while
                    // unrelated badges fly.
                    if let Some(msg) = coord::retire_refusal(&store, &name) {
                        anyhow::bail!("{}", msg);
                    }
                    // The retiree's commit-set, read before the heartbeat
                    // clears (it-4q6t): the moment the session provably
                    // ends is the moment its leftovers are offered a
                    // labeled commit of their own.
                    let leftovers = queries::pending_graph(&store).and_then(|pg| {
                        pg.sets.into_iter().find(|s| s.owner.as_deref() == Some(name.as_str()))
                    });
                    coord::retire_session(&store, &name, &Store::actor())?;
                    outln!("✔ session {} retired — registry entry removed, leases released, last rites logged.", name);
                    if let Some(set) = leftovers {
                        outln!(
                            "  {} uncommitted node file(s) the log says {} still owns — commit them as their own labeled commit:",
                            set.files.len(),
                            name
                        );
                        for f in &set.files {
                            let carries = if f.carries.is_empty() {
                                String::new()
                            } else {
                                format!(" (carries {}'s earlier edits)", f.carries.join(", "))
                            };
                            outln!("    {}{}", f.path, carries);
                        }
                        outln!("    {}", set.add_command());
                        outln!(
                            "    git commit -m \"{}'s graph nodes, committed at retirement: {}\"",
                            name,
                            set.files.iter().map(|f| f.node.as_str()).collect::<Vec<_>>().join(", ")
                        );
                        outln!("    (the full map, any time: q query commit-set)");
                    }
                }
                SessionCmd::List => {
                    let all = store.load_all()?;
                    let reg = coord::load_sessions(&store);
                    if reg.is_empty() {
                        outln!("no sessions registered — q session set <name> --areas <area>...");
                    }
                    for (name, p) in reg {
                        let titles: Vec<String> = p
                            .areas
                            .iter()
                            .filter_map(|id| all.iter().find(|n| &n.front.id == id))
                            .map(|n| quarry::surface::title_raw(n).to_string())
                            .collect();
                        // The kind rides the name as data found in the
                        // registry (dc-ad8b) — kindless renders as today.
                        outln!(
                            "{}{}: {}{}",
                            name,
                            p.kind.as_ref().map(|k| format!(" [{}]", k)).unwrap_or_default(),
                            titles.join(" · "),
                            p.charter.map(|c| format!(" — {}", c)).unwrap_or_default()
                        );
                    }
                }
            }
        }
        Cmd::Reserve {
            item,
            files,
            shared,
            steal,
            reason,
        } => {
            let store = Store::resolve()?;
            let sess = coord::current_session().ok_or_else(|| {
                anyhow::anyhow!("no QUARRY_SESSION set — leases need a session identity (q session set <name> --areas ..., then export QUARRY_SESSION=<name>)")
            })?;
            let all = store.load_all()?;
            let node = store.find(&all, &item)?.clone();
            // The fire-time backstop (dc-p6z4), ahead of C8: the solo
            // station is design-capable, so the refusal teaches the
            // authoring command directly — and it must speak before the
            // brief teach, because the tripwired brief refuses too.
            ops::acceptance_backstop(&store, &node, true)?;
            // The witness executor check (dc-mpg8): the authoring badge
            // cannot solo-build the item it authored — author is never
            // executor. The solo station compares both the acting key and
            // the session (unlike join, where session env is inherited).
            let solo_key = coord::acting_key(
                coord::current_agent().as_deref(),
                coord::current_chat().as_deref(),
                Some(&sess),
            )
            .unwrap_or_else(|| format!("session:{}", sess));
            ops::witness_execution_check(&store, &node.front.id, &solo_key, Some(&sess))?;
            // C8: a lease follows a brief — no lease on unbriefed work.
            if !coord::briefed_this_session(&store, &node.front.id, &sess) {
                anyhow::bail!(
                    "C8: no brief on record for {} from session {} — a lease follows a brief. Render it (q brief {}), read it, then reserve.",
                    aref(&all, &node), sess, node.front.id
                );
            }
            // Engine-native consequence gate: a steal demands its reason —
            // the required response IS the proof of engagement, and it lands
            // in the logged event.
            if steal && reason.is_none() {
                let leases = coord::load_leases(&store);
                outln!("⏸ gated: --steal overrides another session's lease. Who you are overriding:");
                for l in leases.iter().filter(|l| l.session != sess) {
                    outln!(
                        "  session {} holds {:?} for \"{}\" (since {}, actor {})",
                        l.session, l.globs, l.item_title, l.since, l.actor
                    );
                }
                outln!("\nIf the override is justified, re-run the same command adding: --reason \"why\"");
                outln!("The reason is logged on the steal event — the holder will read it.");
                std::process::exit(2);
            }
            let out = coord::reserve(
                &store,
                &node,
                &sess,
                &Store::actor(),
                files.clone(),
                shared,
                steal,
                reason.as_deref(),
            )?;
            outln!(
                "✔ lease: {} holds {:?}{} (session {})",
                aref(&all, &node),
                files,
                if shared { " [shared]" } else { "" },
                sess
            );
            for s in out.stolen {
                outln!(
                    "  ⚠ STOLEN from session {} (\"{}\", held {:?} since {}) — logged; tell them.",
                    s.session, s.item_title, s.globs, s.since
                );
            }
            for c in out.co_holders {
                outln!(
                    "  co-writing with session {} (\"{}\", {:?}) — coordinate at file level.",
                    c.session, c.item_title, c.globs
                );
            }
        }
        Cmd::Release { item } => {
            let store = Store::resolve()?;
            let sess = coord::current_session()
                .ok_or_else(|| anyhow::anyhow!("no QUARRY_SESSION set"))?;
            let all = store.load_all()?;
            let node = store.find(&all, &item)?.clone();
            let held = coord::load_leases(&store)
                .iter()
                .find(|l| l.item == node.front.id)
                .map(|l| l.globs.clone());
            coord::release(&store, &node, &sess, &Store::actor())?;
            outln!("✔ released: {}", aref(&all, &node));
            vein_check(&store, &node, held);
            // The arc is over: land clears the badge and the observed set.
            coord::clear_dispatch(&store, &node.front.id);
            coord::clear_touched(&store, &format!("item:{}", node.front.id));
            // Release is an arc boundary like wrap and harvest: the rendered
            // page must not keep showing a landed arc as live.
            if let Ok(p) = quarry::view::write(&store) {
                outln!("  view regenerated: {}", p.display());
            }
        }
        Cmd::Dispatch { item, files, shared, steal, reason, solo } => {
            let store = Store::resolve()?;
            let sess = coord::current_session().ok_or_else(|| {
                anyhow::anyhow!(
                    "q dispatch is a session act — the lease it takes needs a holder, and an unbound chat has none. Bind this chat first: q session adopt <name> (or register one: q session set <name> --areas <area>...), then re-run."
                )
            })?;
            // Fire-time routing (it-hapc, dc-crea): a pull, never a send.
            // Derived before anything mutates; advisory and stateless —
            // the offer stops the fire, --solo overrides without ceremony.
            if !solo {
                let all = store.load_all()?;
                let node = store.find(&all, &item)?;
                match coord::fire_routing(&store, node, &sess) {
                    coord::FireRouting::Fire => {}
                    coord::FireRouting::Leave(live) => {
                        outln!(
                            "not dispatched — a live dispatcher covers {} (fire-time routing, dc-crea):",
                            aref(&all, node)
                        );
                        for c in &live {
                            outln!(
                                "  · session {} — active {}{}",
                                c.name,
                                c.age_secs.map(human_age).unwrap_or_else(|| "now".into()),
                                c.charter
                                    .as_deref()
                                    .map(|ch| format!(" — charter: {}", ch))
                                    .unwrap_or_default()
                            );
                        }
                        outln!("  leave it: ready IS the dispatcher feed — the kind-shaped wake and the purview surfaces deliver it, and the first to claim dispatches it (the claim point guards the race, dc-qyr5).");
                        if node.front.status != "ready" {
                            outln!(
                                "  note: the item is [{}] — the feed carries ready items; for it to flow: q set {} status=ready",
                                node.front.status, node.front.id
                            );
                        }
                        outln!(
                            "  or fire solo from here (always legitimate): q dispatch {} --solo",
                            node.front.id
                        );
                        return Ok(());
                    }
                    coord::FireRouting::Wake(cands) => {
                        outln!(
                            "not dispatched — no dispatcher is awake for {} (fire-time routing, dc-crea):",
                            aref(&all, node)
                        );
                        for c in &cands {
                            outln!(
                                "  · session {} — {}{}",
                                c.name,
                                c.age_secs
                                    .map(|a| format!("last active {}", human_age(a)))
                                    .unwrap_or_else(|| "never seen on this machine".into()),
                                c.charter
                                    .as_deref()
                                    .map(|ch| format!(" — charter: {}", ch))
                                    .unwrap_or_default()
                            );
                            outln!("      wake it: {}", coord::wake_command(&store, &c.name));
                        }
                        if cands.len() > 1 {
                            outln!("  several cover it — the user picks which to wake (any ambiguity defers to the user, dc-crea).");
                        }
                        outln!(
                            "  or fire solo from here (always legitimate): q dispatch {} --solo",
                            node.front.id
                        );
                        return Ok(());
                    }
                }
            }
            let out = ops::dispatch(
                &store,
                &item,
                files,
                shared,
                steal,
                reason.as_deref(),
                &sess,
                &Store::actor(),
            )?;
            outln!(
                "✔ dispatched: \"{}\" ({}) — lease {:?}{}, in-flight, single-use join token minted",
                out.item_title,
                out.item_id,
                out.globs,
                if out.reused_lease { " [re-dispatch: lease kept]" } else { "" }
            );
            if let Some((holder, from_sess)) = &out.stolen_from {
                outln!(
                    "  ⚠ STOLEN from {} (session {}) — the dispatch moved whole: lease re-homed, old token dead, the old agent's associations cleared. Your reason is logged; tell them.",
                    holder, from_sess
                );
            }
            // The behind confrontation stays the DISPATCHER'S, here at the
            // hand-off moment: the agent fetches its brief at join, so the
            // staleness check must not wait for the render it will read.
            if let Ok(all) = store.load_all() {
                let behinds: Vec<_> = queries::behind(&store, &all)
                    .into_iter()
                    .filter(|b| b.src.id == out.item_id)
                    .collect();
                for b in &behinds {
                    let target = b
                        .to_atom
                        .as_ref()
                        .map(quarry::surface::atom_ref)
                        .unwrap_or_else(|| format!("\"{}\" ({})", b.to_title, b.to));
                    outln!(
                        "  ⚠ [sev {}] this item cites {} at {}, now {} ({}) — review, then: q affirm {} --to {}",
                        b.severity, target, b.at, b.current, b.reason, out.item_id, b.to
                    );
                }
                if !behinds.is_empty() {
                    outln!("  the agent inherits what you do not confront — review before spawning.");
                }
            }
            outln!(
                "  when the report arrives, YOU judge and land: q harvest {}  (the agent's done is a stop signal)",
                out.item_id
            );
            outln!("\nSPAWN PROMPT (one line — the agent fetches its own brief at join):");
            outln!("{}", out.spawn);
        }
        Cmd::Join { token, store: pin } => {
            // The explicit pin (the spawn line's stamp) enters the one
            // resolver here — join is the only verb carrying it, because
            // join is where the badge and its locale bind (dc-g5x5).
            let store = quarry::store::resolve_store(pin.as_deref(), &[], None)?;
            let identity = coord::acting_key(
                coord::current_agent().as_deref(),
                coord::current_chat().as_deref(),
                coord::current_session().as_deref(),
            );
            let out = ops::join(&store, &token, identity)?;
            match (&out.bound, out.rejoined) {
                (Some(key), _) => outln!(
                    "✔ joined: \"{}\" ({}) — identity {} bound to the badge; your q acts and file writes now resolve to it. The brief below is derived fresh from the graph.\n",
                    out.item_title, out.item_id, key
                ),
                (None, true) => outln!(
                    "already joined: \"{}\" ({}) — re-rendering the brief (derived fresh; a re-join is a read, not a state change).\n",
                    out.item_title, out.item_id
                ),
                _ => {}
            }
            // The fork case says itself: ops::join opens the brief with the
            // where-you-stand banner (it-rmqy), which states the pinned-store
            // split this arm used to print — and states the file half too.
            // A second paragraph here would only say it twice.
            out!("{}", out.brief);
        }
        Cmd::Harvest { item } => {
            let store = Store::resolve()?;
            out!("{}", render::harvest(&store, &item)?);
            let all = store.load_all()?;
            let n = store.find(&all, &item)?;
            store.log_event(serde_json::json!({
                "ts": Store::now(), "node": n.front.id, "v": n.front.v,
                "op": "harvest", "actor": Store::actor()
            }))?;
            // Harvest clears the badge (held entry and acting associations):
            // further writes in the dispatching chat are its own. The observed
            // set stays until release — vein_check consumes it at landing.
            coord::clear_dispatch(&store, &n.front.id);
            // A dispatch arc closing is a boundary too — the derived view
            // rides along for free (it-n3fu), best-effort.
            if let Ok(p) = quarry::view::write(&store) {
                outln!("  view regenerated: {}", p.display());
            }
        }
        Cmd::Witness { item, mark, author_session, ratify, by } => {
            let store = Store::resolve()?;
            match (item, mark) {
                (Some(key), Some(line)) => {
                    let author = author_session.ok_or_else(|| anyhow::anyhow!(
                        "--mark transcribes the seat that authored the line — name it: --author-session <name>"
                    ))?;
                    let n = ops::witness_mark(&store, &key, &line, &author)?;
                    let all = store.load_all()?;
                    outln!(
                        "✔ witness mark: {} — the line rides the design wake's review channel until the user ratifies or amends it (dc-mpg8).",
                        aref(&all, &n)
                    );
                }
                (Some(key), None) if ratify => {
                    let (n, count) = ops::witness_ratify(&store, &key, by.as_deref())?;
                    let all = store.load_all()?;
                    outln!(
                        "✔ user-ratified: {} witness line(s) on {} — the marks stay as record; the review channel lets them go (dc-mpg8).",
                        count,
                        aref(&all, &n)
                    );
                }
                (Some(key), None) => {
                    let all = store.load_all()?;
                    let n = store.find(&all, &key)?;
                    if n.front.witness.is_empty() {
                        outln!(
                            "{} carries no witness marks — its acceptance was authored from a design seat (or none exists).",
                            aref(&all, n)
                        );
                    } else {
                        outln!("{}", aref(&all, n));
                        for m in &n.front.witness {
                            let state = match &m.ratified {
                                Some(r) => format!("user-ratified {}", r.date),
                                // A removal mark's line is gone by construction
                                // (it-ds6b): the act itself is what awaits the
                                // user's word.
                                None if m.removed => {
                                    "removal under review — awaiting the user's ratify-or-amend".to_string()
                                }
                                None if n.front.acceptance.iter().any(|a| a == &m.line) => {
                                    "under review — awaiting the user's ratify-or-amend".to_string()
                                }
                                None => "line no longer in acceptance — mark stands as record".to_string(),
                            };
                            let act = if m.removed { "removed" } else { "authored" };
                            outln!("  · \"{}\"\n    {} by {} ({}) {} — {}", m.line, act, m.by, m.kind.as_deref().unwrap_or("kindless"), m.date, state);
                        }
                    }
                }
                (None, Some(_)) => {
                    anyhow::bail!("--mark acts on one item — name it: q witness <item> --mark \"<line>\" --author-session <name>")
                }
                (None, None) if ratify => {
                    anyhow::bail!("--ratify acts on one item — name it: q witness <item> --ratify --by user")
                }
                (None, None) => {
                    let all = store.load_all()?;
                    let flags = queries::witness_flags(&all);
                    if flags.is_empty() {
                        outln!("the witness channel is clear — no witness-authored acceptance line awaits the user (dc-mpg8).");
                    } else {
                        outln!("witness-authored acceptance under review (dc-mpg8) — the pen from a non-design seat is transcription; each act awaits the user's ratify-or-amend:");
                        for (n, m) in &flags {
                            let act = if m.removed { "removed" } else { "authored" };
                            outln!("  · \"{}\"\n      {} — {} by {} {}", m.line, aref(&all, n), act, m.by, m.date);
                        }
                        outln!("the user's word clears a line: q witness <item> --ratify --by user (their word transcribed — the q rule --by user channel)");
                    }
                }
            }
        }
        Cmd::Queue { which } => {
            let store = Store::resolve()?;
            let all = store.load_all()?;
            let (mut q, pruned) = coord::topic_queue_pruned(&store, &all);
            for id in &pruned {
                outln!("  (pruned: {} — resolved or gone)", id);
            }
            match which {
                None => {
                    if q.is_empty() {
                        outln!("topic queue is empty — q queue push <thread>");
                    }
                    for (i, id) in q.iter().enumerate() {
                        if let Some(n) = all.iter().find(|n| &n.front.id == id) {
                            let blockers = queries::live_blockers(&all, n);
                            let state = if blockers.is_empty() {
                                String::new()
                            } else {
                                format!("  (blocked on {})", aref(&all, blockers[0]))
                            };
                            let mark = if i == 0 { " ← next up" } else { "" };
                            outln!("{}. {}{}{}", i + 1, line(&all, n), state, mark);
                        }
                    }
                }
                Some(QueueCmd::Push { thread }) => {
                    let n = store.find(&all, &thread)?;
                    if n.front.ty != "thread" {
                        anyhow::bail!("{} is not a thread", aref(&all, n));
                    }
                    if n.front.status == "resolved" {
                        anyhow::bail!("{} is already resolved", aref(&all, n));
                    }
                    if q.contains(&n.front.id) {
                        anyhow::bail!("{} is already in the topic queue", aref(&all, n));
                    }
                    q.push(n.front.id.clone());
                    coord::save_topic_queue(&store, &q)?;
                    outln!("✔ queued at {}: {}", q.len(), line(&all, n));
                }
                Some(QueueCmd::Pop) => {
                    if q.is_empty() {
                        outln!("topic queue is empty.");
                    } else {
                        let id = q.remove(0);
                        coord::save_topic_queue(&store, &q)?;
                        match store.find(&all, &id) {
                            Ok(n) => {
                                outln!("now: {}", line(&all, n));
                                if !n.body.trim().is_empty() {
                                    for l in n.body.lines() {
                                        outln!("  {}", l);
                                    }
                                }
                                outln!("  (ruling still goes through: q rule {} \"...\" --by user)", n.front.id);
                            }
                            Err(_) => outln!("now: {}", id),
                        }
                        if let Some(next) = q.first().and_then(|id| all.iter().find(|n| &n.front.id == id)) {
                            outln!("  next after this: {}", aref(&all, next));
                        }
                    }
                }
                Some(QueueCmd::Front { thread }) => {
                    let n = store.find(&all, &thread)?;
                    let Some(pos) = q.iter().position(|id| id == &n.front.id) else {
                        anyhow::bail!("{} is not in the topic queue — q queue push first", aref(&all, n));
                    };
                    let id = q.remove(pos);
                    q.insert(0, id);
                    coord::save_topic_queue(&store, &q)?;
                    outln!("✔ front: {}", line(&all, n));
                }
                Some(QueueCmd::Drop { thread }) => {
                    let n = store.find(&all, &thread)?;
                    let Some(pos) = q.iter().position(|id| id == &n.front.id) else {
                        anyhow::bail!("{} is not in the topic queue", aref(&all, n));
                    };
                    q.remove(pos);
                    coord::save_topic_queue(&store, &q)?;
                    outln!("✔ dropped from the topic queue: {} (the thread itself is untouched)", line(&all, n));
                }
            }
        }
        Cmd::Wrap => {
            let store = Store::resolve()?;
            // Boundary guard (it-ymsj): wrap under an active dispatch badge
            // is the dispatcher's boundary run by the dispatched — refuse
            // before any cursor moves.
            if let Some(msg) = coord::boundary_refusal(&store, "q wrap") {
                anyhow::bail!("{}", msg);
            }
            let all = store.load_all()?;
            outln!("wrap — boundary lint:");
            let queued: Vec<_> = all
                .iter()
                .filter(|n| n.front.ty == "thread" && n.front.status == "queued")
                .collect();
            let answerable: Vec<_> = queued
                .iter()
                .filter(|n| queries::live_blockers(&all, n).is_empty())
                .collect();
            outln!(
                "  owed to the user: {} answerable, {} blocked on intermediate work",
                answerable.len(),
                queued.len() - answerable.len()
            );
            for n in &answerable {
                outln!("    {}", line(&all, n));
            }
            let touched_ids: std::collections::HashSet<String>;
            {
                // Session-touched review (user-ruled 2026-08-09): every node
                // this session created or adjusted since its last wrap, for a
                // final look while the context that wrote them is still warm.
                let sess_key = coord::current_session();
                let log = store.read_log()?;
                let touched = queries::session_touched(&log, sess_key.as_deref());
                touched_ids = touched.iter().map(|(id, _)| id.clone()).collect();
                if !touched.is_empty() {
                    outln!(
                        "  session-touched since last wrap ({}) — final review: does each still say what you now know?",
                        touched.len()
                    );
                    for (id, op) in touched.iter().take(15) {
                        match all.iter().find(|n| &n.front.id == id) {
                            Some(n) => outln!("    [{}] {}", op, line(&all, n)),
                            None => outln!("    [{}] {}", op, id),
                        }
                    }
                    if touched.len() > 15 {
                        outln!("    …and {} more", touched.len() - 15);
                    }
                }
                // The spend-down sweep (dc-hzrm): the boundary itself is the
                // waiter — the deep-touch list ranks where this session's
                // dying context concentrates, and the prompt asks what the
                // graph lacks. Direct authorship is the vehicle; the same
                // wrap-event cursor as the list above makes a second wrap in
                // this boundary render silence: once per boundary by
                // construction. A prompt, never a gate.
                let deep: Vec<(&Node, u32)> = queries::deep_touches(&log, sess_key.as_deref())
                    .into_iter()
                    .filter_map(|(id, score)| {
                        all.iter().find(|n| n.front.id == id).map(|n| (n, score))
                    })
                    .collect();
                if !deep.is_empty() {
                    outln!("  {}", quarry::framings::spend_down_header(deep.len()));
                    for (n, score) in deep.iter().take(10) {
                        outln!("    [depth {}] {}", score, line(&all, n));
                    }
                    if deep.len() > 10 {
                        outln!("    …and {} more", deep.len() - 10);
                    }
                    outln!("  {}", quarry::framings::SPEND_DOWN);
                }
                store.log_event(serde_json::json!({
                    "ts": Store::now(),
                    "node": format!("session:{}", sess_key.as_deref().unwrap_or("unbound")),
                    "v": 0, "op": "wrap", "actor": Store::actor()
                }))?;
            }
            // Wrap tells sediment from rot (dc-6gn9): only rot and breakage
            // enumerate; the strata collapse to the ratified count.
            let (sed, behind): (Vec<_>, Vec<_>) =
                queries::behind(&store, &all).into_iter().partition(|e| e.sediment);
            if behind.is_empty() && sed.is_empty() {
                outln!("  behind: none — every ref current");
            }
            if !behind.is_empty() {
                outln!("  behind: {} stale ref(s), worst severity {}", behind.len(), behind[0].severity);
                for e in behind.iter().take(5) {
                    let target = e
                        .to_atom
                        .as_ref()
                        .map(quarry::surface::atom_ref)
                        .unwrap_or_else(|| format!("\"{}\" ({})", e.to_title, e.to));
                    outln!(
                        "    [sev {}] {} -[{}]→ {} ({})",
                        e.severity,
                        quarry::surface::atom_ref(&e.src),
                        e.rel,
                        target,
                        e.reason
                    );
                }
            }
            if !sed.is_empty() {
                outln!("  {}", quarry::framings::sediment_line(sed.len()));
            }
            let inflight: Vec<_> = all
                .iter()
                .filter(|n| n.front.ty == "item" && n.front.status == "in-flight")
                .collect();
            if !inflight.is_empty() {
                outln!("  in-flight items ({}) — land, park, or hand off before stopping:", inflight.len());
                for n in inflight {
                    outln!("    {}", line(&all, n));
                }
            }
            let unfiled: Vec<_> = all
                .iter()
                .filter(|n| {
                    n.front.ty != "area"
                        && !n.front.edges.iter().any(|e| {
                            e.rel == "about"
                                && all.iter().any(|t| t.front.id == e.to && t.front.ty == "area")
                        })
                })
                .collect();
            if !unfiled.is_empty() {
                outln!("  unfiled ({}) — the map cannot place these; q link <id> about <area>:", unfiled.len());
                for n in unfiled {
                    outln!("    {}", line(&all, n));
                }
            }
            let unver = queries::unverified(&all);
            if !unver.is_empty() {
                outln!("  unverified assistant claims ({}):", unver.len());
                for n in unver {
                    outln!("    {}", line(&all, n));
                }
            }
            {
                // Dangling id-shapes in live bodies (dc-wwnk): each is a
                // citation to fix or hyphenated prose to leave — lint, never
                // a gate; backtick prose shapes to quiet them.
                let dang = quarry::mention::danglers(&all);
                if !dang.is_empty() {
                    outln!(
                        "  id-shapes in bodies resolving to nothing ({}) — citations to fix, or hyphenated prose to leave:",
                        dang.len()
                    );
                    for (n, id) in dang.iter().take(10) {
                        outln!("    {} in {}", id, aref(&all, n));
                    }
                    if dang.len() > 10 {
                        outln!("    …and {} more", dang.len() - 10);
                    }
                }
            }
            let log = store.read_log()?;
            let last_user = log.iter().rev().find(|ev| {
                ev.get("node")
                    .and_then(|v| v.as_str())
                    .and_then(|id| all.iter().find(|n| n.front.id == id))
                    .map_or(false, |n| n.front.provenance == "user")
            });
            match last_user {
                Some(ev) => outln!(
                    "  last write to a user-provenance node: {} — if the user has ruled anything since, record it: q rule <thread> \"...\" --by user",
                    ev.get("ts").and_then(|v| v.as_str()).unwrap_or("?")
                ),
                None => outln!(
                    "  no user-provenance writes on record — if the user has ruled anything, record it: q rule <thread> \"...\" --by user"
                ),
            }
            {
                // Leaseless observation pickup: the observed set stands in
                // for lease globs — delivered once, then cleared.
                let key = format!("session:{}", coord::session_key());
                let touched = coord::touched_for(&store, &key);
                if !touched.is_empty() {
                    outln!(
                        "  leaseless code writes this session ({} file(s)) — observed, never denied; was this an item's arc?",
                        touched.len()
                    );
                    for f in touched.iter().take(10) {
                        outln!("    {}", f);
                    }
                    if touched.len() > 10 {
                        outln!("    …and {} more", touched.len() - 10);
                    }
                    let matched = queries::items_matching_files(&all, &touched);
                    if !matched.is_empty() {
                        for m in matched.iter().take(3) {
                            outln!("    resembles: {}", line(&all, m));
                        }
                    }
                    outln!("    {}", quarry::framings::SIGHT_BOUNDARY);
                    outln!("    a recurring arc wants declaring next time: q brief <item>, then q reserve <item> --files <globs>");
                    coord::clear_touched(&store, &key);
                }
                // Dispatches whose report was never harvested: the judgment
                // seat is empty and the lease still held.
                for n in queries::unharvested_dispatches(&all, &log) {
                    outln!(
                        "  unharvested dispatch: {} — the report is owed; judge and land: q harvest {}",
                        aref(&all, n), n.front.id
                    );
                }
            }
            {
                use time::format_description::well_known::Rfc3339;
                for (name, p) in coord::load_sessions(&store) {
                    if !p.ephemeral {
                        continue;
                    }
                    if let Some(ts) = coord::last_seen(&store, &name) {
                        if let Ok(t) = time::OffsetDateTime::parse(&ts, &Rfc3339) {
                            let days = (time::OffsetDateTime::now_utc() - t).whole_days();
                            if days >= 7 {
                                outln!(
                                    "  ephemeral session {} inactive {}d — a retirement flow is pending the archival design; for now it lingers in graph/sessions.json",
                                    name, days
                                );
                            }
                        }
                    }
                }
            }
            {
                let archivable: Vec<&Node> = all
                    .iter()
                    .filter(|n| {
                        !n.front.archived
                            && !matches!(n.front.ty.as_str(), "area" | "doc")
                            && matches!(
                                n.front.status.as_str(),
                                "done" | "dropped" | "resolved" | "refuted" | "superseded"
                            )
                            && !all.iter().any(|c| {
                                !c.front.archived
                                    && c.front.edges.iter().any(|e| e.rel == "part-of" && e.to == n.front.id)
                            })
                    })
                    .collect();
                // Cooling rule (user-ruled 2026-08-09): work settled THIS
                // session archives at a later wrap, not its landing wrap —
                // a just-landed node is still a landmark for follow-up work.
                let (cooling, cold): (Vec<&&Node>, Vec<&&Node>) = archivable
                    .iter()
                    .partition(|n| touched_ids.contains(&n.front.id));
                if !cold.is_empty() {
                    outln!(
                        "  archivable ({} settled leaf/childless node(s)) — q archive <node>; parents index their archived offspring:",
                        cold.len()
                    );
                    for n in cold.iter().take(6) {
                        outln!("    {}", line(&all, n));
                    }
                    if cold.len() > 6 {
                        outln!("    …and {} more", cold.len() - 6);
                    }
                }
                if !cooling.is_empty() {
                    outln!(
                        "  cooling ({} settled this session) — leave live for follow-up work; archive at a later wrap:",
                        cooling.len()
                    );
                    for n in cooling.iter().take(6) {
                        outln!("    {}", line(&all, n));
                    }
                }
                // Landmark backstop: items landed this session whose held
                // files nothing cites — vein or no vein, decided while warm.
                let leases = coord::load_leases(&store);
                for id in &touched_ids {
                    if let Some(n) = all.iter().find(|n| &n.front.id == id) {
                        if n.front.ty == "item" && n.front.status == "done" {
                            let globs = leases
                                .iter()
                                .find(|l| &l.item == id)
                                .map(|l| l.globs.clone())
                                .unwrap_or_else(|| n.front.write_set.clone());
                            if !globs.is_empty() && !queries::files_cited(&all, &globs) {
                                outln!(
                                    "  landed uncited: {} held {:?} and nothing cites those files — vein or no vein? (q claim --source file:...)",
                                    aref(&all, n), globs
                                );
                            }
                        }
                    }
                }
            }
            if let Some(sess) = coord::current_session() {
                if coord::load_sessions(&store).get(&sess).map_or(false, |p| p.ephemeral) {
                    outln!(
                        "  this session ({}) is EPHEMERAL — the default close act is retirement: q session retire {}",
                        sess, sess
                    );
                    outln!(
                        "    (if its purview proved durable this session, instead convert: q session set {} --areas ... without --ephemeral, and say so)",
                        sess
                    );
                }
            }
            let leases = coord::load_leases(&store);
            if !leases.is_empty() {
                let sess = coord::current_session().unwrap_or_default();
                outln!("  leases:");
                for l in &leases {
                    let owner = if l.session == sess { "yours" } else { "theirs" };
                    let held = all.iter().find(|n| n.front.id == l.item);
                    let done =
                        held.map_or(false, |n| matches!(n.front.status.as_str(), "done" | "dropped"));
                    let what = held
                        .map(|n| aref(&all, n))
                        .unwrap_or_else(|| format!("\"{}\" ({})", l.item_title, l.item));
                    outln!(
                        "    [{}] {} holds {:?}{} since {}{}",
                        owner,
                        what,
                        l.globs,
                        if l.shared { " [shared]" } else { "" },
                        l.since,
                        if done && l.session == sess {
                            format!(" — item is done; release it: q release {}", l.item)
                        } else {
                            String::new()
                        }
                    );
                }
            }
            {
                // The atom lint (dc-nnf5), wrap half: fires only inside
                // quarry's own repo (src/surface.rs present) — host-repo
                // wraps skip it naturally. Checked, never remembered.
                let src = store.root.join("src");
                if src.join("surface.rs").exists() {
                    let offenders = quarry::surface::lint_sources(&src);
                    if !offenders.is_empty() {
                        // Worded without the scanned token itself — the
                        // lint once caught this very message.
                        outln!(
                            "  ⚠ atom lint: raw title access outside src/surface.rs ({}) — every register rides the atom:",
                            offenders.len()
                        );
                        for (f, l) in offenders {
                            outln!("    src/{}:{}", f, l);
                        }
                    }
                }
                // The locale lint (dc-g5x5), wrap half — same fence, same
                // scope guard as the atom lint above.
                if src.join("store.rs").exists() {
                    let offenders = quarry::store::lint_locale_sources(&src);
                    if !offenders.is_empty() {
                        outln!(
                            "  ⚠ locale lint: store resolution outside src/store.rs ({}) — one resolver owns the graph locale:",
                            offenders.len()
                        );
                        for (f, l) in offenders {
                            outln!("    src/{}:{}", f, l);
                        }
                    }
                }
            }
            // The commit-set echo (it-4q6t): whenever uncommitted graph
            // changes exist, wrap renders the ownership split — the log is
            // the oracle, last writer since the prior commit owns the file.
            if let Some(pg) = queries::pending_graph(&store) {
                if !pg.is_empty() {
                    let sess = coord::current_session();
                    outln!("  uncommitted graph changes — the commit-set (the log's last writer owns each file):");
                    for set in &pg.sets {
                        let who = match (&set.owner, sess.as_deref()) {
                            (Some(o), Some(s)) if o == s => format!("yours ({})", o),
                            (Some(o), _) => {
                                format!("{} ({})", o, queries::age_phrase(set.age_secs))
                            }
                            (None, _) => "unstamped (no session on the log's record)".into(),
                        };
                        outln!("    {} — {}", who, set.add_command());
                        for f in set.files.iter().filter(|f| !f.carries.is_empty()) {
                            outln!(
                                "      ({} carries {}'s earlier edits — worth naming in the commit message)",
                                f.node,
                                f.carries.join(", ")
                            );
                        }
                    }
                    if !pg.ledger.is_empty() {
                        outln!(
                            "    the log shard rides along with any commit (exempt ledger, internally attributed, never split): git add {}",
                            pg.ledger.join(" ")
                        );
                    }
                    if !pg.shared.is_empty() {
                        outln!("    shared graph files, staged with the work they belong to: {}", pg.shared.join(" "));
                    }
                    outln!("    commit yours with the work it belongs to — explicit paths pass; a bulk sweep denies while others' work is pending. Full map: q query commit-set");
                }
            }
            // The view is derived; the boundary regenerates it (it-n3fu) so
            // freshness never rides a remembered convention. Best-effort:
            // the lint above must land even if the render cannot.
            match quarry::view::write(&store) {
                Ok(p) => outln!("  view regenerated: {}", p.display()),
                Err(e) => outln!("  ⚠ view regeneration failed: {}", e),
            }
        }
        Cmd::New {
            ty,
            title,
            kind,
            status,
            provenance,
            about,
            path,
            method,
            ratified,
            body,
            body_file,
            acceptance,
            fields,
            supports,
            note,
        } => {
            let store = Store::resolve()?;
            let a = NewCliArgs {
                ty,
                title,
                kind,
                status,
                provenance,
                about,
                path,
                method,
                ratified,
                body,
                body_file,
                acceptance,
                fields,
                supports,
                note,
            };
            let all = store.load_all()?;
            if let Some(g) = quarry::protocol::gate_if_needed(
                &store,
                &all,
                "new",
                Some(a.ty.as_str()),
                a.kind.as_deref(),
                serde_json::to_value(&a)?,
            )? {
                print_gate(&g);
                std::process::exit(2);
            }
            do_new(&store, a)?;
        }
        Cmd::Resume { token } => {
            let store = Store::resolve()?;
            let intent = quarry::protocol::take_intent(&store, &token)?;
            match intent.verb.as_str() {
                "new" => {
                    let a: NewCliArgs = serde_json::from_value(intent.args)?;
                    do_new(&store, a)?;
                }
                other => anyhow::bail!("token holds an unsupported verb '{}'", other),
            }
        }
        Cmd::Link {
            src,
            rel,
            dst,
            acknowledge,
            note,
        } => {
            let store = Store::resolve()?;
            let edge = ops::link(&store, &src, &rel, &dst, acknowledge, note)?;
            outln!("✔ {} -[{}]-> {} (at {})", src, edge.rel, edge.to, edge.at);
            let all = store.load_all()?;
            let src_id = store.find(&all, &src)?.front.id.clone();
            if edge.rel == "depends-on" {
                if let Ok(t) = store.find(&all, &edge.to) {
                    let blocking = !queries::live_blockers(&all, store.find(&all, &src_id)?).is_empty();
                    if blocking {
                        outln!(
                            "  note: {} is now blocked on {} — it leaves ready/queue views until that lands.",
                            aref(&all, store.find(&all, &src_id)?), aref(&all, t)
                        );
                    }
                }
            }
            // A lean declared late (dc-6gn9): a reading linked into
            // consumers that already settled is already done serving —
            // sweep now, not at some later settling act.
            let reading_touched = store
                .find(&all, &src_id)
                .map(|n| n.front.ty == "claim" && n.front.kind.as_deref() == Some("reading"))
                .unwrap_or(false)
                || store
                    .find(&all, &edge.to)
                    .map(|n| n.front.ty == "claim" && n.front.kind.as_deref() == Some("reading"))
                    .unwrap_or(false);
            drop(all);
            print_homework(&store, &[src_id.as_str(), edge.to.as_str()]);
            area_watermarks(&store, &src_id);
            if reading_touched {
                sweep_readings(&store);
            }
        }
        Cmd::Unlink { src, rel, dst, note } => {
            let store = Store::resolve()?;
            let (n, edge) = ops::unlink(&store, &src, &rel, &dst, note)?;
            outln!(
                "✔ retired: {} -[{}]-> {} (was at {})",
                n.front.id, edge.rel, edge.to, edge.at
            );
            outln!(
                "  no version bump on either node — retirement is bookkeeping, not content; the log carries who and why."
            );
        }
        Cmd::Set { node, fields, note } => {
            let store = Store::resolve()?;
            let o = ops::set(&store, &node, &fields, note)?;
            let n = o.node;
            outln!("✔ {}", line(&store.load_all().unwrap_or_default(), &n));
            // The removal echo (it-ds6b): the FULL resolved line, never what
            // was typed — the mint-echo pattern: a wrong-but-real match
            // reads wrong here, at the moment it is cheapest to catch.
            for l in &o.removed {
                outln!("  acceptance removed: \"{}\"", l);
            }
            if o.witness_removals > 0 {
                outln!(
                    "  removed from a witness seat — authoring-by-subtraction is authoring (dc-mpg8): the removal rides the design wake's review channel until the user ratifies (q witness {}).",
                    n.front.id
                );
            }
            if let Some(df) = &o.demoted_from {
                outln!(
                    "  UN-READIED: {} [{}] → [shaped], loudly (logged) — the strip took the last acceptance line, and ready is stored intent (dc-p6z4): the ready feed carries only items whose contract is stated. Author acceptance, then flip ready again.",
                    n.front.id, df
                );
            }
            presence_note(&store, &n.front.id);
            print_homework(&store, &[n.front.id.as_str()]);
            area_watermarks(&store, &n.front.id);
            if fields.iter().any(|f| f == "status=done") {
                vein_check(&store, &n, None);
                // The assay office (dc-drr6): the landing act ratifies the
                // arc's vein and feature mints — harvest by the dispatcher's
                // hand, solo self-ratified on the record. Silence when the
                // arc leaves nothing to assay.
                if n.front.ty == "item" {
                    if let Ok(Some(assay)) = ops::ratify_landing(
                        &store,
                        &n.front.id,
                        coord::current_session().as_deref(),
                    ) {
                        let all2 = store.load_all().unwrap_or_default();
                        let count = assay.ratified.len();
                        outln!(
                            "  {}",
                            if assay.solo {
                                quarry::framings::assay_solo_line(count)
                            } else {
                                quarry::framings::assay_harvest_line(count)
                            }
                        );
                        for c in &assay.ratified {
                            outln!("    · {}", line(&all2, c));
                        }
                    }
                }
            }
            // A settled-status flip is a landing: regenerate the page so it
            // never shows settled work as live (the stale-view class).
            if fields
                .iter()
                .any(|f| matches!(f.as_str(), "status=done" | "status=dropped" | "status=resolved" | "status=parked" | "status=superseded" | "status=refuted"))
            {
                if let Ok(p) = quarry::view::write(&store) {
                    outln!("  view regenerated: {}", p.display());
                }
            }
            // A settling act may have been a reading's last live consumer
            // (dc-6gn9); a kind flip to reading may find its consumers
            // already settled. Either way the sweep says what it hid.
            if fields
                .iter()
                .any(|f| f.starts_with("status=") || f.starts_with("kind="))
            {
                sweep_readings(&store);
            }
            // The lean prompt at ready (it-6349): the flip is the station
            // where shaping completes — enumerate the un-edged citations
            // while the design session still holds context.
            if fields.iter().any(|f| f == "status=ready") && n.front.ty == "item" {
                print_lean_prompt(&store, &n);
            }
            // Solo-path advert: taking up an item without a lease is legal —
            // leaseless writes accrue and nudge, never deny — but a declared
            // arc gets the full surfaces. Said at the moment it fires.
            if fields.iter().any(|f| f == "status=in-flight")
                && n.front.ty == "item"
                && !coord::load_leases(&store).iter().any(|l| l.item == n.front.id)
            {
                outln!(
                    "  note: in flight with no lease — about to write code solo? Declare the arc: q brief {} then q reserve {} --files <globs> (or hand it off whole: q dispatch {}).",
                    n.front.id, n.front.id, n.front.id
                );
            }
        }
        Cmd::Edit {
            node,
            body,
            body_file,
            note,
        } => {
            let store = Store::resolve()?;
            let body = read_body(body, body_file)?;
            if body.is_empty() {
                anyhow::bail!("provide --body or --body-file");
            }
            let n = ops::edit_body(&store, &node, body, note)?;
            if let Ok(all) = store.load_all() {
                outln!("✔ {}", line(&all, &n));
                print_mention_surfaces(&all, &n);
            }
            presence_note(&store, &n.front.id);
            print_homework(&store, &[n.front.id.as_str()]);
            area_watermarks(&store, &n.front.id);
        }
        Cmd::Rule {
            thread,
            text,
            by,
            title,
        } => {
            let store = Store::resolve()?;
            let d = ops::rule(&store, &thread, &text, by, title)?;
            outln!("✔ {}", line(&store.load_all().unwrap_or_default(), &d));
            outln!("  thread resolved.");
            print_mint_surfaces(&store, &d);
            let all = store.load_all()?;
            let th_id = store.find(&all, &thread)?.front.id.clone();
            drop(all);
            print_homework(&store, &[th_id.as_str(), d.front.id.as_str()]);
            area_watermarks(&store, &d.front.id);
            // A ruling settles its thread and lands in force (dc-6gn9):
            // readings either consumed may now archive.
            sweep_readings(&store);
        }
        Cmd::Claim {
            text,
            title,
            kind,
            about,
            source,
            method,
            provenance,
            status,
        } => {
            let store = Store::resolve()?;
            let n = ops::claim(&store, &text, title, kind, about, source, method, provenance, status)?;
            outln!("✔ {}", line(&store.load_all().unwrap_or_default(), &n));
            print_mint_surfaces(&store, &n);
            area_watermarks(&store, &n.front.id);
        }
        Cmd::Refute { claim, by, note } => {
            let store = Store::resolve()?;
            let (c, blast) = ops::refute(&store, &claim, &by, note)?;
            let all = store.load_all().unwrap_or_default();
            outln!("✔ {} is now refuted", aref(&all, &c));
            if blast.is_empty() {
                outln!("  nothing leaned on it.");
            } else {
                outln!("  blast radius — these leaned on it:");
                for n in blast {
                    outln!("    {}", line(&all, &n));
                }
            }
            print_homework(&store, &[c.front.id.as_str()]);
            // A refuted claim is settled (dc-6gn9): readings it consumed
            // may just have lost their last live consumer.
            sweep_readings(&store);
        }
        Cmd::Affirm { node, to } => {
            let store = Store::resolve()?;
            // Affirm is species-shaped by method (dc-6gn9): the ratified
            // teaching frames what this affirm records — re-read for
            // instrument-backed measurements, re-run for manual methods,
            // the sediment framing for readings. A prompt, never a gate.
            let teaching = store.load_all().ok().and_then(|all| {
                store
                    .find(&all, &node)
                    .ok()
                    .and_then(queries::affirm_teaching)
            });
            let count = ops::affirm(&store, &node, to)?;
            if count == 0 {
                outln!("nothing behind — no restamp needed.");
            } else {
                if let Some(t) = teaching {
                    outln!("{}", t);
                }
                outln!("✔ restamped {} ref(s)", count);
            }
        }
        Cmd::Archive { node, undo } => {
            let store = Store::resolve()?;
            let n = ops::archive(&store, &node, undo)?;
            let all = store.load_all().unwrap_or_default();
            if undo {
                outln!("✔ restored to default surfaces: {}", line(&all, &n));
            } else {
                outln!("✔ archived: {}", line(&all, &n));
                outln!("  still reachable — ids resolve, edges hold, blast/behind see it; surfaces show counts of what they hide.");
                // Archiving a consumer settles it (dc-6gn9): its readings
                // may just have lost their last live consumer.
                sweep_readings(&store);
            }
        }
        Cmd::Open { node, all } => {
            let store = Store::resolve()?;
            out!("{}", render::open(&store, &node, all)?);
            // An area open is the canonical read-first act: record it for
            // the ACTING identity's attention (dc-pwyd — a joined agent's
            // read spends its badge, never the holding session's watermark)
            // so the first-touch gate passes silently on the diligent path.
            if let Ok(loaded) = store.load_all() {
                if let Ok(n) = store.find(&loaded, &node) {
                    if n.front.ty == "area" {
                        coord::record_area_read(&store, &coord::attention_key(&store), &n.front.id);
                    }
                }
            }
        }
        Cmd::Brief { item } => {
            let store = Store::resolve()?;
            let text = render::brief(&store, &item)?;
            out!("{}", text);
            let all = store.load_all()?;
            let n = store.find(&all, &item)?;
            store.log_event(serde_json::json!({
                "ts": Store::now(), "node": n.front.id, "v": n.front.v,
                "op": "brief", "actor": Store::actor()
            }))?;
        }
        Cmd::Log { node } => {
            let store = Store::resolve()?;
            out!("{}", render::log(&store, &node)?);
        }
        Cmd::Query { which } => {
            let store = Store::resolve()?;
            let all = store.load_all()?;
            match which {
                Query::Ready { mine } => {
                    let leases = coord::load_leases(&store);
                    let sess = coord::current_session().unwrap_or_default();
                    let r = scope_mine(&store, &all, queries::ready(&all), mine);
                    let mut shown = 0;
                    for n in r {
                        let foreign: Vec<&coord::Lease> = leases
                            .iter()
                            .filter(|l| l.session != sess && !l.shared)
                            .filter(|l| {
                                n.front.write_set.iter().any(|w| {
                                    l.globs.iter().any(|g| coord::globs_overlap(w, g))
                                })
                            })
                            .collect();
                        if foreign.is_empty() {
                            outln!("{}", line(&all, n));
                            shown += 1;
                        } else {
                            outln!(
                                "{}  ⚠ write-set leased by session {} (\"{}\")",
                                line(&all, n),
                                foreign[0].session,
                                foreign[0].item_title
                            );
                        }
                    }
                    if shown == 0 {
                        outln!("nothing dispatchable.");
                    } else {
                        outln!("dispatch the chain in one act: q dispatch <item> --files <globs>  ·  solo: q brief <item>, then q reserve");
                    }
                }
                Query::Shaping { mine } => {
                    let items: Vec<&Node> =
                        queries::shaping(&all).into_iter().map(|(n, _)| n).collect();
                    for n in scope_mine(&store, &all, items, mine) {
                        outln!("{}", line(&all, n));
                        for b in queries::live_blockers(&all, n) {
                            outln!("    blocked on {}", aref(&all, b));
                        }
                    }
                    // Intent and reality share vocabulary — the delta derives.
                    // Advertised where shaping is judged; silence the default.
                    let d = queries::intent_delta(&all, None);
                    if !d.unlanded.is_empty() || !d.unintended.is_empty() {
                        outln!(
                            "intent delta — plan and reality join by name: {} intended-but-unlanded, {} landed-but-unintended (q query intent-delta)",
                            d.unlanded.len(),
                            d.unintended.len()
                        );
                    }
                }
                Query::Queue { mine } => {
                    let q = scope_mine(&store, &all, queries::queue(&all), mine);
                    if q.is_empty() {
                        outln!("queue is empty.");
                    }
                    let waiting = all
                        .iter()
                        .filter(|n| n.front.ty == "thread" && n.front.status == "queued")
                        .count()
                        .saturating_sub(queries::queue(&all).len());
                    for n in &q {
                        outln!("{}", line(&all, n));
                    }
                    if waiting > 0 {
                        outln!("({} queued thread(s) still blocked on intermediate work)", waiting);
                    }
                }
                Query::Behind => {
                    // The classifier (dc-6gn9): rot enumerates loud under a
                    // wants-action header; sediment — drift over readings —
                    // collapses to the ratified count with its reach-line.
                    let (sed, rot): (Vec<_>, Vec<_>) =
                        queries::behind(&store, &all).into_iter().partition(|e| e.sediment);
                    if rot.is_empty() && sed.is_empty() {
                        outln!("nothing behind — every ref current.");
                    }
                    if !rot.is_empty() {
                        outln!("wants action ({} — every entry means something):", rot.len());
                    }
                    for e in rot {
                        let target = e
                            .to_atom
                            .as_ref()
                            .map(quarry::surface::atom_ref)
                            .unwrap_or_else(|| format!("\"{}\" ({})", e.to_title, e.to));
                        outln!(
                            "  [sev {}] {} -[{}]→ {}  at {} now {}  ({})",
                            e.severity,
                            quarry::surface::atom_ref(&e.src),
                            e.rel,
                            target,
                            e.at,
                            e.current,
                            e.reason
                        );
                    }
                    if !sed.is_empty() {
                        outln!("{}", quarry::framings::sediment_line(sed.len()));
                        let mut strata: Vec<&str> = Vec::new();
                        for e in &sed {
                            if !strata.contains(&e.src.id.as_str()) {
                                strata.push(&e.src.id);
                            }
                        }
                        outln!(
                            "  the strata: {} — q open <id> reads one at its date",
                            strata.join(", ")
                        );
                    }
                }
                Query::Blast { node } => {
                    let n = store.find(&all, &node)?;
                    let ids = queries::blast(&all, &n.front.id);
                    if ids.is_empty() {
                        outln!("nothing leans on {}.", n.front.id);
                    }
                    for id in ids {
                        if let Some(m) = all.iter().find(|m| m.front.id == id) {
                            outln!("{}", line(&all, m));
                        }
                    }
                }
                Query::Homework { node } => {
                    // The per-node pull (it-8tcy): homework is DERIVED from
                    // current graph state, never stored — the act-time print
                    // after a mutating verb delivers this same surface, so a
                    // lost print is recovered here, not protected there.
                    let n = store.find(&all, &node)?;
                    let lines = render::homework(&all, &[n.front.id.as_str()]);
                    if lines.is_empty() {
                        outln!(
                            "no homework on {} — no citer behind it, nothing it unblocks.",
                            aref(&all, n)
                        );
                    } else {
                        outln!("homework on {} — derived from the graph now:", aref(&all, n));
                        for l in lines {
                            outln!("  {}", l);
                        }
                    }
                }
                Query::Contested => {
                    let c = queries::contested(&all);
                    if c.is_empty() {
                        outln!("no contested writes.");
                    }
                    for (src, rel, dst) in c {
                        outln!(
                            "{} -[{}]-> {} (user-provenance)",
                            aref(&all, src), rel, aref(&all, dst)
                        );
                    }
                }
                Query::Idle { days } => {
                    for n in queries::idle(&all, days) {
                        outln!("{}", line(&all, n));
                    }
                }
                Query::Unverified => {
                    let u = queries::unverified(&all);
                    if u.is_empty() {
                        outln!("no unverified assistant claims.");
                    }
                    for n in u {
                        outln!("{}", line(&all, n));
                    }
                }
                Query::Load => {
                    let u = queries::load_bearing_unassayed(&all);
                    if u.is_empty() {
                        outln!("no load-bearing unassayed claims — every claim builds stand on has a judge on record.");
                    } else {
                        outln!("{}", quarry::framings::ASSAY_WARNING);
                        for (n, _) in u {
                            outln!("  {}", line(&all, n));
                        }
                    }
                }
                Query::AwaitingAcceptance => {
                    // The gate's derived filter (dc-p6z4): never stored —
                    // authoring acceptance clears an item by construction.
                    let aw = queries::awaiting_acceptance(&all);
                    if aw.is_empty() {
                        outln!("nothing awaiting acceptance — every live item states what done means.");
                    } else {
                        for n in &aw {
                            outln!("{}", line(&all, n));
                        }
                        outln!("derived, never stored: authoring acceptance clears an item by construction — q set <id> acceptance+=\"<outcome>\" (the gate: ready refuses the flip and reserve un-readies without it, dc-p6z4)");
                    }
                }
                Query::Defects => {
                    // The standing ruling's pull surface (dc-ygzz, dc-dty5):
                    // every live defect, the shaping stratum first — those
                    // are the bugs no feed carries yet.
                    let d = queries::defects(&all);
                    if d.is_empty() {
                        outln!("no live defects — nothing kind=bug stands unfixed.");
                    } else {
                        for n in &d {
                            outln!("{}", line(&all, n));
                        }
                        outln!("defects are bugs on sight and fix with urgency (dc-ygzz) — the shaping stratum leads: those are the bugs no feed carries yet");
                    }
                }
                Query::CommitSet => {
                    // The commit-set pull handle (it-4q6t): the deny and the
                    // wrap echo both advertise this surface.
                    match queries::pending_graph(&store) {
                        None => outln!("git is unavailable or the store root is not a checkout — no commit-set to derive."),
                        Some(pg) if pg.is_empty() => {
                            outln!("nothing uncommitted under graph/ — the tree is clean.")
                        }
                        Some(pg) => {
                            let sess = coord::current_session();
                            outln!("the uncommitted graph, split by the log's ownership (last writer since the prior commit owns the file):");
                            for set in &pg.sets {
                                match (&set.owner, sess.as_deref()) {
                                    (Some(o), Some(s)) if o == s => {
                                        outln!("  yours ({}):", o)
                                    }
                                    (Some(o), _) => outln!(
                                        "  {} ({}){}:",
                                        o,
                                        queries::age_phrase(set.age_secs),
                                        if queries::owner_stale(set.age_secs) {
                                            " — stale"
                                        } else {
                                            ""
                                        }
                                    ),
                                    (None, _) => outln!(
                                        "  unstamped (no session on the log's record) — blocks no sweep; stage explicitly with the work it belongs to:"
                                    ),
                                }
                                for f in &set.files {
                                    let node = all
                                        .iter()
                                        .find(|n| n.front.id == f.node)
                                        .map(|n| aref(&all, n))
                                        .unwrap_or_else(|| f.node.clone());
                                    let carries = if f.carries.is_empty() {
                                        String::new()
                                    } else {
                                        format!(
                                            " (carries {}'s earlier edits)",
                                            f.carries.join(", ")
                                        )
                                    };
                                    outln!("    {} — {}{}", f.path, node, carries);
                                }
                                let foreign_stale = set.owner.is_some()
                                    && set.owner.as_deref() != sess.as_deref()
                                    && queries::owner_stale(set.age_secs);
                                if foreign_stale {
                                    outln!("    adoption offer (explicit paths pass the guard by construction):");
                                    for l in set.adoption_offer() {
                                        outln!("      {}", l);
                                    }
                                } else {
                                    outln!("    stage: {}", set.add_command());
                                }
                            }
                            if !pg.ledger.is_empty() {
                                outln!(
                                    "  exempt ledger — rides along with any commit, internally attributed, never split: git add {}",
                                    pg.ledger.join(" ")
                                );
                            }
                            if !pg.shared.is_empty() {
                                outln!(
                                    "  shared graph files, staged with the work they belong to: {}",
                                    pg.shared.join(" ")
                                );
                            }
                            outln!("explicit-path staging always passes; a bulk sweep (git add graph / -A / . / git commit -a) denies at the hook while another session's node files are pending.");
                        }
                    }
                }
                Query::Dispatch { item } => {
                    out!("{}", render::dispatch_trace(&store, &item)?);
                }
                Query::IntentDelta { area } => {
                    let scope = match &area {
                        Some(key) => {
                            let n = store.find(&all, key)?;
                            if n.front.ty != "area" {
                                anyhow::bail!(
                                    "{} is not an area — the delta joins on shared areas",
                                    aref(&all, n)
                                );
                            }
                            Some(n.front.id.clone())
                        }
                        None => None,
                    };
                    let d = queries::intent_delta(&all, scope.as_deref());
                    if d.unlanded.is_empty() && d.unintended.is_empty() {
                        outln!("no intent delta — every capability named in live acceptance has a vein in a shared area, and every registered vein was named by some intent (or nothing is named yet).");
                    }
                    if !d.unlanded.is_empty() {
                        outln!("intended but unlanded — named in live acceptance, no vein claim in a shared area carries it:");
                        for (name, item) in &d.unlanded {
                            outln!("  · `{}` — {}", name, aref(&all, item));
                        }
                    }
                    if !d.unintended.is_empty() {
                        outln!("landed but unintended — a vein no intent named (emergent scope, visible instead of silent):");
                        for (name, claim) in &d.unintended {
                            outln!("  · `{}` — {}", name, aref(&all, claim));
                        }
                    }
                    if !d.unlanded.is_empty() || !d.unintended.is_empty() {
                        outln!("(an index for judgment, never a sweep — land it, name it in an item's acceptance, or leave it and know why)");
                    }
                }
            }
        }
    }
    Ok(())
}
