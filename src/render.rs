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
                "  (claim backlinks are this area's `load-bearing bones` — build from spine before handrolling anew)"
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

    // The area read-first is where intent meets its reality — the delta
    // advertises itself here, scoped, only when non-empty.
    if n.front.ty == "area" {
        let d = queries::intent_delta(&all, Some(n.front.id.as_str()));
        if !d.unlanded.is_empty() || !d.unintended.is_empty() {
            writeln!(
                s,
                "\n  intent delta — plan and reality join by name: {} intended-but-unlanded, {} landed-but-unintended (q query intent-delta {})",
                d.unlanded.len(),
                d.unintended.len(),
                n.front.id
            )?;
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

    // Render-time behind check, addressed to the DISPATCHER: a brief built
    // on stale stamps dispatches stale context — confront it before the
    // agent has to.
    let behinds: Vec<crate::queries::Behind> = crate::queries::behind(store, &all)
        .into_iter()
        .filter(|b| b.src_id == item.front.id)
        .collect();
    if !behinds.is_empty() {
        writeln!(s, "\nDISPATCHER — BEHIND CHECK ({} stale ref(s) at render time):", behinds.len())?;
        for b in &behinds {
            writeln!(
                s,
                "  ⚠ [sev {}] this item cites \"{}\" at {}, now {} ({}) — review the change, then: q affirm {} --to {}",
                b.severity, b.to_title, b.at, b.current, b.reason, item.front.id, b.to
            )?;
        }
        writeln!(s, "  Do not hand this off until each is reviewed — the agent inherits what you did not confront.")?;
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
                    // The spine shelf carries BODIES, not name-tags: the
                    // agent builds from these bones without a round-trip.
                    writeln!(s, "      spine: \"{}\" [{}] ({} v{})", n.front.title, n.front.status, n.front.id, n.front.v)?;
                    if !n.body.trim().is_empty() && n.body.trim() != n.front.title.trim() {
                        for l in n.body.lines() {
                            writeln!(s, "        {}", l)?;
                        }
                    }
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
        let p = ev.front.path.as_deref().map(|p| format!(" · {}", p)).unwrap_or_default();
        writeln!(s, "  · evidence: {} \"{}\" [{}] ({} v{}){}", ev.front.ty, ev.front.title, ev.front.status, ev.front.id, ev.front.v, p)?;
        if ev.front.kind.as_deref() == Some("report") {
            writeln!(s, "      a prior dispatch's report — read it before repeating its ground.")?;
        }
        cited += 1;
    }
    if cited == 0 {
        writeln!(s, "  (nothing cited — an item with no neighborhood usually means the graph is missing edges, not that there is nothing to read)")?;
    }
    // Backlinks: who leans on this item — landing context the forward edges
    // cannot show.
    let leaners: Vec<(&Node, &str)> = all
        .iter()
        .flat_map(|m| {
            m.front
                .edges
                .iter()
                .filter(|e| e.to == item.front.id && matches!(e.rel.as_str(), "depends-on" | "part-of"))
                .map(move |e| (m, e.rel.as_str()))
        })
        .filter(|(m, _)| !m.front.archived)
        .collect();
    if !leaners.is_empty() {
        writeln!(s, "  who leans on this landing:")?;
        for (m, rel) in leaners {
            writeln!(s, "    · \"{}\" [{}] ({}) -[{}]→ this item", m.front.title, m.front.status, m.front.id, rel)?;
        }
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
            writeln!(s, "  (the whole chain in one act — brief, lease, in-flight, hand-off payload: q dispatch {} --files <globs>)", item.front.id)?;
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
    writeln!(s, "  REFLECTIONS (always): close the report with doubts, surprises, and design friction in your own words — candor beats polish; reflections are mined afterward.")?;
    writeln!(s, "  STOP-REPORTS: stopping before acceptance is met is a valid outcome — say so explicitly (why, where you stopped, what remains) and the dispatcher re-dispatches from your report. A partial report registers like any other.")?;

    let riders = crate::protocol::matching(&all, "brief", None, item.front.kind.as_deref());
    for r in &riders {
        writeln!(s, "\nPROTOCOL — {}:", r.front.title)?;
        for l in r.body.lines() {
            writeln!(s, "  {}", l)?;
        }
    }

    writeln!(s, "\nACTOR RULES:")?;
    writeln!(s, "  · Build from spine: the claims above are `load-bearing capabilities` — design from these bones before proposing new structure.")?;
    writeln!(s, "  · All graph writes go through q verbs; your work logs under QUARRY_ACTOR (auto-injected).")?;
    writeln!(s, "  · C3: never settle or supersede user-provenance nodes. When the work hits a call that is the user's — a fork that would contradict or overturn a user ruling — queue a thread, take the least-committal provisional path consistent with standing rulings, and keep building; flag the provisional call in your report. The thread surfaces the call; it never blocks your arc.")?;
    writeln!(s, "  · Cite what you build on (q link ... / q claim --source ...); harvest is judged from the diff, not the report.")?;
    writeln!(s, "  · Landing belongs to the dispatcher: report against the RETURN spec and stop — never flip status=done, never release the lease. \"Done\" from an agent is a stop signal, not a transition; the dispatcher judges at: q harvest {}", item.front.id)?;
    Ok(s)
}

/// The harvest surface: the dispatcher's judgment seat at a dispatch's exit.
/// Observed-vs-leased, the acts stamped under the badge, the RETURN spec to
/// judge against, and the report-registration homework. Prints; never
/// transitions — landing stays the dispatcher's own act.
pub fn harvest(store: &Store, key: &str) -> Result<String> {
    let all = store.load_all()?;
    let item = store.find(&all, key)?;
    if item.front.ty != "item" {
        anyhow::bail!("{} is a {}, not an item — harvest judges dispatched items", item.front.id, item.front.ty);
    }
    let id = &item.front.id;
    let log = store.read_log()?;
    let mut s = String::new();
    writeln!(s, "══ HARVEST — \"{}\" ({}) [{}] ══", item.front.title, id, item.front.status)?;
    writeln!(s, "the agent's \"done\" was a stop signal, never a transition — judge by outcome, land by your own hand.")?;

    let observed = crate::coord::touched_for(store, &format!("item:{}", id));
    let globs = crate::coord::load_leases(store)
        .iter()
        .find(|l| &l.item == id)
        .map(|l| l.globs.clone())
        .or_else(|| crate::coord::load_dispatch(store).filter(|d| &d.item == id).map(|d| d.globs))
        .unwrap_or_else(|| item.front.write_set.clone());
    writeln!(s, "\nOBSERVED vs LEASED:")?;
    if observed.is_empty() {
        writeln!(s, "  no code writes observed under this badge — a research dispatch, or the agent's shells ran outside the guard's sight.")?;
    } else {
        writeln!(s, "  files touched under the badge ({}):", observed.len())?;
        for f in &observed {
            let inside = globs.iter().any(|g| crate::coord::globs_overlap(g, f));
            writeln!(s, "    {}{}", f, if inside { "" } else { "  ⚠ outside the lease" })?;
        }
    }
    let untouched: Vec<&String> = globs
        .iter()
        .filter(|g| !observed.iter().any(|f| crate::coord::globs_overlap(g, f)))
        .collect();
    for g in untouched {
        writeln!(s, "  leased but untouched: {} — dead weight in the lease, or unfinished work?", g)?;
    }

    let acts: Vec<&serde_json::Value> = log
        .iter()
        .filter(|ev| ev.get("dispatch").and_then(|v| v.as_str()) == Some(id.as_str()))
        .filter(|ev| !matches!(ev.get("op").and_then(|v| v.as_str()), Some("dispatch") | Some("harvest")))
        .collect();
    let count_of = |ty: &str| {
        acts.iter()
            .filter(|ev| {
                ev.get("op").and_then(|v| v.as_str()) == Some("create")
                    && ev.get("type").and_then(|v| v.as_str()) == Some(ty)
            })
            .count()
    };
    writeln!(
        s,
        "\nGRAPH ACTS UNDER THE BADGE: {} event(s) — {} claim(s) minted, {} thread(s) filed, {} doc(s) registered. Full trace: q query dispatch {}",
        acts.len(), count_of("claim"), count_of("thread"), count_of("doc"), id
    )?;

    if !item.front.acceptance.is_empty() {
        writeln!(s, "\nJUDGE EACH BY OUTCOME (the RETURN spec):")?;
        for a in &item.front.acceptance {
            writeln!(s, "  · {}", a)?;
        }
    }

    let area = item
        .front
        .edges
        .iter()
        .find(|e| e.rel == "about" && all.iter().any(|n| n.front.id == e.to && n.front.ty == "area"))
        .map(|e| e.to.clone())
        .unwrap_or_else(|| "<area>".into());
    writeln!(s, "\nHOMEWORK — register the report (a stop/partial report registers the same way):")?;
    writeln!(
        s,
        "  q new doc \"dispatch report: {}\" --kind report --path <report.md> --about {} --supports {}",
        item.front.title, area, id
    )?;
    writeln!(s, "  (the supports edge carries it into any re-dispatch brief — prior reports ride along.)")?;
    writeln!(s, "\nLANDING (yours, if the outcomes hold): q set {} status=done · q release {}", id, id)?;
    writeln!(s, "  not yet earned → re-dispatch from the report: q dispatch {}", id)?;
    Ok(s)
}

/// What a dispatch wrote: the badge-stamped events and the accrued touches.
pub fn dispatch_trace(store: &Store, key: &str) -> Result<String> {
    let all = store.load_all()?;
    let item = store.find(&all, key)?;
    let id = &item.front.id;
    let log = store.read_log()?;
    let mut s = String::new();
    writeln!(s, "dispatch trace for \"{}\" ({}):", item.front.title, id)?;
    let mut any = false;
    for ev in log
        .iter()
        .filter(|ev| ev.get("dispatch").and_then(|v| v.as_str()) == Some(id.as_str()))
    {
        let node = ev.get("node").and_then(|v| v.as_str()).unwrap_or("?");
        let title = all
            .iter()
            .find(|n| n.front.id == node)
            .map(|n| n.front.title.as_str())
            .unwrap_or(node);
        let ts = ev.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        let op = ev.get("op").and_then(|v| v.as_str()).unwrap_or("?");
        writeln!(s, "  {} {} \"{}\" ({})", ts, op, title, node)?;
        any = true;
    }
    let touched = crate::coord::touched_for(store, &format!("item:{}", id));
    if !touched.is_empty() {
        writeln!(s, "  files touched (accrued by the write guard):")?;
        for f in &touched {
            writeln!(s, "    {}", f)?;
        }
        any = true;
    }
    if !any {
        writeln!(s, "  nothing under this badge yet — badge-stamped acts and guard-observed writes will appear here.")?;
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
