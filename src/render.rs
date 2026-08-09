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
    let has_claim_backlink = backlinks.iter().any(|(m, _)| m.front.ty == "claim");
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
        if n.front.ty == "area" && has_claim_backlink {
            writeln!(
                s,
                "  (claim backlinks are this area's load-bearing bones — build from spine before handrolling anew)"
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

/// The dispatch brief: DERIVED, NEVER HAND-WRITTEN. If it reads wrong, fix
/// the graph and re-render. Sections: read-first (the item's neighborhood at
/// pinned versions), write-set from the live lease (complement = do-not-
/// touch), acceptance as the RETURN spec, protocol rider (on=brief entries),
/// and the actor rules a dispatched agent works under.
pub fn brief(store: &Store, key: &str) -> Result<String> {
    let all = store.load_all()?;
    let item = store.find(&all, key)?;
    if item.front.ty != "item" {
        anyhow::bail!("{} is a {}, not an item — briefs dispatch items", item.front.id, item.front.ty);
    }
    let first_line = |body: &str| body.lines().next().unwrap_or("").trim().to_string();
    let mut s = String::new();
    writeln!(s, "══ DISPATCH BRIEF — \"{}\" ({} v{}) ══", item.front.title, item.front.id, item.front.v)?;
    writeln!(s, "derived from the graph at render time; if this brief reads wrong, the graph is wrong — fix the graph, re-render.")?;
    if let Some(k) = &item.front.kind {
        writeln!(s, "kind: {}", k)?;
    }
    if !item.body.trim().is_empty() {
        writeln!(s, "\nTHE WORK:")?;
        for l in item.body.lines() {
            writeln!(s, "  {}", l)?;
        }
    }

    writeln!(s, "\nREAD-FIRST (cited at pinned versions — q open <id> for any neighborhood):")?;
    let mut cited = 0usize;
    for e in &item.front.edges {
        if e.rel != "depends-on" {
            continue;
        }
        if let Some(t) = all.iter().find(|n| n.front.id == e.to) {
            writeln!(s, "  · depends on {} \"{}\" [{}] ({} v{})", t.front.ty, t.front.title, t.front.status, t.front.id, t.front.v)?;
            for l in t.body.lines() {
                writeln!(s, "      {}", l)?;
            }
            cited += 1;
        }
    }
    let area_ids: Vec<&str> = item
        .front
        .edges
        .iter()
        .filter(|e| e.rel == "about")
        .map(|e| e.to.as_str())
        .filter(|id| all.iter().any(|n| n.front.id == *id && n.front.ty == "area"))
        .collect();
    for aid in &area_ids {
        let Some(area) = all.iter().find(|n| n.front.id == *aid) else { continue };
        writeln!(s, "  · area \"{}\" ({} v{})", area.front.title, area.front.id, area.front.v)?;
        if !area.body.trim().is_empty() {
            writeln!(s, "      {}", first_line(&area.body))?;
        }
        for n in all.iter().filter(|n| !n.front.archived && crate::coord::in_purview(n, &[aid])) {
            match (n.front.ty.as_str(), n.front.status.as_str()) {
                ("decision", "in-force") => {
                    writeln!(s, "      decision in force: \"{}\" ({} v{}) — {}", n.front.title, n.front.id, n.front.v, first_line(&n.body))?;
                    cited += 1;
                }
                ("doc", "registered") => {
                    let p = n.front.path.as_deref().map(|p| format!(" · {}", p)).unwrap_or_default();
                    writeln!(s, "      doc: \"{}\" ({} v{}){}", n.front.title, n.front.id, n.front.v, p)?;
                    cited += 1;
                }
                ("thread", "open") | ("thread", "queued") => {
                    writeln!(s, "      open thread: \"{}\" ({}) — NOT yours to settle", n.front.title, n.front.id)?;
                }
                ("claim", "asserted") | ("claim", "measured") | ("claim", "ratified") => {
                    writeln!(s, "      spine: \"{}\" [{}] ({} v{})", n.front.title, n.front.status, n.front.id, n.front.v)?;
                    cited += 1;
                }
                _ => {}
            }
        }
    }
    let evidence: Vec<&crate::model::Node> = all
        .iter()
        .filter(|n| {
            n.front.edges.iter().any(|e| matches!(e.rel.as_str(), "supports") && e.to == item.front.id)
        })
        .collect();
    for ev in evidence {
        writeln!(s, "  · evidence: {} \"{}\" [{}] ({} v{})", ev.front.ty, ev.front.title, ev.front.status, ev.front.id, ev.front.v)?;
        cited += 1;
    }
    if cited == 0 {
        writeln!(s, "  (nothing cited — an item with no neighborhood usually means the graph is missing edges, not that there is nothing to read)")?;
    }

    writeln!(s, "\nWRITE-SET:")?;
    let leases = crate::coord::load_leases(store);
    match leases.iter().find(|l| l.item == item.front.id) {
        Some(l) => {
            writeln!(s, "  leased{}: {:?}", if l.shared { " [shared — co-writers may be present]" } else { "" }, l.globs)?;
            writeln!(s, "  everything outside those globs is DO-NOT-TOUCH.")?;
        }
        None => {
            writeln!(s, "  leaseless — research dispatch; DO-NOT-TOUCH: every file. To write code, the dispatcher reserves first: q reserve {} --files <globs>", item.front.id)?;
        }
    }

    writeln!(s, "\nRETURN SPEC (accept by outcome):")?;
    if item.front.acceptance.is_empty() {
        writeln!(s, "  ⚠ no acceptance recorded — outcomes cannot be judged. Fix the graph first: q set {} acceptance+=\"...\"", item.front.id)?;
    } else {
        for a in &item.front.acceptance {
            writeln!(s, "  · {}", a)?;
        }
        writeln!(s, "  Report against these outcomes — not effort, not process. A number needs its method; a mechanism is a hypothesis until measured.")?;
    }

    let riders = crate::protocol::matching(&all, "brief", None, item.front.kind.as_deref());
    for r in &riders {
        writeln!(s, "\nPROTOCOL — {}:", r.front.title)?;
        for l in r.body.lines() {
            writeln!(s, "  {}", l)?;
        }
    }

    writeln!(s, "\nACTOR RULES:")?;
    writeln!(s, "  · Build from spine: the claims above are load-bearing capabilities — design from these bones before proposing new structure.")?;
    writeln!(s, "  · All graph writes go through q verbs; your work logs under QUARRY_ACTOR (auto-injected).")?;
    writeln!(s, "  · C3: you may not settle or supersede user-provenance nodes; if a call belongs to the user, queue a thread.")?;
    writeln!(s, "  · Cite what you build on (q link ... / q claim --source ...); harvest is judged from the diff, not the report.")?;
    writeln!(s, "  · When the work lands: q set {} status=done, release any lease, and do the homework the verbs print.", item.front.id)?;
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
