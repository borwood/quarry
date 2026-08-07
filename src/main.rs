use anyhow::Result;
use clap::{Parser, Subcommand};

use quarry::model::Node;
use quarry::ops::{self, NewArgs};
use quarry::queries;
use quarry::render;
use quarry::store::Store;

#[derive(Parser)]
#[command(name = "q", version, about = "quarry — a work graph for AI-native development")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Bootstrap graph/ in the current directory
    Init,
    /// Mint a node
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
    Rule {
        thread: String,
        text: String,
        #[arg(long)]
        by: Option<String>,
        #[arg(long)]
        title: Option<String>,
    },
    /// Extract a claim (extraction-on-citation)
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
}

#[derive(Subcommand)]
enum Query {
    /// Items dispatchable right now
    Ready,
    /// Upcoming work (sketch/shaped) and what blocks each piece
    Shaping,
    /// Threads awaiting the user, answerable now
    Queue,
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
        "{}  v{:<2} [{:<9}] {}",
        n.front.id, n.front.v, n.front.status, n.front.title
    )
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init => {
            let cwd = std::env::current_dir()?;
            Store::init(&cwd)?;
            println!("✔ graph/ initialized at {}", cwd.display());
            println!("  suggested .gitignore lines: graph/.index/  graph/view/");
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
        }
        Cmd::Set { node, fields, note } => {
            let store = Store::discover()?;
            let n = ops::set(&store, &node, &fields, note)?;
            println!("✔ {}", line(&n));
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
            println!("✔ {} is now refuted", c.front.id);
            if blast.is_empty() {
                println!("  nothing leaned on it.");
            } else {
                println!("  blast radius — these leaned on it:");
                for n in blast {
                    println!("    {}", line(&n));
                }
            }
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
                Query::Ready => {
                    let r = queries::ready(&all);
                    if r.is_empty() {
                        println!("nothing dispatchable.");
                    }
                    for n in r {
                        println!("{}", line(n));
                    }
                }
                Query::Shaping => {
                    for (n, blockers) in queries::shaping(&all) {
                        println!("{}", line(n));
                        for b in blockers {
                            println!("    blocked on {} \"{}\" [{}]", b.front.id, b.front.title, b.front.status);
                        }
                    }
                }
                Query::Queue => {
                    let q = queries::queue(&all);
                    if q.is_empty() {
                        println!("queue is empty.");
                    }
                    let waiting = all
                        .iter()
                        .filter(|n| n.front.ty == "thread" && n.front.status == "queued")
                        .count()
                        - q.len();
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
                            "[sev {}] {} \"{}\" -[{}]-> {} \"{}\"  at {} now {}  ({})",
                            e.severity, e.src_id, e.src_title, e.rel, e.to, e.to_title, e.at, e.current, e.reason
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
