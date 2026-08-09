use anyhow::Result;
use std::collections::{HashSet, VecDeque};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::model::{At, Node};
use crate::store::Store;

/// A live blocker of an item or thread: a depends-on target not yet landed/resolved.
pub fn live_blockers<'a>(all: &'a [Node], n: &Node) -> Vec<&'a Node> {
    let mut out = Vec::new();
    for e in &n.front.edges {
        if e.rel != "depends-on" {
            continue;
        }
        if let Some(t) = all.iter().find(|x| x.front.id == e.to) {
            let live = match t.front.ty.as_str() {
                "item" => !matches!(t.front.status.as_str(), "done" | "dropped"),
                "thread" => t.front.status != "resolved",
                "decision" => t.front.status != "in-force",
                _ => false,
            };
            if live {
                out.push(t);
            }
        }
    }
    out
}

/// Items marked ready with nothing actually blocking them.
pub fn ready<'a>(all: &'a [Node]) -> Vec<&'a Node> {
    all.iter()
        .filter(|n| n.front.ty == "item" && n.front.status == "ready")
        .filter(|n| live_blockers(all, n).is_empty())
        .collect()
}

/// Upcoming work (sketch/shaped) with whatever blocks each piece.
pub fn shaping<'a>(all: &'a [Node]) -> Vec<(&'a Node, Vec<&'a Node>)> {
    all.iter()
        .filter(|n| n.front.ty == "item" && matches!(n.front.status.as_str(), "sketch" | "shaped"))
        .map(|n| (n, live_blockers(all, n)))
        .collect()
}

/// Threads awaiting the user whose prerequisites have landed — answerable now.
pub fn queue<'a>(all: &'a [Node]) -> Vec<&'a Node> {
    let mut q: Vec<&Node> = all
        .iter()
        .filter(|n| n.front.ty == "thread" && n.front.status == "queued")
        .filter(|n| live_blockers(all, n).is_empty())
        .collect();
    q.sort_by(|a, b| a.front.created.cmp(&b.front.created));
    q
}

pub struct Behind {
    pub src_id: String,
    pub src_title: String,
    pub rel: String,
    pub to: String,
    pub to_title: String,
    pub at: String,
    pub current: String,
    pub severity: u8, // 1 dead target · 2 dangling · 3 content changed · 4 file drifted
    pub reason: String,
}

pub fn behind(store: &Store, all: &[Node]) -> Vec<Behind> {
    let mut out = Vec::new();
    for n in all {
        for e in &n.front.edges {
            match &e.at {
                At::V(v) => match all.iter().find(|t| t.front.id == e.to) {
                    None => out.push(Behind {
                        src_id: n.front.id.clone(),
                        src_title: n.front.title.clone(),
                        rel: e.rel.clone(),
                        to: e.to.clone(),
                        to_title: "?".into(),
                        at: format!("v{}", v),
                        current: "missing".into(),
                        severity: 2,
                        reason: "dangling — target not found".into(),
                    }),
                    Some(t) if t.front.v > *v => {
                        let dead = matches!(t.front.status.as_str(), "refuted" | "superseded");
                        out.push(Behind {
                            src_id: n.front.id.clone(),
                            src_title: n.front.title.clone(),
                            rel: e.rel.clone(),
                            to: t.front.id.clone(),
                            to_title: t.front.title.clone(),
                            at: format!("v{}", v),
                            current: format!("v{}", t.front.v),
                            severity: if dead { 1 } else { 3 },
                            reason: if dead {
                                format!("target is {}", t.front.status)
                            } else {
                                "target content changed".into()
                            },
                        });
                    }
                    _ => {}
                },
                At::Blob(b) => {
                    if let Some(f) = e.to.strip_prefix("file:") {
                        match store.blob(f) {
                            Ok(nb) if &nb != b => out.push(Behind {
                                src_id: n.front.id.clone(),
                                src_title: n.front.title.clone(),
                                rel: e.rel.clone(),
                                to: e.to.clone(),
                                to_title: f.into(),
                                at: b.clone(),
                                current: nb,
                                severity: 4,
                                reason: "file drifted".into(),
                            }),
                            Err(_) => out.push(Behind {
                                src_id: n.front.id.clone(),
                                src_title: n.front.title.clone(),
                                rel: e.rel.clone(),
                                to: e.to.clone(),
                                to_title: f.into(),
                                at: b.clone(),
                                current: "missing".into(),
                                severity: 2,
                                reason: "dangling — file not found".into(),
                            }),
                            _ => {}
                        }
                    }
                }
            }
        }
        // A path-backed doc whose file moved on without an affirm.
        if n.front.ty == "doc" {
            if let (Some(p), Some(old)) = (&n.front.path, &n.front.blob) {
                if let Ok(nb) = store.blob(p) {
                    if &nb != old {
                        out.push(Behind {
                            src_id: n.front.id.clone(),
                            src_title: n.front.title.clone(),
                            rel: "path".into(),
                            to: format!("file:{}", p),
                            to_title: p.clone(),
                            at: old.clone(),
                            current: nb,
                            severity: 4,
                            reason: "registered doc drifted — affirm to bump".into(),
                        });
                    }
                }
            }
        }
    }
    out.sort_by_key(|b| b.severity);
    out
}

/// Who leans on this node: forward closure of its `supports`, plus inbound
/// `depends-on` and `source` edges, transitively.
pub fn blast(all: &[Node], start: &str) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(start.to_string());
    let mut q: VecDeque<String> = VecDeque::new();
    q.push_back(start.to_string());
    let mut out = Vec::new();
    while let Some(cur) = q.pop_front() {
        if let Some(n) = all.iter().find(|n| n.front.id == cur) {
            for e in &n.front.edges {
                if e.rel == "supports" && seen.insert(e.to.clone()) {
                    out.push(e.to.clone());
                    q.push_back(e.to.clone());
                }
            }
        }
        for n in all {
            for e in &n.front.edges {
                if e.to == cur
                    && matches!(e.rel.as_str(), "depends-on" | "source")
                    && seen.insert(n.front.id.clone())
                {
                    out.push(n.front.id.clone());
                    q.push_back(n.front.id.clone());
                }
            }
        }
    }
    out
}

/// Assistant-provenance settles/supersedes against user-provenance targets.
/// C3 denies these at write time; nonzero here means data arrived another way.
pub fn contested<'a>(all: &'a [Node]) -> Vec<(&'a Node, &'a str, &'a Node)> {
    let mut out = Vec::new();
    for n in all {
        if n.front.provenance != "assistant" {
            continue;
        }
        for e in &n.front.edges {
            if !matches!(e.rel.as_str(), "settles" | "supersedes") {
                continue;
            }
            if let Some(t) = all.iter().find(|t| t.front.id == e.to) {
                if t.front.provenance == "user" {
                    out.push((n, e.rel.as_str(), t));
                }
            }
        }
    }
    out
}

/// Nodes nothing points at, older than `days`.
pub fn idle<'a>(all: &'a [Node], days: i64) -> Vec<&'a Node> {
    let now = OffsetDateTime::now_utc();
    all.iter()
        .filter(|n| {
            !all.iter()
                .any(|m| m.front.edges.iter().any(|e| e.to == n.front.id))
        })
        .filter(|n| {
            OffsetDateTime::parse(&n.front.created, &Rfc3339)
                .map(|c| (now - c).whole_days() >= days)
                .unwrap_or(false)
        })
        .collect()
}

/// Assistant claims never verified: no method, still asserted.
pub fn unverified<'a>(all: &'a [Node]) -> Vec<&'a Node> {
    all.iter()
        .filter(|n| {
            n.front.ty == "claim"
                && n.front.provenance == "assistant"
                && n.front.status == "asserted"
                && n.front.method.is_none()
        })
        .collect()
}

/// Graph-generic vocabulary excluded from relatedness matching: on any
/// quarry graph these words appear everywhere and carry no subject signal.
const GENERIC_TOKENS: &[&str] = &[
    "quarry", "graph", "session", "sessions", "node", "nodes", "area", "areas",
    "item", "items", "thread", "threads", "claim", "claims", "decision",
    "decisions", "doc", "docs", "verb", "verbs", "surface", "status", "user",
    "assistant", "title", "titles", "about", "content", "record", "records",
];

/// Distinctive tokens of a title: length ≥ 5, hyphen-compounds kept
/// (core-sample, deep-time), generic graph vocabulary dropped.
fn sig_tokens(title: &str) -> Vec<String> {
    let lower = title.to_lowercase();
    let mut out: Vec<String> = Vec::new();
    for raw in lower.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        let t = raw.trim_matches('-');
        if t.len() >= 5 && !GENERIC_TOKENS.contains(&t) && !out.iter().any(|x| x == t) {
            out.push(t.to_string());
        }
    }
    out
}

/// Word-boundary containment: `word` occurs in `text` not embedded in a
/// longer token ("wrap" must not hit "wrapper").
fn contains_word(text: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let mut start = 0;
    while let Some(pos) = text[start..].find(word) {
        let i = start + pos;
        let before_ok = text[..i]
            .chars()
            .last()
            .map_or(true, |c| !(c.is_ascii_alphanumeric() || c == '-'));
        let j = i + word.len();
        let after_ok = text[j..]
            .chars()
            .next()
            .map_or(true, |c| !(c.is_ascii_alphanumeric() || c == '-'));
        if before_ok && after_ok {
            return true;
        }
        start = j;
    }
    false
}

/// Mint-time relatedness: an index of candidates the new node's text touches,
/// for the minting agent to review — never auto-linked. Forward: the existing
/// title lexicon matched against the new node's title+body (backticked spans
/// strengthen). Reverse: the new title's distinctive tokens matched against
/// existing bodies — prior mentions of a concept that just earned its node.
/// Archived nodes, areas, and already-linked neighbors are excluded; the
/// strongest few qualify (silence is the default).
pub fn relatedness<'a>(all: &'a [Node], node: &Node) -> Vec<(&'a Node, String)> {
    let new_text = format!("{} {}", node.front.title, node.body).to_lowercase();
    let new_title_toks: Vec<String> = sig_tokens(&node.front.title)
        .into_iter()
        .filter(|t| t.len() >= 6)
        .collect();
    let backticked: Vec<String> = node
        .body
        .split('`')
        .skip(1)
        .step_by(2)
        .map(|s| s.trim().to_lowercase())
        .filter(|s| s.len() >= 4 && s.len() <= 60)
        .collect();
    let mut scored: Vec<(i32, &Node, String)> = Vec::new();
    for cand in all {
        if cand.front.id == node.front.id || cand.front.archived || cand.front.ty == "area" {
            continue;
        }
        if node.front.edges.iter().any(|e| e.to == cand.front.id)
            || cand.front.edges.iter().any(|e| e.to == node.front.id)
        {
            continue;
        }
        let title_lower = cand.front.title.to_lowercase();
        let toks = sig_tokens(&cand.front.title);
        let hits: Vec<&String> = toks.iter().filter(|t| contains_word(&new_text, t)).collect();
        let full_title = title_lower.len() >= 8 && new_text.contains(&title_lower);
        let tick = backticked
            .iter()
            .any(|b| title_lower.contains(b.as_str()) || toks.iter().any(|t| t == b));
        if full_title || hits.len() >= 2 || hits.iter().any(|t| t.len() >= 6) || tick {
            let why = if full_title {
                "mentions its title".to_string()
            } else if let Some(t) = hits.first() {
                format!("mentions '{}'", t)
            } else {
                "backtick reference".to_string()
            };
            let score = hits.len() as i32 + if full_title { 2 } else { 0 } + if tick { 2 } else { 0 };
            scored.push((score, cand, why));
            continue;
        }
        if !cand.body.is_empty() {
            let cbody = cand.body.to_lowercase();
            let rhits: Vec<&String> = new_title_toks
                .iter()
                .filter(|t| contains_word(&cbody, t))
                .collect();
            if !rhits.is_empty() {
                scored.push((
                    rhits.len() as i32,
                    cand,
                    format!("its body mentions '{}'", rhits[0]),
                ));
            }
        }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.front.id.cmp(&b.1.front.id)));
    scored.truncate(4);
    scored.into_iter().map(|(_, n, w)| (n, w)).collect()
}

/// Citers of `id` whose stamp is now behind the target's version — the
/// staleness homework a bump creates.
pub fn citers_behind<'a>(all: &'a [Node], id: &str) -> Vec<(&'a Node, &'a crate::model::Edge)> {
    let Some(target) = all.iter().find(|n| n.front.id == id) else {
        return vec![];
    };
    let mut out = Vec::new();
    for n in all {
        for e in &n.front.edges {
            if e.to == id {
                if let At::V(v) = &e.at {
                    if target.front.v > *v {
                        out.push((n, e));
                    }
                }
            }
        }
    }
    out
}

/// Nodes whose blockage just cleared because `id` reached a satisfied state —
/// the flow homework a resolution creates. Empty if `id` still blocks.
pub fn unblocked_by<'a>(all: &'a [Node], id: &str) -> Vec<&'a Node> {
    let Some(target) = all.iter().find(|n| n.front.id == id) else {
        return vec![];
    };
    let satisfied = match target.front.ty.as_str() {
        "item" => matches!(target.front.status.as_str(), "done" | "dropped"),
        "thread" => target.front.status == "resolved",
        "decision" => target.front.status == "in-force",
        _ => false,
    };
    if !satisfied {
        return vec![];
    }
    all.iter()
        .filter(|n| n.front.edges.iter().any(|e| e.rel == "depends-on" && e.to == id))
        .filter(|n| live_blockers(all, n).is_empty())
        .collect()
}

/// Events for one node, oldest first.
pub fn node_log(store: &Store, id: &str) -> Result<Vec<serde_json::Value>> {
    Ok(store
        .read_log()?
        .into_iter()
        .filter(|e| e.get("node").and_then(|v| v.as_str()) == Some(id))
        .collect())
}
