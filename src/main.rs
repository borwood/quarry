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
  q link it-4k7f about file:src/water/body.rs:42   # blob-stamped file ref")]
    Link {
        src: String,
        /// about | part-of | depends-on | settles | supports | refutes | supersedes | source
        rel: String,
        dst: String,
        /// Required to cite a refuted/superseded target (C5)
        #[arg(long)]
        acknowledge: bool,
        #[arg(long)]
        note: Option<String>,
    },
    /// Mutate fields: status=… title=… kind=… method=… path=… acceptance+=… ratified=…
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
Extract a claim only when something depends on the statement or kills it —
never while writing prose. C1: at least one --about. C2: non-user claims
name their --source doc.")]
    Claim {
        text: String,
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
    /// Render the whole graph as one self-contained HTML page (graph/view/index.html)
    View {
        /// Open the rendered page in the default browser
        #[arg(long)]
        open: bool,
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
            println!("    \"{}\" [{} {}] — {}", t.front.title, t.front.ty, t.front.id, why);
        }
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
                .map(|n| (n.front.id.clone(), n.front.title.clone()))
        })
        .collect();
    for (aid, title) in areas {
        match coord::touch_area(store, &all, &sess, &aid) {
            coord::AreaTouch::FirstTouch => {
                println!(
                    "  note: first touch of area \"{}\" this session without a read — the read-first: q open {}",
                    title, aid
                );
                coord::record_area_read(store, &sess, &aid);
            }
            coord::AreaTouch::Drift(lines) => {
                println!("  since your last read of \"{}\" (other sessions):", title);
                for l in lines {
                    println!("    · {}", l);
                }
            }
            coord::AreaTouch::Current => {}
        }
    }
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
        if n.front.ty == "area" && coord::area_cursor(store, &sess, &n.front.id).is_none() {
            unread.push((n.front.id.clone(), n.front.title.clone()));
        }
    }
    if unread.is_empty() {
        return Ok(false);
    }
    let token = quarry::protocol::save_intent(store, "new", serde_json::to_value(a)?)?;
    println!("⏸ gated: first write into unread area(s) this session — the read-first arrives now.");
    for (aid, title) in &unread {
        println!("\n── area \"{}\" ──", title);
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
    println!("✔ {}", line(&node));
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

fn line(n: &Node) -> String {
    format!(
        "\"{}\" — {} [{}] v{} ({})",
        n.front.title, n.front.ty, n.front.status, n.front.v, n.front.id
    )
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
                "⚠ behind: \"{}\" -[{}]→ \"{}\" (cited {}, now v{}). Review the change, then: q affirm {} --to {}",
                citer.front.title, e.rel, target.front.title, e.at, target.front.v,
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
            lines.push(format!("✔ {}: \"{}\" ({})", what, n.front.title, n.front.id));
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
            println!("  suggested .gitignore lines: graph/.index/  graph/view/");
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
                #[cfg(windows)]
                std::process::Command::new("cmd")
                    .args(["/C", "start", ""])
                    .arg(&path)
                    .spawn()?;
                #[cfg(not(windows))]
                std::process::Command::new("open").arg(&path).spawn()?;
            }
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
                    let rel = p
                        .strip_prefix(&root)
                        .map(|r| r.trim_start_matches('/').to_string())
                        .unwrap_or(p.clone());
                    let session = coord::current_session().or_else(|| {
                        serde_json::from_str::<serde_json::Value>(&input)
                            .ok()
                            .and_then(|v| {
                                v.get("session_id")
                                    .and_then(|x| x.as_str())
                                    .and_then(|cid| coord::chat_binding(&store, cid))
                            })
                    });
                    let dispatch = std::env::var("QUARRY_DISPATCH").ok().filter(|s| !s.trim().is_empty());
                    let leases = coord::load_leases(&store);
                    match quarry::teach::lease_check(
                        &leases,
                        session.as_deref(),
                        dispatch.as_deref(),
                        &rel,
                    ) {
                        quarry::teach::LeaseCheck::Deny(msg) => {
                            eprintln!("{}", msg);
                            std::process::exit(2);
                        }
                        quarry::teach::LeaseCheck::Warn(msg) => {
                            println!(
                                "{}",
                                serde_json::json!({"hookSpecificOutput": {
                                    "hookEventName": "PreToolUse",
                                    "additionalContext": msg
                                }})
                            );
                        }
                        quarry::teach::LeaseCheck::Allow => {}
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
                        let titles: Vec<String> =
                            queue.iter().map(|n| format!("\"{}\"", n.front.title)).collect();
                        println!(
                            "quarry: {} nodes · owed to the user: {}{} · ready to dispatch: {} · behind: {}",
                            all.len(),
                            queue.len(),
                            if titles.is_empty() { String::new() } else { format!(" ({})", titles.join(", ")) },
                            ready.len(),
                            behind.len()
                        );
                        if let Some((sess, areas)) = coord::purview(&store, &all) {
                            let ids: Vec<&str> = areas.iter().map(|a| a.front.id.as_str()).collect();
                            let mine_q = queue.iter().filter(|n| coord::in_purview(n, &ids)).count();
                            let mine_r = ready.iter().filter(|n| coord::in_purview(n, &ids)).count();
                            let names: Vec<&str> = areas.iter().map(|a| a.front.title.as_str()).collect();
                            println!(
                                "session {} purview ({}): {} answerable thread(s), {} ready item(s) — scope with --mine",
                                sess, names.join(", "), mine_q, mine_r
                            );
                            let leases = coord::load_leases(&store);
                            for l in leases.iter().filter(|l| l.session != sess) {
                                println!(
                                    "live lease elsewhere: session {} holds {:?} (\"{}\")",
                                    l.session, l.globs, l.item_title
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
                                                "new in your purview from session {}: \"{}\" [{}]",
                                                from, n.front.title, n.front.status
                                            );
                                        }
                                    }
                                }
                            }
                        } else if coord::current_session().is_some() {
                            println!(
                                "session '{}' has no registered purview — q session set <name> --areas <area>...",
                                coord::current_session().unwrap_or_default()
                            );
                        } else {
                            let bound = chat_id
                                .as_deref()
                                .and_then(|cid| coord::chat_binding(&store, cid));
                            let reg = coord::load_sessions(&store);
                            if bound.is_none() && !reg.is_empty() {
                                let names: Vec<&str> = reg.keys().map(|s| s.as_str()).collect();
                                println!(
                                    "unbound chat in a multi-session repo (sessions: {}). Before substantive work, ask the user: adopt one of these (q session adopt <name>), or define a new session — and if new, is it meant to persist across chats and be re-entered, or is it ephemeral, for this chat only?",
                                    names.join(" · ")
                                );
                            }
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
            let hits: Vec<&Node> = all
                .iter()
                .filter(|n| {
                    n.front.id.contains(&q)
                        || n.front.title.to_lowercase().contains(&q)
                        || n.body.to_lowercase().contains(&q)
                })
                .collect();
            if hits.is_empty() {
                println!("no node matches \"{}\".", text);
            }
            for n in hits {
                let where_ = if n.front.title.to_lowercase().contains(&q) || n.front.id.contains(&q) {
                    ""
                } else {
                    "  (matched in body)"
                };
                let arch = if n.front.archived { "  [ARCHIVED]" } else { "" };
                println!("{}{}{}", line(n), arch, where_);
            }
        }
        Cmd::Session { which } => {
            let store = Store::discover()?;
            match which {
                SessionCmd::Set { name, areas, charter, launcher, ephemeral } => {
                    let all = store.load_all()?;
                    let ids = coord::resolve_area_ids(&store, &all, &areas)?;
                    let titles: Vec<String> = ids
                        .iter()
                        .filter_map(|id| all.iter().find(|n| &n.front.id == id))
                        .map(|n| n.front.title.clone())
                        .collect();
                    let overlaps = coord::purview_overlaps(&store, &ids);
                    coord::save_session(&store, &name, ids, charter, ephemeral)?;
                    println!(
                        "✔ session {} covers: {}{}",
                        name,
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
                            .map(|n| n.front.title.clone())
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
                    let all = store.load_all()?;
                    let Some(sess) = coord::current_session() else {
                        anyhow::bail!("no QUARRY_SESSION set — launch via a session launcher, or ask the user which session this chat is and relaunch")
                    };
                    let reg = coord::load_sessions(&store);
                    let Some(p) = reg.get(&sess) else {
                        anyhow::bail!("session '{}' is not registered — q session set {} --areas <area>...", sess, sess)
                    };
                    println!(
                        "resuming session {}{}",
                        sess,
                        p.charter.as_ref().map(|c| format!(" — {}", c)).unwrap_or_default()
                    );
                    let area_titles: Vec<String> = p
                        .areas
                        .iter()
                        .filter_map(|id| all.iter().find(|n| &n.front.id == id))
                        .map(|n| n.front.title.clone())
                        .collect();
                    println!("  purview: {}", area_titles.join(" · "));
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
                    let leases = coord::load_leases(&store);
                    let mine: Vec<_> = leases.iter().filter(|l| l.session == sess).collect();
                    if !mine.is_empty() {
                        println!("  holdings:");
                        for l in mine {
                            let done = all
                                .iter()
                                .find(|n| n.front.id == l.item)
                                .map_or(false, |n| matches!(n.front.status.as_str(), "done" | "dropped"));
                            println!(
                                "    \"{}\" holds {:?}{}",
                                l.item_title,
                                l.globs,
                                if done { format!(" — item done; release: q release {}", l.item) } else { String::new() }
                            );
                        }
                    }
                    let inflight: Vec<_> = all
                        .iter()
                        .filter(|n| n.front.ty == "item" && n.front.status == "in-flight" && coord::in_purview(n, &ids))
                        .collect();
                    if !inflight.is_empty() {
                        println!("  in-flight in purview:");
                        for n in inflight {
                            println!("    {}", line(n));
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
                            let title = all.iter().find(|n| n.front.id == node).map(|n| n.front.title.as_str()).unwrap_or(node);
                            let op = ev.get("op").and_then(|v| v.as_str()).unwrap_or("?");
                            let ts = ev.get("ts").and_then(|v| v.as_str()).unwrap_or("").split('T').nth(1).unwrap_or("");
                            println!("    {} {} \"{}\"", ts, op, title);
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
                            println!("    {}", line(n));
                        }
                    }
                    let owed: Vec<&Node> = queries::queue(&all)
                        .into_iter()
                        .filter(|n| coord::in_purview(n, &ids))
                        .collect();
                    if !owed.is_empty() {
                        println!("  owed to the user in your purview:");
                        for n in owed {
                            println!("    {}", line(n));
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
                            .map(|n| n.front.title.clone())
                            .collect();
                        println!(
                            "{}: {}{}",
                            name,
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
                    "C8: no brief on record for \"{}\" from session {} — a lease follows a brief. Render it (q brief {}), read it, then reserve.",
                    node.front.title, sess, node.front.id
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
                "✔ lease: \"{}\" holds {:?}{} (session {})",
                node.front.title,
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
            coord::release(&store, &node, &sess, &Store::actor())?;
            println!("✔ released: \"{}\"", node.front.title);
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
                                format!("  (blocked on \"{}\")", blockers[0].front.title)
                            };
                            let mark = if i == 0 { " ← next up" } else { "" };
                            println!("{}. {}{}{}", i + 1, line(n), state, mark);
                        }
                    }
                }
                Some(QueueCmd::Push { thread }) => {
                    let n = store.find(&all, &thread)?;
                    if n.front.ty != "thread" {
                        anyhow::bail!("{} is a {}, not a thread", n.front.id, n.front.ty);
                    }
                    if n.front.status == "resolved" {
                        anyhow::bail!("\"{}\" is already resolved", n.front.title);
                    }
                    if q.contains(&n.front.id) {
                        anyhow::bail!("\"{}\" is already in the topic queue", n.front.title);
                    }
                    q.push(n.front.id.clone());
                    coord::save_topic_queue(&store, &q)?;
                    println!("✔ queued at {}: {}", q.len(), line(n));
                }
                Some(QueueCmd::Pop) => {
                    if q.is_empty() {
                        println!("topic queue is empty.");
                    } else {
                        let id = q.remove(0);
                        coord::save_topic_queue(&store, &q)?;
                        match store.find(&all, &id) {
                            Ok(n) => {
                                println!("now: {}", line(n));
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
                            println!("  next after this: \"{}\"", next.front.title);
                        }
                    }
                }
                Some(QueueCmd::Front { thread }) => {
                    let n = store.find(&all, &thread)?;
                    let Some(pos) = q.iter().position(|id| id == &n.front.id) else {
                        anyhow::bail!("\"{}\" is not in the topic queue — q queue push first", n.front.title);
                    };
                    let id = q.remove(pos);
                    q.insert(0, id);
                    coord::save_topic_queue(&store, &q)?;
                    println!("✔ front: {}", line(n));
                }
                Some(QueueCmd::Drop { thread }) => {
                    let n = store.find(&all, &thread)?;
                    let Some(pos) = q.iter().position(|id| id == &n.front.id) else {
                        anyhow::bail!("\"{}\" is not in the topic queue", n.front.title);
                    };
                    q.remove(pos);
                    coord::save_topic_queue(&store, &q)?;
                    println!("✔ dropped from the topic queue: {} (the thread itself is untouched)", line(n));
                }
            }
        }
        Cmd::Wrap => {
            let store = Store::discover()?;
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
                println!("    {}", line(n));
            }
            {
                // Session-touched review (user-ruled 2026-08-09): every node
                // this session created or adjusted since its last wrap, for a
                // final look while the context that wrote them is still warm.
                let sess_key = coord::current_session();
                let log = store.read_log()?;
                let touched = queries::session_touched(&log, sess_key.as_deref());
                if !touched.is_empty() {
                    println!(
                        "  session-touched since last wrap ({}) — final review: does each still say what you now know?",
                        touched.len()
                    );
                    for (id, op) in touched.iter().take(15) {
                        match all.iter().find(|n| &n.front.id == id) {
                            Some(n) => println!("    [{}] {}", op, line(n)),
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
                    println!(
                        "    [sev {}] \"{}\" -[{}]→ \"{}\" ({})",
                        e.severity, e.src_title, e.rel, e.to_title, e.reason
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
                    println!("    {}", line(n));
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
                    println!("    {}", line(n));
                }
            }
            let unver = queries::unverified(&all);
            if !unver.is_empty() {
                println!("  unverified assistant claims ({}):", unver.len());
                for n in unver {
                    println!("    {}", line(n));
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
                if !archivable.is_empty() {
                    println!(
                        "  archivable ({} settled leaf/childless node(s)) — q archive <node>; parents index their archived offspring:",
                        archivable.len()
                    );
                    for n in archivable.iter().take(6) {
                        println!("    {}", line(n));
                    }
                    if archivable.len() > 6 {
                        println!("    …and {} more", archivable.len() - 6);
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
                    let done = all
                        .iter()
                        .find(|n| n.front.id == l.item)
                        .map_or(false, |n| matches!(n.front.status.as_str(), "done" | "dropped"));
                    println!(
                        "    [{}] \"{}\" holds {:?}{} since {}{}",
                        owner,
                        l.item_title,
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
                            "  note: \"{}\" is now blocked on \"{}\" — it leaves ready/queue views until that lands.",
                            store.find(&all, &src_id)?.front.title, t.front.title
                        );
                    }
                }
            }
            drop(all);
            print_homework(&store, &[src_id.as_str(), edge.to.as_str()]);
            area_watermarks(&store, &src_id);
        }
        Cmd::Set { node, fields, note } => {
            let store = Store::discover()?;
            let n = ops::set(&store, &node, &fields, note)?;
            println!("✔ {}", line(&n));
            presence_note(&store, &n.front.id);
            print_homework(&store, &[n.front.id.as_str()]);
            area_watermarks(&store, &n.front.id);
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
            println!("✔ {}", line(&n));
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
            println!("✔ {}", line(&d));
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
            about,
            source,
            method,
            provenance,
            status,
        } => {
            let store = Store::discover()?;
            let n = ops::claim(&store, &text, about, source, method, provenance, status)?;
            println!("✔ {}", line(&n));
            print_mint_surfaces(&store, &n);
            area_watermarks(&store, &n.front.id);
        }
        Cmd::Refute { claim, by, note } => {
            let store = Store::discover()?;
            let (c, blast) = ops::refute(&store, &claim, &by, note)?;
            println!("✔ \"{}\" is now refuted ({})", c.front.title, c.front.id);
            if blast.is_empty() {
                println!("  nothing leaned on it.");
            } else {
                println!("  blast radius — these leaned on it:");
                for n in blast {
                    println!("    {}", line(&n));
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
            if undo {
                println!("✔ restored to default surfaces: {}", line(&n));
            } else {
                println!("✔ archived: {}", line(&n));
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
                            println!("{}", line(n));
                            shown += 1;
                        } else {
                            println!(
                                "{}  ⚠ write-set leased by session {} (\"{}\")",
                                line(n),
                                foreign[0].session,
                                foreign[0].item_title
                            );
                        }
                    }
                    if shown == 0 {
                        println!("nothing dispatchable.");
                    }
                }
                Query::Shaping { mine } => {
                    let items: Vec<&Node> =
                        queries::shaping(&all).into_iter().map(|(n, _)| n).collect();
                    for n in scope_mine(&store, &all, items, mine) {
                        println!("{}", line(n));
                        for b in queries::live_blockers(&all, n) {
                            println!("    blocked on \"{}\" [{}] ({})", b.front.title, b.front.status, b.front.id);
                        }
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
                        println!("{}", line(n));
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
                        println!(
                            "[sev {}] \"{}\" -[{}]→ \"{}\"  at {} now {}  ({})  [{} → {}]",
                            e.severity, e.src_title, e.rel, e.to_title, e.at, e.current, e.reason, e.src_id, e.to
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
                            println!("{}", line(m));
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
                            "{} \"{}\" -[{}]-> {} \"{}\" (user-provenance)",
                            src.front.id, src.front.title, rel, dst.front.id, dst.front.title
                        );
                    }
                }
                Query::Idle { days } => {
                    for n in queries::idle(&all, days) {
                        println!("{}", line(n));
                    }
                }
                Query::Unverified => {
                    let u = queries::unverified(&all);
                    if u.is_empty() {
                        println!("no unverified assistant claims.");
                    }
                    for n in u {
                        println!("{}", line(n));
                    }
                }
            }
        }
    }
    Ok(())
}
