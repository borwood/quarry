use anyhow::Result;
use std::fmt::Write as _;

use crate::model::{At, Node};
use crate::queries;
use crate::store::Store;

fn event_line(e: &serde_json::Value) -> String {
    let v = e.get("v").and_then(|x| x.as_u64()).unwrap_or(0);
    let op = e.get("op").and_then(|x| x.as_str()).unwrap_or("?");
    let ts = e
        .get("ts")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .split('T')
        .next()
        .unwrap_or("");
    let mut detail = String::new();
    match op {
        "set" => {
            if let Some(fields) = e.get("fields").and_then(|x| x.as_array()) {
                detail = fields
                    .iter()
                    .filter_map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
            } else if let (Some(f), Some(t)) = (
                e.get("field").and_then(|x| x.as_str()),
                e.get("to").and_then(|x| x.as_str()),
            ) {
                detail = format!("{} → {}", f, t);
            }
        }
        "link" => {
            detail = format!(
                "-[{}]-> {}",
                e.get("rel").and_then(|x| x.as_str()).unwrap_or("?"),
                e.get("to").and_then(|x| x.as_str()).unwrap_or("?")
            );
        }
        "create" => {
            detail = format!("\"{}\"", e.get("title").and_then(|x| x.as_str()).unwrap_or(""));
        }
        "affirm" => {
            detail = format!(
                "restamped {}",
                e.get("restamped").and_then(|x| x.as_u64()).unwrap_or(0)
            );
        }
        _ => {}
    }
    let note = e
        .get("note")
        .and_then(|x| x.as_str())
        .map(|n| format!("  — {}", n))
        .unwrap_or_default();
    format!("v{:<3} {}  {} {}{}", v, ts, op, detail, note)
}

/// The neighborhood brief: the node, its edges with staleness (and the log
/// delta for anything behind), and its backlinks. Archived neighbors are
/// hidden by default but always COUNTED with the reach-them hint — nothing
/// hides silently (`show_all` includes them).
pub fn open(store: &Store, key: &str, show_all: bool) -> Result<String> {
    let all = store.load_all()?;
    let n = store.find(&all, key)?;
    let log = store.read_log()?;
    let mut s = String::new();

    writeln!(
        s,
        "■ {}  {}  v{}  [{}]{}",
        n.front.id,
        n.front.ty,
        n.front.v,
        n.front.status,
        if n.front.archived { "  ARCHIVED" } else { "" }
    )?;
    writeln!(s, "  {}", n.front.title)?;
    let mut meta = format!(
        "  {} · {} · {}",
        n.front.provenance,
        n.front.actor,
        n.front.created.split('T').next().unwrap_or("")
    );
    if let Some(k) = &n.front.kind {
        meta.push_str(&format!(" · kind: {}", k));
    }
    if let Some(r) = &n.front.ratified {
        meta.push_str(&format!(" · RATIFIED by {} {}", r.by, r.date));
    }
    if let Some(p) = &n.front.path {
        meta.push_str(&format!(" · path: {}", p));
    }
    if let Some(m) = &n.front.method {
        meta.push_str(&format!(" · method: {}", m));
    }
    writeln!(s, "{}", meta)?;

    if !n.front.acceptance.is_empty() {
        writeln!(s, "\n  acceptance:")?;
        for a in &n.front.acceptance {
            writeln!(s, "    · {}", a)?;
        }
    }

    if !n.body.trim().is_empty() {
        writeln!(s)?;
        for line in n.body.lines() {
            writeln!(s, "  {}", line)?;
        }
    }

    let mut hidden_edges = 0usize;
    if !n.front.edges.is_empty() {
        writeln!(s, "\n  edges:")?;
        for e in &n.front.edges {
            if !show_all {
                if let At::V(_) = &e.at {
                    if all
                        .iter()
                        .find(|t| t.front.id == e.to)
                        .map_or(false, |t| t.front.archived)
                    {
                        hidden_edges += 1;
                        continue;
                    }
                }
            }
            match &e.at {
                At::V(v) => {
                    let t = all.iter().find(|t| t.front.id == e.to);
                    match t {
                        Some(t) => {
                            let marker = if t.front.v > *v {
                                if matches!(t.front.status.as_str(), "refuted" | "superseded") {
                                    format!("  ✗ BEHIND (at v{}, now v{} — {})", v, t.front.v, t.front.status)
                                } else {
                                    format!("  ⚠ behind (at v{}, now v{})", v, t.front.v)
                                }
                            } else {
                                format!("  (v{} ✓)", v)
                            };
                            writeln!(
                                s,
                                "    {:<10} → {} \"{}\" [{}]{}",
                                e.rel, t.front.id, t.front.title, t.front.status, marker
                            )?;
                            if t.front.v > *v {
                                for ev in log.iter().filter(|ev| {
                                    ev.get("node").and_then(|x| x.as_str()) == Some(e.to.as_str())
                                        && ev.get("v").and_then(|x| x.as_u64()).unwrap_or(0) > *v
                                }) {
                                    writeln!(s, "        · {}", event_line(ev))?;
                                }
                            }
                        }
                        None => writeln!(s, "    {:<10} → {} (MISSING)", e.rel, e.to)?,
                    }
                }
                At::Blob(b) => {
                    let f = e.to.strip_prefix("file:").unwrap_or(&e.to);
                    let marker = match store.blob(f) {
                        Ok(nb) if &nb == b => "  ✓".to_string(),
                        Ok(nb) => format!("  ⚠ drifted (at {}, now {})", b, nb),
                        Err(_) => "  ✗ MISSING".to_string(),
                    };
                    writeln!(s, "    {:<10} → {}{}", e.rel, e.to, marker)?;
                }
            }
        }
    }

    if hidden_edges > 0 {
        writeln!(
            s,
            "  ({} archived edge target(s) hidden — q open {} --all)",
            hidden_edges, n.front.id
        )?;
    }

    let backlinks_all: Vec<(&Node, &crate::model::Edge)> = all
        .iter()
        .flat_map(|m| m.front.edges.iter().map(move |e| (m, e)))
        .filter(|(_, e)| e.to == n.front.id)
        .collect();
    let hidden_back = backlinks_all
        .iter()
        .filter(|(m, _)| m.front.archived && !show_all)
        .count();
    let backlinks: Vec<&(&Node, &crate::model::Edge)> = backlinks_all
        .iter()
        .filter(|(m, _)| show_all || !m.front.archived)
        .collect();
    if !backlinks.is_empty() || hidden_back > 0 {
        writeln!(s, "\n  backlinks:")?;
        for &(m, e) in backlinks {
            let stale = match &e.at {
                At::V(v) if n.front.v > *v => {
                    format!("  ⚠ cited at v{}, this is v{}", v, n.front.v)
                }
                _ => String::new(),
            };
            writeln!(
                s,
                "    {:<10} ← {} \"{}\" [{}]{}",
                e.rel, m.front.id, m.front.title, m.front.status, stale
            )?;
        }
        if hidden_back > 0 {
            writeln!(
                s,
                "    ({} archived backlink(s) hidden — q open {} --all)",
                hidden_back, n.front.id
            )?;
        }
    }

    if matches!(n.front.ty.as_str(), "item" | "thread") {
        let blockers = queries::live_blockers(&all, n);
        if !blockers.is_empty() {
            writeln!(s, "\n  blocked on:")?;
            for b in blockers {
                writeln!(s, "    {} \"{}\" [{}]", b.front.id, b.front.title, b.front.status)?;
            }
        }
    }

    Ok(s)
}

pub fn log(store: &Store, key: &str) -> Result<String> {
    let all = store.load_all()?;
    let n = store.find(&all, key)?;
    let mut s = String::new();
    writeln!(s, "history of {} \"{}\":", n.front.id, n.front.title)?;
    for e in queries::node_log(store, &n.front.id)? {
        writeln!(s, "  {}", event_line(&e))?;
    }
    Ok(s)
}
