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
        #[arg(long)]
        note: Option<String>,
    },
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
    /// Render a node's neighborhood brief
    Open { node: String },
    /// Per-node history from the event log
    Log { node: String },
    /// Canned queries
    #[command(alias = "q")]
    Query {
        #[command(subcommand)]
        which: Query,
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
when the arc lands; sessions start leaseless and reserve at dispatch.")]
    Reserve {
        item: String,
        #[arg(long = "files", required = true)]
        files: Vec<String>,
        #[arg(long)]
        shared: bool,
        #[arg(long)]
        steal: bool,
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
enum HookCmd {
    /// PreToolUse guard: denies freehand Write/Edit under graph/ (C6)
    Guard,
    /// SessionStart orientation: one-line graph summary for a fresh session
    Orient,
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
    },
    /// List registered sessions and their areas
    List,
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
            }
            HookCmd::Orient => {
                // Best-effort: a hook must never fail a session over a missing graph.
                if let Ok(store) = Store::discover() {
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
                println!("{}{}", line(n), where_);
            }
        }
        Cmd::Session { which } => {
            let store = Store::discover()?;
            match which {
                SessionCmd::Set { name, areas, charter } => {
                    let all = store.load_all()?;
                    let ids = coord::resolve_area_ids(&store, &all, &areas)?;
                    let titles: Vec<String> = ids
                        .iter()
                        .filter_map(|id| all.iter().find(|n| &n.front.id == id))
                        .map(|n| n.front.title.clone())
                        .collect();
                    coord::save_session(&store, &name, ids, charter)?;
                    println!("✔ session {} covers: {}", name, titles.join(" · "));
                    println!("  set QUARRY_SESSION={} in that session's environment.", name);
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
        } => {
            let store = Store::discover()?;
            let sess = coord::current_session().ok_or_else(|| {
                anyhow::anyhow!("no QUARRY_SESSION set — leases need a session identity (q session set <name> --areas ..., then export QUARRY_SESSION=<name>)")
            })?;
            let all = store.load_all()?;
            let node = store.find(&all, &item)?.clone();
            let out = coord::reserve(&store, &node, &sess, &Store::actor(), files.clone(), shared, steal)?;
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
            note,
        } => {
            let store = Store::discover()?;
            let body = read_body(body, body_file)?;
            let node = ops::new_node(
                &store,
                NewArgs {
                    ty,
                    title,
                    kind,
                    status,
                    provenance,
                    about,
                    path,
                    method,
                    ratified_by: ratified,
                    body,
                    acceptance,
                    note,
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
        }
        Cmd::Set { node, fields, note } => {
            let store = Store::discover()?;
            let n = ops::set(&store, &node, &fields, note)?;
            println!("✔ {}", line(&n));
            presence_note(&store, &n.front.id);
            print_homework(&store, &[n.front.id.as_str()]);
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
            let all = store.load_all()?;
            let th_id = store.find(&all, &thread)?.front.id.clone();
            drop(all);
            print_homework(&store, &[th_id.as_str(), d.front.id.as_str()]);
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
        Cmd::Open { node } => {
            let store = Store::discover()?;
            print!("{}", render::open(&store, &node)?);
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
