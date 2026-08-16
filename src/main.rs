use anyhow::Result;
use clap::{Parser, Subcommand};

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
    /// write-set+=… ratified=…
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
    #[command(after_help = "EXAMPLES:
  q claim \"halo is 4-11 cells\" --about hydrology --source s11-results --method \"ring differencing\"
  q claim \"`body-graph`: water bodies keep identity across chunk regen\" --about hydrology --source file:src/water/body.rs
Extract a claim only when something depends on the statement or kills it —
never while writing prose. C1: at least one --about. C2: non-user claims
name their --source doc. A landing counts as dependence: register a landed
capability as a VEIN claim (--source file:<the code>), titled name-first
in the project's register (`name`: what it provides) — systematic,
intention-revealing names. Titles feed the relatedness lexicon, so a
well-named vein surfaces itself to future work.")]
    Claim {
        text: String,
        /// Full title when the derived first-line cut would truncate it (register-length vein names)
        #[arg(long)]
        title: Option<String>,
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
Exclusive by default: an overlapping foreign lease denies, naming the
holder. --shared marks a co-write zone (shared leases coexist, with mutual
visibility). --steal overrides loudly and is logged. Release explicitly
when the arc lands; sessions start leaseless and reserve at dispatch.
A lease follows a brief: reserve refuses unless this session rendered
`q brief <item>` first (C8) — no lease on unbriefed work.")]
    Reserve {
        item: String,
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
        /// Write-set globs for the lease (falls back to the item's write-set)
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
q join <token> — and everything else derives here. The token is single-use
(re-join by the same identity re-prints the brief; a second identity
refuses). Identity is hook-injected (QUARRY_AGENT/QUARRY_CHAT) — outside
hook coverage, export QUARRY_DISPATCH=<item> instead and skip join.")]
    Join { token: String },
    /// Harvest a dispatch: observed-vs-leased, badge-stamped acts, report
    /// homework — the dispatcher judges acceptance and lands by hand
    #[command(after_help = "An agent's \"done\" is a stop signal, never a transition: the item stays
in-flight and the lease held until YOU land it (q set <item> status=done ·
q release <item>). Harvest prints the judgment surface and clears the
machine-local badge; a partial or stop report harvests the same way.")]
    Harvest { item: String },
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
    /// Assistant writes against user provenance (should be empty — C3)
    Contested,
    /// Nodes nothing points at
    Idle {
        #[arg(long, default_value_t = 14)]
        days: i64,
    },
    /// Assistant claims never verified
    Unverified,
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
        println!(
            "  note: bodyless sketch — a title-only node leaves the next reader nothing to open; one sentence of intent is the floor: q edit {} --body \"...\"",
            node.front.id
        );
    }
    let Ok(all) = store.load_all() else { return };
    let touches = quarry::queries::relatedness(&all, node);
    if !touches.is_empty() {
        println!("  touches — review and judge; link only what genuinely relates:");
        for (t, why) in touches {
            println!("    {} — {}", line(&all, t), why);
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
        println!("  body cites — read the echo; a wrong-but-real id reads wrong here:");
        for t in &resolved {
            println!("    {}", aref(all, t));
        }
        println!(
            "    (render unpacks these; a mention references, an edge leans — if this stands on one, record it: q link {} <rel> <id>)",
            node.front.id
        );
    }
    for d in &dangling {
        println!(
            "  {} is id-shaped but resolves to nothing — a citation to fix, or hyphenated prose to leave as is? (wrap lints danglers)",
            d
        );
    }
}

/// The per-area watermark surface, run after a mutating verb touched a node.
/// First touch of an unread area nudges once; foreign drift since the
/// recorded read prints inline (the delta IS the delivery); own writes and
/// quiet checks advance the cursor silently.
fn area_watermarks(store: &Store, node_id: &str) {
    let sess = coord::session_key();
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
        match coord::touch_area(store, &all, &sess, &aid) {
            coord::AreaTouch::FirstTouch => {
                println!(
                    "  note: first touch of area {} this session without a read — the read-first: q open {}",
                    area_ref, aid
                );
                coord::record_area_read(store, &sess, &aid);
            }
            coord::AreaTouch::Drift(lines) => {
                println!("  since your last read of {} (other sessions):", area_ref);
                for l in lines {
                    println!("    · {}", l);
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
    println!(
        "  landed uncited: no claim or doc cites {:?}. If this work left a durable capability, register its vein while the diff is warm:",
        globs
    );
    println!("    q claim \"`capability-name`: what it now provides\" --about <area> --source file:<path>");
    println!("  Name it — a `named` capability reasons better than a description, and every build that cites it surfaces by blast.");
    println!("  Skip freely if nothing durable landed — a claim minted to silence this line is Goodhart, worse than silence. Presence is checked; quality is judged at review.");
}

/// The area-first-touch gate (user-agreed 2026-08-09): minting into an area
/// this session has never read intercepts once, delivers the area's derived
/// read-first, and saves the intent for q resume. A prior same-session
/// `q open <area>` passes silently — the gate is the backstop, not the path.
fn area_gate_if_needed(store: &Store, a: &NewCliArgs) -> Result<bool> {
    let sess = coord::session_key();
    let all = store.load_all()?;
    let mut unread: Vec<(String, String)> = Vec::new();
    for key in &a.about {
        if key.starts_with("file:") {
            continue;
        }
        let Ok(n) = store.find(&all, key) else { continue };
        if n.front.ty == "area" && !coord::has_area_read(store, &sess, &n.front.id) {
            unread.push((n.front.id.clone(), aref(&all, n)));
        }
    }
    if unread.is_empty() {
        return Ok(false);
    }
    let token = quarry::protocol::save_intent(store, "new", serde_json::to_value(a)?)?;
    println!("⏸ gated: first write into unread area(s) this session — the read-first arrives now.");
    for (aid, area_ref) in &unread {
        println!("\n── area {} ──", area_ref);
        print!("{}", render::open(store, aid, false)?);
        coord::record_area_read(store, &coord::session_key(), aid);
    }
    println!("\nYour intent is saved. Read the above, then run: q resume {}", token);
    println!("(args are remembered; delivery is recorded — this session will not be gated on these areas again)");
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
    println!("✔ {}", line(&store.load_all().unwrap_or_default(), &node));
    if let Some(target) = supports {
        let edge = ops::link(store, &node.front.id, "supports", &target, false, None)?;
        println!("  ✔ {} -[supports]-> {} (at {})", node.front.id, edge.to, edge.at);
    }
    let filed = node.front.ty == "area"
        || node.front.edges.iter().any(|e| e.rel == "about" && e.to.starts_with("ar-"));
    if !filed {
        println!(
            "  note: unfiled — the map cannot place it. Attach it: q link {} about <area>",
            node.front.id
        );
    }
    print_mint_surfaces(store, &node);
    area_watermarks(store, &node.front.id);
    if let Ok(all) = store.load_all() {
        for (title, text) in quarry::protocol::inline_texts(
            &all,
            "new",
            Some(node.front.ty.as_str()),
            node.front.kind.as_deref(),
        ) {
            println!("\nprotocol — {}:", title);
            for l in text.lines() {
                println!("  {}", l);
            }
        }
    }
    Ok(())
}

fn print_gate(g: &quarry::protocol::Gate) {
    println!("⏸ gated: this act carries project protocol, delivered once per session.");
    for (title, body) in &g.rules {
        println!("\n── {} ──", title);
        for l in body.lines() {
            println!("{}", l);
        }
    }
    println!("\nYour intent is saved. Do the work under the protocol, then run: q resume {}", g.token);
    println!("(args are remembered; to change them, re-run the original command — this session is now cleared for this rule)");
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
fn print_homework(store: &Store, touched: &[&str]) {
    let Ok(all) = store.load_all() else { return };
    let mut lines: Vec<String> = Vec::new();
    for &id in touched {
        let Ok(target) = store.find(&all, id) else { continue };
        for (citer, e) in queries::citers_behind(&all, id) {
            lines.push(format!(
                "⚠ behind: {} -[{}]→ {} (cited {}, now v{}). Review the change, then: q affirm {} --to {}",
                aref(&all, citer), e.rel, aref(&all, target), e.at, target.front.v,
                citer.front.id, target.front.id
            ));
        }
        for n in queries::unblocked_by(&all, id) {
            let what = match (n.front.ty.as_str(), n.front.status.as_str()) {
                ("thread", "queued") => "answerable in the queue",
                ("thread", _) => "unblocked",
                ("item", "ready") => "dispatchable",
                _ => "unblocked (still shaping)",
            };
            lines.push(format!("✔ {}: {}", what, line(&all, n)));
        }
    }
    if !lines.is_empty() {
        println!("homework:");
        for l in lines {
            println!("  {}", l);
        }
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
        println!("  note: session {} touched this node ({} at {})", sess, op, ts);
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
            eprintln!("(--mine ignored: set QUARRY_SESSION and register it with q session set)");
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
            println!("✔ graph/ initialized at {}", cwd.display());
            println!("  suggested .gitignore lines: graph/.index/  graph/view/  graph/.*  (machine-local state — leases, cursors, the touched-set, the dispatch badge)");
            if claude {
                for a in quarry::teach::install_claude(&cwd)? {
                    println!("  ✔ {}", a);
                }
                println!("  note: the hook names this q binary by absolute path — re-run `q init --claude` if the binary moves.");
            }
        }
        Cmd::View { open } => {
            let store = Store::discover()?;
            let path = quarry::view::write(&store)?;
            println!("✔ rendered {}", path.display());
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
                    println!("  a serve loop is up — opening {} instead of the baked file", url);
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
            let store = Store::discover()?;
            let port = port.unwrap_or(quarry::view::DEFAULT_PORT);
            let listener = std::net::TcpListener::bind(("127.0.0.1", port))?;
            println!(
                "✔ serving the view at http://127.0.0.1:{} — every request renders the live graph; Ctrl-C stops",
                port
            );
            quarry::view::serve(&store, listener)?;
        }
        Cmd::Guide => print!("{}", quarry::teach::GUIDE),
        Cmd::Hook { which } => match which {
            HookCmd::Guard => {
                use std::io::Read as _;
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input)?;
                if let Some(msg) = quarry::teach::guard(&input) {
                    eprintln!("{}", msg);
                    std::process::exit(2);
                }
                // The lease layer (dispatch chain): best-effort, never fails
                // a session over a missing graph.
                if let (Some(path), Ok(store)) =
                    (quarry::teach::write_target(&input), Store::discover())
                {
                    let root = store.root.to_string_lossy().replace('\\', "/").to_lowercase();
                    let p = path.replace('\\', "/").to_lowercase();
                    // The lease layer judges REPO-RELATIVE paths only: a
                    // write outside the host repo (scratchpads, temp files)
                    // is never scope creep, never accrual, never contract
                    // material. Component-boundary strip: root + '/' + rel.
                    let rel = p
                        .strip_prefix(&root)
                        .and_then(|r| r.strip_prefix('/'))
                        .map(String::from);
                    if let Some(rel) = rel {
                        let parsed = serde_json::from_str::<serde_json::Value>(&input).ok();
                        let chat = parsed.as_ref().and_then(|v| {
                            v.get("session_id").and_then(|x| x.as_str()).map(String::from)
                        });
                        // The agent id rides the hook input in subagents —
                        // the subagent's only distinguishing identity (its
                        // session_id matches the parent chat's).
                        let agent = parsed.as_ref().and_then(quarry::teach::hook_agent_id);
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
                                eprintln!("{}", msg);
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
                            println!(
                                "{}",
                                serde_json::json!({"hookSpecificOutput": {
                                    "hookEventName": "PreToolUse",
                                    "additionalContext": context.join("\n")
                                }})
                            );
                        }
                    }
                }
            }
            HookCmd::Session => {
                use std::io::Read as _;
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input)?;
                if let Ok(store) = Store::discover() {
                    if let Some(out) = quarry::teach::session_hook_output(&store, &input) {
                        println!("{}", serde_json::to_string(&out)?);
                    }
                }
            }
            HookCmd::Orient => {
                // Best-effort: a hook must never fail a session over a missing graph.
                let (chat_id, model) = {
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
                    )
                };
                if let Ok(store) = Store::discover() {
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
                        println!(
                            "quarry: {} nodes · owed to the user: {} · ready to dispatch: {} · behind: {}",
                            all.len(),
                            queue.len(),
                            ready.len(),
                            behind.len()
                        );
                        // Omitted under the dispatch shape: threads are not
                        // a dispatch session's to settle (dc-wngq).
                        if shape.owed_threads {
                            for n in &queue {
                                println!("  owed: {}", line(&all, n));
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
                            println!(
                                "session {} purview ({}): {} answerable thread(s), {} ready item(s) — scope with --mine",
                                sess, names.join(", "), mine_q, mine_r
                            );
                            // Kind and charter beneath the purview line
                            // (it-sumw, it-skpa): the same texts q session
                            // resume renders — the kind field first, the
                            // charter prose on top of it (dc-ad8b).
                            if let Some(k) = coord::kind_line(p) {
                                println!("  {}", k);
                            }
                            if let Some(c) = coord::charter_line(p) {
                                println!("  {}", c);
                            }
                            // The dispatch-kind wake leads with what a
                            // dispatcher owes (it-wub5): ready in purview,
                            // in-flight with harvest commands, homework
                            // residue — one render, both wake surfaces.
                            if shape.dispatcher_lead {
                                for l in quarry::render::dispatch_wake(&store, &all, &ids) {
                                    println!("  {}", l);
                                }
                            }
                            let leases = coord::load_leases(&store);
                            for l in leases.iter().filter(|l| l.session != sess) {
                                let what = all
                                    .iter()
                                    .find(|n| n.front.id == l.item)
                                    .map(|n| aref(&all, n))
                                    .unwrap_or_else(|| format!("\"{}\" ({})", l.item_title, l.item));
                                println!(
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
                                            println!(
                                                "new in your purview from session {}: {}",
                                                from,
                                                line(&all, n)
                                            );
                                        }
                                    }
                                }
                            }
                        } else if let Some(sess) = &wake {
                            println!(
                                "session '{}' has no registered purview — q session set <name> --areas <area>...",
                                sess
                            );
                        } else if !reg.is_empty() {
                            let names: Vec<&str> = reg.keys().map(|s| s.as_str()).collect();
                            println!(
                                "unbound chat in a multi-session repo (sessions: {}). Before substantive work, ask the user: adopt one of these (q session adopt <name>), or define a new session — and if new, is it meant to persist across chats and be re-entered, or is it ephemeral, for this chat only?",
                                names.join(" · ")
                            );
                        }
                        println!("orient with: q query queue · q query ready · q query shaping · q guide");
                    }
                }
            }
        },
        Cmd::Find { text } => {
            let store = Store::discover()?;
            let all = store.load_all()?;
            let q = text.to_lowercase();
            // Tiered output (it-hjed): word-boundary and id hits first,
            // substring-only hits trailing as the labeled loose tail —
            // always shown, never hidden, no flag. Every hit line rides
            // atom_line; the tier label is a suffix in the same register
            // as the matched-in-body one.
            let hits = queries::find_hits(&all, &q);
            if hits.strong.is_empty() && hits.body.is_empty() && hits.loose.is_empty() {
                println!("no node matches \"{}\".", text);
            }
            for n in hits.strong {
                println!("{}", line(&all, n));
            }
            for n in hits.body {
                println!("{}  (matched in body)", line(&all, n));
            }
            for n in hits.loose {
                println!("{}  (loose: substring only)", line(&all, n));
            }
        }
        Cmd::Session { which } => {
            let store = Store::discover()?;
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
                    println!(
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
                        println!(
                            "  ⚠ purview overlaps session {} on: {} — legal (shared areas exist), but confirm it is deliberate; co-writes there want --shared leases.",
                            other,
                            shared_titles.join(", ")
                        );
                    }
                    println!("  if THIS chat is to be the session: q session adopt {}", name);
                    if launcher {
                        let path = store.root.join(format!("{}-session.cmd", name));
                        std::fs::write(
                            &path,
                            format!("@echo off\r\nset QUARRY_SESSION={}\r\nclaude %*\r\n", name),
                        )?;
                        println!("  ✔ launcher written: {}", path.display());
                        println!("  run it to start a chat that IS this session; /clear keeps the identity, switching roles means relaunching.");
                    } else {
                        println!(
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
                    println!("resuming session {}", sess);
                    let area_titles: Vec<String> = p
                        .areas
                        .iter()
                        .filter_map(|id| all.iter().find(|n| &n.front.id == id))
                        .map(|n| quarry::surface::title_raw(n).to_string())
                        .collect();
                    println!("  purview: {}", area_titles.join(" · "));
                    // Kind and charter beneath the purview line (it-sumw,
                    // it-skpa): the kind field first, the charter prose on
                    // top of it (dc-ad8b) — the same texts the orient prints.
                    if let Some(k) = coord::kind_line(p) {
                        println!("  {}", k);
                    }
                    if let Some(c) = coord::charter_line(p) {
                        println!("  {}", c);
                    }
                    if let Some(ts) = coord::last_seen(&store, &sess) {
                        use time::format_description::well_known::Rfc3339;
                        let age_s = time::OffsetDateTime::parse(&ts, &Rfc3339)
                            .ok()
                            .map(|t| (time::OffsetDateTime::now_utc() - t).whole_seconds());
                        match age_s {
                            Some(a) if a < 120 => println!(
                                "  ⚠ an incarnation of {} was active {}s ago — if another chat holds this identity, close one before writing.",
                                sess, a
                            ),
                            Some(a) => println!("  last active: {}", human_age(a)),
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
                        println!("  holdings:");
                        for l in mine {
                            let held = all.iter().find(|n| n.front.id == l.item);
                            let done = held
                                .map_or(false, |n| matches!(n.front.status.as_str(), "done" | "dropped"));
                            let what = held
                                .map(|n| aref(&all, n))
                                .unwrap_or_else(|| format!("\"{}\" ({})", l.item_title, l.item));
                            println!(
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
                            println!("  {}", l);
                        }
                    } else {
                        let inflight: Vec<_> = all
                            .iter()
                            .filter(|n| n.front.ty == "item" && n.front.status == "in-flight" && coord::in_purview(n, &ids))
                            .collect();
                        if !inflight.is_empty() {
                            println!("  in-flight in purview:");
                            for n in inflight {
                                println!("    {}", line(&all, n));
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
                        println!("  your session's recent acts:");
                        for ev in my_events.iter().rev().take(8).rev() {
                            let node = ev.get("node").and_then(|v| v.as_str()).unwrap_or("?");
                            let what = all
                                .iter()
                                .find(|n| n.front.id == node)
                                .map(|n| aref(&all, n))
                                .unwrap_or_else(|| format!("({})", node));
                            let op = ev.get("op").and_then(|v| v.as_str()).unwrap_or("?");
                            let ts = ev.get("ts").and_then(|v| v.as_str()).unwrap_or("").split('T').nth(1).unwrap_or("");
                            println!("    {} {} {}", ts, op, what);
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
                        println!("  arrived in your purview since your last act:");
                        for n in arrivals {
                            println!("    {}", line(&all, n));
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
                            println!("  owed to the user in your purview:");
                            for n in owed {
                                println!("    {}", line(&all, n));
                            }
                        }
                    }
                    println!("  next: q query ready --mine · q query shaping --mine · q wrap before stopping");
                }
                SessionCmd::Adopt { name } => {
                    let reg = coord::load_sessions(&store);
                    if !reg.contains_key(&name) {
                        anyhow::bail!("session '{}' is not registered — q session set {} --areas <area>...", name, name);
                    }
                    coord::write_adopt_request(&store, &name)?;
                    println!("✔ adopt request written for session {}.", name);
                    println!("  the next shell tool call binds this chat and injects QUARRY_SESSION automatically (120s window).");
                    println!("  prefer launcher-owned identity for new chats: the {}-session launcher.", name);
                }
                SessionCmd::Retire { name } => {
                    // Boundary guard (it-ymsj): last rites are the
                    // dispatcher's act, never a badged agent's.
                    if let Some(msg) = coord::boundary_refusal(&store, "q session retire") {
                        anyhow::bail!("{}", msg);
                    }
                    coord::retire_session(&store, &name, &Store::actor())?;
                    println!("✔ session {} retired — registry entry removed, leases released, last rites logged.", name);
                }
                SessionCmd::List => {
                    let all = store.load_all()?;
                    let reg = coord::load_sessions(&store);
                    if reg.is_empty() {
                        println!("no sessions registered — q session set <name> --areas <area>...");
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
                        println!(
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
            let store = Store::discover()?;
            let sess = coord::current_session().ok_or_else(|| {
                anyhow::anyhow!("no QUARRY_SESSION set — leases need a session identity (q session set <name> --areas ..., then export QUARRY_SESSION=<name>)")
            })?;
            let all = store.load_all()?;
            let node = store.find(&all, &item)?.clone();
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
                println!("⏸ gated: --steal overrides another session's lease. Who you are overriding:");
                for l in leases.iter().filter(|l| l.session != sess) {
                    println!(
                        "  session {} holds {:?} for \"{}\" (since {}, actor {})",
                        l.session, l.globs, l.item_title, l.since, l.actor
                    );
                }
                println!("\nIf the override is justified, re-run the same command adding: --reason \"why\"");
                println!("The reason is logged on the steal event — the holder will read it.");
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
            println!(
                "✔ lease: {} holds {:?}{} (session {})",
                aref(&all, &node),
                files,
                if shared { " [shared]" } else { "" },
                sess
            );
            for s in out.stolen {
                println!(
                    "  ⚠ STOLEN from session {} (\"{}\", held {:?} since {}) — logged; tell them.",
                    s.session, s.item_title, s.globs, s.since
                );
            }
            for c in out.co_holders {
                println!(
                    "  co-writing with session {} (\"{}\", {:?}) — coordinate at file level.",
                    c.session, c.item_title, c.globs
                );
            }
        }
        Cmd::Release { item } => {
            let store = Store::discover()?;
            let sess = coord::current_session()
                .ok_or_else(|| anyhow::anyhow!("no QUARRY_SESSION set"))?;
            let all = store.load_all()?;
            let node = store.find(&all, &item)?.clone();
            let held = coord::load_leases(&store)
                .iter()
                .find(|l| l.item == node.front.id)
                .map(|l| l.globs.clone());
            coord::release(&store, &node, &sess, &Store::actor())?;
            println!("✔ released: {}", aref(&all, &node));
            vein_check(&store, &node, held);
            // The arc is over: land clears the badge and the observed set.
            coord::clear_dispatch(&store, &node.front.id);
            coord::clear_touched(&store, &format!("item:{}", node.front.id));
            // Release is an arc boundary like wrap and harvest: the rendered
            // page must not keep showing a landed arc as live.
            if let Ok(p) = quarry::view::write(&store) {
                println!("  view regenerated: {}", p.display());
            }
        }
        Cmd::Dispatch { item, files, shared, steal, reason, solo } => {
            let store = Store::discover()?;
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
                        println!(
                            "not dispatched — a live dispatcher covers {} (fire-time routing, dc-crea):",
                            aref(&all, node)
                        );
                        for c in &live {
                            println!(
                                "  · session {} — active {}{}",
                                c.name,
                                c.age_secs.map(human_age).unwrap_or_else(|| "now".into()),
                                c.charter
                                    .as_deref()
                                    .map(|ch| format!(" — charter: {}", ch))
                                    .unwrap_or_default()
                            );
                        }
                        println!("  leave it: ready IS the dispatcher feed — the kind-shaped wake and the purview surfaces deliver it, and the first to claim dispatches it (the claim point guards the race, dc-qyr5).");
                        if node.front.status != "ready" {
                            println!(
                                "  note: the item is [{}] — the feed carries ready items; for it to flow: q set {} status=ready",
                                node.front.status, node.front.id
                            );
                        }
                        println!(
                            "  or fire solo from here (always legitimate): q dispatch {} --solo",
                            node.front.id
                        );
                        return Ok(());
                    }
                    coord::FireRouting::Wake(cands) => {
                        println!(
                            "not dispatched — no dispatcher is awake for {} (fire-time routing, dc-crea):",
                            aref(&all, node)
                        );
                        for c in &cands {
                            println!(
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
                            println!("      wake it: {}", coord::wake_command(&store, &c.name));
                        }
                        if cands.len() > 1 {
                            println!("  several cover it — the user picks which to wake (any ambiguity defers to the user, dc-crea).");
                        }
                        println!(
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
            println!(
                "✔ dispatched: \"{}\" ({}) — lease {:?}{}, in-flight, single-use join token minted",
                out.item_title,
                out.item_id,
                out.globs,
                if out.reused_lease { " [re-dispatch: lease kept]" } else { "" }
            );
            if let Some((holder, from_sess)) = &out.stolen_from {
                println!(
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
                    println!(
                        "  ⚠ [sev {}] this item cites {} at {}, now {} ({}) — review, then: q affirm {} --to {}",
                        b.severity, target, b.at, b.current, b.reason, out.item_id, b.to
                    );
                }
                if !behinds.is_empty() {
                    println!("  the agent inherits what you do not confront — review before spawning.");
                }
            }
            println!(
                "  when the report arrives, YOU judge and land: q harvest {}  (the agent's done is a stop signal)",
                out.item_id
            );
            println!("\nSPAWN PROMPT (one line — the agent fetches its own brief at join):");
            println!("{}", out.spawn);
        }
        Cmd::Join { token } => {
            let store = Store::discover()?;
            let identity = coord::acting_key(
                coord::current_agent().as_deref(),
                coord::current_chat().as_deref(),
                coord::current_session().as_deref(),
            );
            let out = ops::join(&store, &token, identity)?;
            match (&out.bound, out.rejoined) {
                (Some(key), _) => println!(
                    "✔ joined: \"{}\" ({}) — identity {} bound to the badge; your q acts and file writes now resolve to it. The brief below is derived fresh from the graph.\n",
                    out.item_title, out.item_id, key
                ),
                (None, true) => println!(
                    "already joined: \"{}\" ({}) — re-rendering the brief (derived fresh; a re-join is a read, not a state change).\n",
                    out.item_title, out.item_id
                ),
                _ => {}
            }
            print!("{}", out.brief);
        }
        Cmd::Harvest { item } => {
            let store = Store::discover()?;
            print!("{}", render::harvest(&store, &item)?);
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
                println!("  view regenerated: {}", p.display());
            }
        }
        Cmd::Queue { which } => {
            let store = Store::discover()?;
            let all = store.load_all()?;
            let (mut q, pruned) = coord::topic_queue_pruned(&store, &all);
            for id in &pruned {
                println!("  (pruned: {} — resolved or gone)", id);
            }
            match which {
                None => {
                    if q.is_empty() {
                        println!("topic queue is empty — q queue push <thread>");
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
                            println!("{}. {}{}{}", i + 1, line(&all, n), state, mark);
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
                    println!("✔ queued at {}: {}", q.len(), line(&all, n));
                }
                Some(QueueCmd::Pop) => {
                    if q.is_empty() {
                        println!("topic queue is empty.");
                    } else {
                        let id = q.remove(0);
                        coord::save_topic_queue(&store, &q)?;
                        match store.find(&all, &id) {
                            Ok(n) => {
                                println!("now: {}", line(&all, n));
                                if !n.body.trim().is_empty() {
                                    for l in n.body.lines() {
                                        println!("  {}", l);
                                    }
                                }
                                println!("  (ruling still goes through: q rule {} \"...\" --by user)", n.front.id);
                            }
                            Err(_) => println!("now: {}", id),
                        }
                        if let Some(next) = q.first().and_then(|id| all.iter().find(|n| &n.front.id == id)) {
                            println!("  next after this: {}", aref(&all, next));
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
                    println!("✔ front: {}", line(&all, n));
                }
                Some(QueueCmd::Drop { thread }) => {
                    let n = store.find(&all, &thread)?;
                    let Some(pos) = q.iter().position(|id| id == &n.front.id) else {
                        anyhow::bail!("{} is not in the topic queue", aref(&all, n));
                    };
                    q.remove(pos);
                    coord::save_topic_queue(&store, &q)?;
                    println!("✔ dropped from the topic queue: {} (the thread itself is untouched)", line(&all, n));
                }
            }
        }
        Cmd::Wrap => {
            let store = Store::discover()?;
            // Boundary guard (it-ymsj): wrap under an active dispatch badge
            // is the dispatcher's boundary run by the dispatched — refuse
            // before any cursor moves.
            if let Some(msg) = coord::boundary_refusal(&store, "q wrap") {
                anyhow::bail!("{}", msg);
            }
            let all = store.load_all()?;
            println!("wrap — boundary lint:");
            let queued: Vec<_> = all
                .iter()
                .filter(|n| n.front.ty == "thread" && n.front.status == "queued")
                .collect();
            let answerable: Vec<_> = queued
                .iter()
                .filter(|n| queries::live_blockers(&all, n).is_empty())
                .collect();
            println!(
                "  owed to the user: {} answerable, {} blocked on intermediate work",
                answerable.len(),
                queued.len() - answerable.len()
            );
            for n in &answerable {
                println!("    {}", line(&all, n));
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
                    println!(
                        "  session-touched since last wrap ({}) — final review: does each still say what you now know?",
                        touched.len()
                    );
                    for (id, op) in touched.iter().take(15) {
                        match all.iter().find(|n| &n.front.id == id) {
                            Some(n) => println!("    [{}] {}", op, line(&all, n)),
                            None => println!("    [{}] {}", op, id),
                        }
                    }
                    if touched.len() > 15 {
                        println!("    …and {} more", touched.len() - 15);
                    }
                }
                store.log_event(serde_json::json!({
                    "ts": Store::now(),
                    "node": format!("session:{}", sess_key.as_deref().unwrap_or("unbound")),
                    "v": 0, "op": "wrap", "actor": Store::actor()
                }))?;
            }
            let behind = queries::behind(&store, &all);
            if behind.is_empty() {
                println!("  behind: none — every ref current");
            } else {
                println!("  behind: {} stale ref(s), worst severity {}", behind.len(), behind[0].severity);
                for e in behind.iter().take(5) {
                    let target = e
                        .to_atom
                        .as_ref()
                        .map(quarry::surface::atom_ref)
                        .unwrap_or_else(|| format!("\"{}\" ({})", e.to_title, e.to));
                    println!(
                        "    [sev {}] {} -[{}]→ {} ({})",
                        e.severity,
                        quarry::surface::atom_ref(&e.src),
                        e.rel,
                        target,
                        e.reason
                    );
                }
            }
            let inflight: Vec<_> = all
                .iter()
                .filter(|n| n.front.ty == "item" && n.front.status == "in-flight")
                .collect();
            if !inflight.is_empty() {
                println!("  in-flight items ({}) — land, park, or hand off before stopping:", inflight.len());
                for n in inflight {
                    println!("    {}", line(&all, n));
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
                println!("  unfiled ({}) — the map cannot place these; q link <id> about <area>:", unfiled.len());
                for n in unfiled {
                    println!("    {}", line(&all, n));
                }
            }
            let unver = queries::unverified(&all);
            if !unver.is_empty() {
                println!("  unverified assistant claims ({}):", unver.len());
                for n in unver {
                    println!("    {}", line(&all, n));
                }
            }
            {
                // Dangling id-shapes in live bodies (dc-wwnk): each is a
                // citation to fix or hyphenated prose to leave — lint, never
                // a gate; backtick prose shapes to quiet them.
                let dang = quarry::mention::danglers(&all);
                if !dang.is_empty() {
                    println!(
                        "  id-shapes in bodies resolving to nothing ({}) — citations to fix, or hyphenated prose to leave:",
                        dang.len()
                    );
                    for (n, id) in dang.iter().take(10) {
                        println!("    {} in {}", id, aref(&all, n));
                    }
                    if dang.len() > 10 {
                        println!("    …and {} more", dang.len() - 10);
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
                Some(ev) => println!(
                    "  last write to a user-provenance node: {} — if the user has ruled anything since, record it: q rule <thread> \"...\" --by user",
                    ev.get("ts").and_then(|v| v.as_str()).unwrap_or("?")
                ),
                None => println!(
                    "  no user-provenance writes on record — if the user has ruled anything, record it: q rule <thread> \"...\" --by user"
                ),
            }
            {
                // Leaseless observation pickup: the observed set stands in
                // for lease globs — delivered once, then cleared.
                let key = format!("session:{}", coord::session_key());
                let touched = coord::touched_for(&store, &key);
                if !touched.is_empty() {
                    println!(
                        "  leaseless code writes this session ({} file(s)) — observed, never denied; was this an item's arc?",
                        touched.len()
                    );
                    for f in touched.iter().take(10) {
                        println!("    {}", f);
                    }
                    if touched.len() > 10 {
                        println!("    …and {} more", touched.len() - 10);
                    }
                    let matched = queries::items_matching_files(&all, &touched);
                    if !matched.is_empty() {
                        for m in matched.iter().take(3) {
                            println!("    resembles: {}", line(&all, m));
                        }
                    }
                    println!("    a recurring arc wants declaring next time: q brief <item>, then q reserve <item> --files <globs>");
                    coord::clear_touched(&store, &key);
                }
                // Dispatches whose report was never harvested: the judgment
                // seat is empty and the lease still held.
                for n in queries::unharvested_dispatches(&all, &log) {
                    println!(
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
                                println!(
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
                    println!(
                        "  archivable ({} settled leaf/childless node(s)) — q archive <node>; parents index their archived offspring:",
                        cold.len()
                    );
                    for n in cold.iter().take(6) {
                        println!("    {}", line(&all, n));
                    }
                    if cold.len() > 6 {
                        println!("    …and {} more", cold.len() - 6);
                    }
                }
                if !cooling.is_empty() {
                    println!(
                        "  cooling ({} settled this session) — leave live for follow-up work; archive at a later wrap:",
                        cooling.len()
                    );
                    for n in cooling.iter().take(6) {
                        println!("    {}", line(&all, n));
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
                                println!(
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
                    println!(
                        "  this session ({}) is EPHEMERAL — the default close act is retirement: q session retire {}",
                        sess, sess
                    );
                    println!(
                        "    (if its purview proved durable this session, instead convert: q session set {} --areas ... without --ephemeral, and say so)",
                        sess
                    );
                }
            }
            let leases = coord::load_leases(&store);
            if !leases.is_empty() {
                let sess = coord::current_session().unwrap_or_default();
                println!("  leases:");
                for l in &leases {
                    let owner = if l.session == sess { "yours" } else { "theirs" };
                    let held = all.iter().find(|n| n.front.id == l.item);
                    let done =
                        held.map_or(false, |n| matches!(n.front.status.as_str(), "done" | "dropped"));
                    let what = held
                        .map(|n| aref(&all, n))
                        .unwrap_or_else(|| format!("\"{}\" ({})", l.item_title, l.item));
                    println!(
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
                        println!(
                            "  ⚠ atom lint: raw title access outside src/surface.rs ({}) — every register rides the atom:",
                            offenders.len()
                        );
                        for (f, l) in offenders {
                            println!("    src/{}:{}", f, l);
                        }
                    }
                }
            }
            if let Ok(out) = std::process::Command::new("git")
                .current_dir(&store.root)
                .args(["status", "--porcelain", "--", "graph"])
                .output()
            {
                let dirty = String::from_utf8_lossy(&out.stdout).lines().count();
                if dirty > 0 {
                    println!(
                        "  graph/ has {} uncommitted change(s) — commit them with the work they belong to",
                        dirty
                    );
                }
            }
            // The view is derived; the boundary regenerates it (it-n3fu) so
            // freshness never rides a remembered convention. Best-effort:
            // the lint above must land even if the render cannot.
            match quarry::view::write(&store) {
                Ok(p) => println!("  view regenerated: {}", p.display()),
                Err(e) => println!("  ⚠ view regeneration failed: {}", e),
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
            let store = Store::discover()?;
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
            let store = Store::discover()?;
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
            let store = Store::discover()?;
            let edge = ops::link(&store, &src, &rel, &dst, acknowledge, note)?;
            println!("✔ {} -[{}]-> {} (at {})", src, edge.rel, edge.to, edge.at);
            let all = store.load_all()?;
            let src_id = store.find(&all, &src)?.front.id.clone();
            if edge.rel == "depends-on" {
                if let Ok(t) = store.find(&all, &edge.to) {
                    let blocking = !queries::live_blockers(&all, store.find(&all, &src_id)?).is_empty();
                    if blocking {
                        println!(
                            "  note: {} is now blocked on {} — it leaves ready/queue views until that lands.",
                            aref(&all, store.find(&all, &src_id)?), aref(&all, t)
                        );
                    }
                }
            }
            drop(all);
            print_homework(&store, &[src_id.as_str(), edge.to.as_str()]);
            area_watermarks(&store, &src_id);
        }
        Cmd::Unlink { src, rel, dst, note } => {
            let store = Store::discover()?;
            let (n, edge) = ops::unlink(&store, &src, &rel, &dst, note)?;
            println!(
                "✔ retired: {} -[{}]-> {} (was at {})",
                n.front.id, edge.rel, edge.to, edge.at
            );
            println!(
                "  no version bump on either node — retirement is bookkeeping, not content; the log carries who and why."
            );
        }
        Cmd::Set { node, fields, note } => {
            let store = Store::discover()?;
            let n = ops::set(&store, &node, &fields, note)?;
            println!("✔ {}", line(&store.load_all().unwrap_or_default(), &n));
            presence_note(&store, &n.front.id);
            print_homework(&store, &[n.front.id.as_str()]);
            area_watermarks(&store, &n.front.id);
            if fields.iter().any(|f| f == "status=done") {
                vein_check(&store, &n, None);
            }
            // A settled-status flip is a landing: regenerate the page so it
            // never shows settled work as live (the stale-view class).
            if fields
                .iter()
                .any(|f| matches!(f.as_str(), "status=done" | "status=dropped" | "status=resolved" | "status=parked" | "status=superseded" | "status=refuted"))
            {
                if let Ok(p) = quarry::view::write(&store) {
                    println!("  view regenerated: {}", p.display());
                }
            }
            // Solo-path advert: taking up an item without a lease is legal —
            // leaseless writes accrue and nudge, never deny — but a declared
            // arc gets the full surfaces. Said at the moment it fires.
            if fields.iter().any(|f| f == "status=in-flight")
                && n.front.ty == "item"
                && !coord::load_leases(&store).iter().any(|l| l.item == n.front.id)
            {
                println!(
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
            let store = Store::discover()?;
            let body = read_body(body, body_file)?;
            if body.is_empty() {
                anyhow::bail!("provide --body or --body-file");
            }
            let n = ops::edit_body(&store, &node, body, note)?;
            if let Ok(all) = store.load_all() {
                println!("✔ {}", line(&all, &n));
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
            let store = Store::discover()?;
            let d = ops::rule(&store, &thread, &text, by, title)?;
            println!("✔ {}", line(&store.load_all().unwrap_or_default(), &d));
            println!("  thread resolved.");
            print_mint_surfaces(&store, &d);
            let all = store.load_all()?;
            let th_id = store.find(&all, &thread)?.front.id.clone();
            drop(all);
            print_homework(&store, &[th_id.as_str(), d.front.id.as_str()]);
            area_watermarks(&store, &d.front.id);
        }
        Cmd::Claim {
            text,
            title,
            about,
            source,
            method,
            provenance,
            status,
        } => {
            let store = Store::discover()?;
            let n = ops::claim(&store, &text, title, about, source, method, provenance, status)?;
            println!("✔ {}", line(&store.load_all().unwrap_or_default(), &n));
            print_mint_surfaces(&store, &n);
            area_watermarks(&store, &n.front.id);
        }
        Cmd::Refute { claim, by, note } => {
            let store = Store::discover()?;
            let (c, blast) = ops::refute(&store, &claim, &by, note)?;
            let all = store.load_all().unwrap_or_default();
            println!("✔ {} is now refuted", aref(&all, &c));
            if blast.is_empty() {
                println!("  nothing leaned on it.");
            } else {
                println!("  blast radius — these leaned on it:");
                for n in blast {
                    println!("    {}", line(&all, &n));
                }
            }
            print_homework(&store, &[c.front.id.as_str()]);
        }
        Cmd::Affirm { node, to } => {
            let store = Store::discover()?;
            let count = ops::affirm(&store, &node, to)?;
            if count == 0 {
                println!("nothing behind — no restamp needed.");
            } else {
                println!("✔ restamped {} ref(s)", count);
            }
        }
        Cmd::Archive { node, undo } => {
            let store = Store::discover()?;
            let n = ops::archive(&store, &node, undo)?;
            let all = store.load_all().unwrap_or_default();
            if undo {
                println!("✔ restored to default surfaces: {}", line(&all, &n));
            } else {
                println!("✔ archived: {}", line(&all, &n));
                println!("  still reachable — ids resolve, edges hold, blast/behind see it; surfaces show counts of what they hide.");
            }
        }
        Cmd::Open { node, all } => {
            let store = Store::discover()?;
            print!("{}", render::open(&store, &node, all)?);
            // An area open is the canonical read-first act: record it so the
            // first-touch gate passes silently on the diligent path.
            if let Ok(loaded) = store.load_all() {
                if let Ok(n) = store.find(&loaded, &node) {
                    if n.front.ty == "area" {
                        coord::record_area_read(&store, &coord::session_key(), &n.front.id);
                    }
                }
            }
        }
        Cmd::Brief { item } => {
            let store = Store::discover()?;
            let text = render::brief(&store, &item)?;
            print!("{}", text);
            let all = store.load_all()?;
            let n = store.find(&all, &item)?;
            store.log_event(serde_json::json!({
                "ts": Store::now(), "node": n.front.id, "v": n.front.v,
                "op": "brief", "actor": Store::actor()
            }))?;
        }
        Cmd::Log { node } => {
            let store = Store::discover()?;
            print!("{}", render::log(&store, &node)?);
        }
        Cmd::Query { which } => {
            let store = Store::discover()?;
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
                            println!("{}", line(&all, n));
                            shown += 1;
                        } else {
                            println!(
                                "{}  ⚠ write-set leased by session {} (\"{}\")",
                                line(&all, n),
                                foreign[0].session,
                                foreign[0].item_title
                            );
                        }
                    }
                    if shown == 0 {
                        println!("nothing dispatchable.");
                    } else {
                        println!("dispatch the chain in one act: q dispatch <item> --files <globs>  ·  solo: q brief <item>, then q reserve");
                    }
                }
                Query::Shaping { mine } => {
                    let items: Vec<&Node> =
                        queries::shaping(&all).into_iter().map(|(n, _)| n).collect();
                    for n in scope_mine(&store, &all, items, mine) {
                        println!("{}", line(&all, n));
                        for b in queries::live_blockers(&all, n) {
                            println!("    blocked on {}", aref(&all, b));
                        }
                    }
                    // Intent and reality share vocabulary — the delta derives.
                    // Advertised where shaping is judged; silence the default.
                    let d = queries::intent_delta(&all, None);
                    if !d.unlanded.is_empty() || !d.unintended.is_empty() {
                        println!(
                            "intent delta — plan and reality join by name: {} intended-but-unlanded, {} landed-but-unintended (q query intent-delta)",
                            d.unlanded.len(),
                            d.unintended.len()
                        );
                    }
                }
                Query::Queue { mine } => {
                    let q = scope_mine(&store, &all, queries::queue(&all), mine);
                    if q.is_empty() {
                        println!("queue is empty.");
                    }
                    let waiting = all
                        .iter()
                        .filter(|n| n.front.ty == "thread" && n.front.status == "queued")
                        .count()
                        .saturating_sub(queries::queue(&all).len());
                    for n in &q {
                        println!("{}", line(&all, n));
                    }
                    if waiting > 0 {
                        println!("({} queued thread(s) still blocked on intermediate work)", waiting);
                    }
                }
                Query::Behind => {
                    let b = queries::behind(&store, &all);
                    if b.is_empty() {
                        println!("nothing behind — every ref current.");
                    }
                    for e in b {
                        let target = e
                            .to_atom
                            .as_ref()
                            .map(quarry::surface::atom_ref)
                            .unwrap_or_else(|| format!("\"{}\" ({})", e.to_title, e.to));
                        println!(
                            "[sev {}] {} -[{}]→ {}  at {} now {}  ({})",
                            e.severity,
                            quarry::surface::atom_ref(&e.src),
                            e.rel,
                            target,
                            e.at,
                            e.current,
                            e.reason
                        );
                    }
                }
                Query::Blast { node } => {
                    let n = store.find(&all, &node)?;
                    let ids = queries::blast(&all, &n.front.id);
                    if ids.is_empty() {
                        println!("nothing leans on {}.", n.front.id);
                    }
                    for id in ids {
                        if let Some(m) = all.iter().find(|m| m.front.id == id) {
                            println!("{}", line(&all, m));
                        }
                    }
                }
                Query::Contested => {
                    let c = queries::contested(&all);
                    if c.is_empty() {
                        println!("no contested writes.");
                    }
                    for (src, rel, dst) in c {
                        println!(
                            "{} -[{}]-> {} (user-provenance)",
                            aref(&all, src), rel, aref(&all, dst)
                        );
                    }
                }
                Query::Idle { days } => {
                    for n in queries::idle(&all, days) {
                        println!("{}", line(&all, n));
                    }
                }
                Query::Unverified => {
                    let u = queries::unverified(&all);
                    if u.is_empty() {
                        println!("no unverified assistant claims.");
                    }
                    for n in u {
                        println!("{}", line(&all, n));
                    }
                }
                Query::Dispatch { item } => {
                    print!("{}", render::dispatch_trace(&store, &item)?);
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
                        println!("no intent delta — every capability named in live acceptance has a vein in a shared area, and every registered vein was named by some intent (or nothing is named yet).");
                    }
                    if !d.unlanded.is_empty() {
                        println!("intended but unlanded — named in live acceptance, no vein claim in a shared area carries it:");
                        for (name, item) in &d.unlanded {
                            println!("  · `{}` — {}", name, aref(&all, item));
                        }
                    }
                    if !d.unintended.is_empty() {
                        println!("landed but unintended — a vein no intent named (emergent scope, visible instead of silent):");
                        for (name, claim) in &d.unintended {
                            println!("  · `{}` — {}", name, aref(&all, claim));
                        }
                    }
                    if !d.unlanded.is_empty() || !d.unintended.is_empty() {
                        println!("(an index for judgment, never a sweep — land it, name it in an item's acceptance, or leave it and know why)");
                    }
                }
            }
        }
    }
    Ok(())
}
