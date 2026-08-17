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

    write!(s, "{}", crate::surface::atom_head(&all, n))?;

    if !n.front.acceptance.is_empty() {
        writeln!(s, "\n  acceptance:")?;
        for a in &n.front.acceptance {
            writeln!(s, "    · {}", a)?;
        }
    }

    if !n.body.trim().is_empty() {
        writeln!(s)?;
        for line in crate::mention::unpack(&all, n, &n.body).lines() {
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
                                "    {:<10} → {}{}",
                                e.rel,
                                crate::surface::atom_ref(&crate::surface::atom(&all, t)),
                                marker
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

    // An area's live claims leave the generic backlinks for the species
    // shelf below (shelf-by-claim-kind, dc-xfgz) — rendered once. Dead
    // claims (refuted/superseded) stay ordinary backlinks, statuses loud.
    let is_area = n.front.ty == "area";
    let shelf_claim = |m: &Node, e: &crate::model::Edge| {
        is_area
            && m.front.ty == "claim"
            && e.rel == "about"
            && matches!(m.front.status.as_str(), "asserted" | "measured" | "ratified")
    };
    let backlinks_all: Vec<(&Node, &crate::model::Edge)> = all
        .iter()
        .flat_map(|m| m.front.edges.iter().map(move |e| (m, e)))
        .filter(|(_, e)| e.to == n.front.id)
        .filter(|(m, e)| !shelf_claim(m, e))
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
                "    {:<10} ← {}{}",
                e.rel,
                crate::surface::atom_ref(&crate::surface::atom(&all, m)),
                stale
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

    // The species shelf (shelf-by-claim-kind, dc-xfgz · dc-gaa9): an
    // area's live claims render per species, each species under its
    // framing's one-line disposition — the same shelf the brief tiers
    // against dispatched work, at the directory register here.
    if is_area {
        let claims_all: Vec<&Node> = all
            .iter()
            .filter(|m| {
                m.front.ty == "claim"
                    && matches!(m.front.status.as_str(), "asserted" | "measured" | "ratified")
                    && m.front.edges.iter().any(|e| e.rel == "about" && e.to == n.front.id)
            })
            .collect();
        let hidden_claims = claims_all.iter().filter(|c| c.front.archived && !show_all).count();
        let claims: Vec<&Node> = claims_all
            .iter()
            .filter(|c| show_all || !c.front.archived)
            .copied()
            .collect();
        if !claims.is_empty() || hidden_claims > 0 {
            writeln!(s, "\n  claims shelf — this area's record by species (the brief tiers these against dispatched work; q open <id> digs in):")?;
            let mut grouped: Vec<(String, Option<&str>, Vec<&Node>)> = Vec::new();
            for (kind, label, framing) in crate::framings::SPECIES {
                let group: Vec<&Node> = claims
                    .iter()
                    .filter(|c| c.front.kind.as_deref() == Some(kind))
                    .copied()
                    .collect();
                if !group.is_empty() {
                    grouped.push((
                        label.to_string(),
                        Some(crate::framings::first_sentence(framing)),
                        group,
                    ));
                }
            }
            let mut other_kinds: Vec<String> = claims
                .iter()
                .filter_map(|c| c.front.kind.clone())
                .filter(|k| !crate::framings::SPECIES.iter().any(|(sk, _, _)| *sk == k.as_str()))
                .collect();
            other_kinds.sort();
            other_kinds.dedup();
            for k in &other_kinds {
                let group: Vec<&Node> = claims
                    .iter()
                    .filter(|c| c.front.kind.as_deref() == Some(k.as_str()))
                    .copied()
                    .collect();
                grouped.push((format!("CLAIMS·{}", k.to_uppercase()), None, group));
            }
            let kindless: Vec<&Node> =
                claims.iter().filter(|c| c.front.kind.is_none()).copied().collect();
            if !kindless.is_empty() {
                grouped.push(("CLAIMS (no species recorded)".to_string(), None, kindless));
            }
            for (label, disposition, group) in &grouped {
                writeln!(
                    s,
                    "    {}{}",
                    label,
                    disposition.map(|d| format!(" — {}", d)).unwrap_or_default()
                )?;
                for c in group {
                    writeln!(s, "      · {}", crate::surface::atom_line(&crate::surface::atom(&all, c)))?;
                }
            }
            if hidden_claims > 0 {
                writeln!(
                    s,
                    "    ({} archived claim(s) hidden — q open {} --all)",
                    hidden_claims, n.front.id
                )?;
            }
        }
    }

    // Derived mentions, outbound: the nodes this body cites — the forward
    // face of the mention index, parallel to mentioned-by below. Derived,
    // never stored; blast and behind never traverse these.
    let out_all = crate::mention::mentions_out(&all, n);
    let hidden_out = out_all
        .iter()
        .filter(|m| m.front.archived && !show_all)
        .count();
    let outs: Vec<&&Node> = out_all
        .iter()
        .filter(|m| show_all || !m.front.archived)
        .collect();
    if !outs.is_empty() || hidden_out > 0 {
        writeln!(s, "\n  mentions → (derived from this body's citations — a mention references; an edge leans):")?;
        for m in outs {
            writeln!(s, "    {}", crate::surface::atom_ref(&crate::surface::atom(&all, m)))?;
        }
        if hidden_out > 0 {
            writeln!(
                s,
                "    ({} archived mention(s) hidden — q open {} --all)",
                hidden_out, n.front.id
            )?;
        }
    }

    // Derived mentions: bodies citing this id — backlinks nobody stored.
    // Distinct from edges: a mention references, an edge leans, so blast
    // and behind never traverse these.
    let mentions_all = crate::mention::mentioned_by(&all, &n.front.id);
    let hidden_mentions = mentions_all
        .iter()
        .filter(|m| m.front.archived && !show_all)
        .count();
    let mentions: Vec<&&Node> = mentions_all
        .iter()
        .filter(|m| show_all || !m.front.archived)
        .collect();
    if !mentions.is_empty() || hidden_mentions > 0 {
        writeln!(s, "\n  ← mentioned by (derived from body citations — a mention references; an edge leans):")?;
        for m in mentions {
            writeln!(s, "    {}", crate::surface::atom_ref(&crate::surface::atom(&all, m)))?;
        }
        if hidden_mentions > 0 {
            writeln!(
                s,
                "    ({} archived mention(s) hidden — q open {} --all)",
                hidden_mentions, n.front.id
            )?;
        }
    }

    if matches!(n.front.ty.as_str(), "item" | "thread") {
        let blockers = queries::live_blockers(&all, n);
        if !blockers.is_empty() {
            writeln!(s, "\n  blocked on:")?;
            for b in blockers {
                writeln!(s, "    {}", crate::surface::atom_ref(&crate::surface::atom(&all, b)))?;
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

/// The truncated register (dc-xfgz): every body line carrying a matched
/// term renders, elisions mark themselves [...], and the dig-in command
/// closes at the call site. Lines match through the trio-repaired lexicon
/// predicate, so inflection and compound halves keep matching here too.
fn truncated_lines(body: &str, terms: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut gap = false;
    for line in body.lines() {
        let ll = line.to_lowercase();
        if terms.iter().any(|t| crate::queries::contains_word(&ll, t)) {
            if gap {
                out.push("[...]".into());
            }
            out.push(line.to_string());
            gap = false;
        } else {
            gap = true;
        }
    }
    if gap && !out.is_empty() {
        out.push("[...]".into());
    }
    out
}

/// One tiered backdrop section (dc-xfgz, dc-hjad): entries sort weight
/// descending then alphabetical; 3+ distinct shared terms (or a shared
/// backticked capability name) earn the full body, 1-2 earn the truncated
/// register with its dig-in command, 0 fold into the counted remainder
/// with the reach-them hint — counted, never hidden. A node whose body
/// already rendered in an earlier section shows a one-line ref (render
/// once at highest earned fidelity). Typography per dc-xfgz: double
/// newlines between entries showing body, single between one-liners.
#[allow(clippy::too_many_arguments)]
fn backdrop_section(
    s: &mut String,
    all: &[Node],
    ctx: &crate::queries::ShelfCtx,
    label: &str,
    framing: Option<&str>,
    nodes: &[&Node],
    placed: &mut std::collections::HashMap<String, String>,
    hint: &str,
) -> Result<()> {
    if nodes.is_empty() {
        return Ok(());
    }
    let mut scored: Vec<(&Node, crate::queries::ShelfMatch)> = nodes
        .iter()
        .map(|n| (*n, crate::queries::shelf_match(all, ctx, n)))
        .collect();
    scored.sort_by(|a, b| {
        b.1.weight.cmp(&a.1.weight).then_with(|| {
            crate::surface::title_raw(a.0)
                .to_lowercase()
                .cmp(&crate::surface::title_raw(b.0).to_lowercase())
        })
    });
    writeln!(s, "\n  {}:", label)?;
    if let Some(f) = framing {
        writeln!(s, "  {}", f)?;
    }
    let mut unmatched = 0usize;
    for (n, m) in &scored {
        let already = placed.get(&n.front.id).cloned();
        if already.is_none() && m.weight == 0 && !m.capability {
            unmatched += 1;
            continue;
        }
        let line = crate::surface::atom_line(&crate::surface::atom(all, n));
        let path = if n.front.ty == "doc" {
            n.front.path.as_deref().map(|p| format!(" · {}", p)).unwrap_or_default()
        } else {
            String::new()
        };
        if let Some(sec) = already {
            writeln!(s, "    · {}{} — body in {} above", line, path, sec)?;
            continue;
        }
        writeln!(s, "    · {}{}", line, path)?;
        let body = crate::mention::unpack(all, n, &n.body);
        let has_body =
            !n.body.trim().is_empty() && n.body.trim() != crate::surface::title_raw(n).trim();
        let body_shown = if m.capability || m.weight >= 3 {
            if has_body {
                for l in body.lines() {
                    writeln!(s, "        {}", l)?;
                }
            }
            has_body
        } else {
            if has_body {
                for l in truncated_lines(&body, &m.terms) {
                    writeln!(s, "        {}", l)?;
                }
            }
            writeln!(s, "        (q open {} if it appears to bear on your task)", n.front.id)?;
            true
        };
        if body_shown {
            writeln!(s)?;
        }
        placed.insert(n.front.id.clone(), label.to_string());
    }
    if unmatched > 0 {
        writeln!(s, "    ({} more matched nothing here — {})", unmatched, hint)?;
    }
    Ok(())
}

/// The dispatch brief: DERIVED, NEVER HAND-WRITTEN. If it reads wrong, fix
/// the graph and re-render. The hierarchy (dc-casn): the ratified preamble
/// with the floor line, then CONTRACT (the work, the RETURN spec, the
/// write-set, protocol riders), SEMANTICS (read-first bodies, once), THE
/// MAP (the item's own record at full fidelity), YOUR-WRITES (the graph
/// acts the landing owes), then the tiered BACKDROP (dc-xfgz, dc-hjad) —
/// per-species claim shelf, decisions, threads, docs, each framed in the
/// user's ratified register and matched against the work. Every node
/// renders once at its highest earned fidelity; omissions are counted,
/// never silent. Actor rules close, verbatim.
pub fn brief(store: &Store, key: &str) -> Result<String> {
    let all = store.load_all()?;
    let item = store.find(&all, key)?;
    if item.front.ty != "item" {
        anyhow::bail!(
            "{} is not an item — briefs dispatch items",
            crate::surface::atom_ref(&crate::surface::atom(&all, item))
        );
    }
    let first_line = |body: &str| body.lines().next().unwrap_or("").trim().to_string();
    let mut s = String::new();
    writeln!(
        s,
        "══ DISPATCH BRIEF — {} ══",
        crate::surface::atom_line(&crate::surface::atom(&all, item))
    )?;
    writeln!(s, "derived from the graph at render time; if this brief reads wrong, the graph is wrong — fix the graph, re-render.")?;
    // The ratified preamble carries the floor line (dc-83nk, do-g8r4):
    // clean and caveat-free, the brief is the floor, not the whole
    // interface.
    writeln!(s, "\n{}", crate::framings::PREAMBLE)?;
    if !item.body.trim().is_empty() {
        writeln!(s, "\nTHE WORK:")?;
        for l in crate::mention::unpack(&all, item, &item.body).lines() {
            writeln!(s, "  {}", l)?;
        }
    }

    // Render-time behind check, addressed to the DISPATCHER: a brief built
    // on stale stamps dispatches stale context — confront it before the
    // agent has to.
    let behinds: Vec<crate::queries::Behind> = crate::queries::behind(store, &all)
        .into_iter()
        .filter(|b| b.src.id == item.front.id)
        .collect();
    if !behinds.is_empty() {
        writeln!(s, "\nDISPATCHER — BEHIND CHECK ({} stale ref(s) at render time):", behinds.len())?;
        for b in &behinds {
            let target = b
                .to_atom
                .as_ref()
                .map(crate::surface::atom_ref)
                .unwrap_or_else(|| format!("\"{}\" ({})", b.to_title, b.to));
            writeln!(
                s,
                "  ⚠ [sev {}] this item cites {} at {}, now {} ({}) — review the change, then: q affirm {} --to {}",
                b.severity, target, b.at, b.current, b.reason, item.front.id, b.to
            )?;
        }
        writeln!(s, "  Do not hand this off until each is reviewed — the agent inherits what you did not confront.")?;
    }

    // CONTRACT first (dc-casn hierarchy): the work above, then the RETURN
    // spec, the write-set, and any protocol riders — what is owed and
    // where it may land — before any context renders.
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

    let riders = crate::protocol::matching(&all, "brief", None, item.front.kind.as_deref());
    for r in &riders {
        writeln!(s, "\nPROTOCOL — {}:", crate::surface::atom_ref(&crate::surface::atom(&all, r)))?;
        for l in r.body.lines() {
            writeln!(s, "  {}", l)?;
        }
    }

    // Render-once bookkeeping (dc-xfgz): node id → the section that
    // rendered its body. Every later position shows a one-line ref, never
    // a second body — full > truncated > atom > count.
    let mut placed: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    writeln!(s, "\nREAD-FIRST (cited at pinned versions — q open <id> for any neighborhood):")?;
    let mut cited = 0usize;
    for e in &item.front.edges {
        if e.rel != "depends-on" {
            continue;
        }
        if let Some(t) = all.iter().find(|n| n.front.id == e.to) {
            writeln!(s, "  · depends on {}", crate::surface::atom_line(&crate::surface::atom(&all, t)))?;
            for l in crate::mention::unpack(&all, t, &t.body).lines() {
                writeln!(s, "      {}", l)?;
            }
            placed.insert(t.front.id.clone(), "READ-FIRST".to_string());
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
    let evidence: Vec<&crate::model::Node> = all
        .iter()
        .filter(|n| {
            n.front.edges.iter().any(|e| matches!(e.rel.as_str(), "supports") && e.to == item.front.id)
        })
        .collect();
    for ev in evidence {
        let p = ev.front.path.as_deref().map(|p| format!(" · {}", p)).unwrap_or_default();
        writeln!(s, "  · evidence: {}{}", crate::surface::atom_line(&crate::surface::atom(&all, ev)), p)?;
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
            writeln!(
                s,
                "    · {} -[{}]→ this item",
                crate::surface::atom_ref(&crate::surface::atom(&all, m)),
                rel
            )?;
        }
    }

    // THE MAP (dc-xfgz; the brief-map outcome): the item's own record —
    // its file geography with blob stamps, the claims over those files at
    // full fidelity, and the bodies that cite this item. The intrinsic
    // class: full fidelity regardless of match, never backdrop.
    let file_edges: Vec<&crate::model::Edge> = item
        .front
        .edges
        .iter()
        .filter(|e| e.to.starts_with("file:"))
        .collect();
    let item_paths: Vec<String> = file_edges
        .iter()
        .filter_map(|e| e.to.strip_prefix("file:"))
        .map(crate::store::strip_line)
        .collect();
    let map_claims: Vec<&Node> = all
        .iter()
        .filter(|c| {
            c.front.ty == "claim"
                && !c.front.archived
                && matches!(c.front.status.as_str(), "asserted" | "measured" | "ratified")
                && c.front.id != item.front.id
                && c.front.edges.iter().any(|e| {
                    matches!(e.rel.as_str(), "about" | "source")
                        && e.to.strip_prefix("file:").map_or(false, |f| {
                            let p = crate::store::strip_line(f);
                            item_paths.iter().any(|ip| crate::coord::globs_overlap(ip, &p))
                        })
                })
        })
        .collect();
    let map_mentions: Vec<&Node> = crate::mention::mentioned_by(&all, &item.front.id)
        .into_iter()
        .filter(|m| !m.front.archived)
        .collect();
    if !file_edges.is_empty() || !map_mentions.is_empty() {
        writeln!(s, "\nTHE MAP (the item's own record — rendered whole regardless of match):")?;
        if !file_edges.is_empty() {
            writeln!(s, "  where this work lives (the item's file edges, blob-stamped):")?;
            for e in &file_edges {
                let f = e.to.strip_prefix("file:").unwrap_or(&e.to);
                let marker = match &e.at {
                    At::Blob(b) => match store.blob(f) {
                        Ok(nb) if &nb == b => "  ✓".to_string(),
                        Ok(nb) => format!("  ⚠ drifted (at {}, now {})", b, nb),
                        Err(_) => "  ✗ MISSING".to_string(),
                    },
                    At::V(_) => String::new(),
                };
                writeln!(s, "    {:<10} → {}{}", e.rel, e.to, marker)?;
            }
        }
        for c in &map_claims {
            if placed.contains_key(&c.front.id) {
                writeln!(s, "  · claim over these files: {} — body in READ-FIRST above", crate::surface::atom_line(&crate::surface::atom(&all, c)))?;
                continue;
            }
            writeln!(s, "  · claim over these files: {}", crate::surface::atom_line(&crate::surface::atom(&all, c)))?;
            if !c.body.trim().is_empty() && c.body.trim() != crate::surface::title_raw(c).trim() {
                for l in crate::mention::unpack(&all, c, &c.body).lines() {
                    writeln!(s, "      {}", l)?;
                }
            }
            placed.insert(c.front.id.clone(), "THE MAP".to_string());
        }
        if !map_mentions.is_empty() {
            writeln!(s, "  ← mentioned by (bodies citing this item — derived, never stored):")?;
            for m in &map_mentions {
                writeln!(s, "    {}", crate::surface::atom_ref(&crate::surface::atom(&all, m)))?;
            }
        }
    }

    // YOUR-WRITES (dc-casn; the your-writes outcome): the graph acts this
    // landing owes. The framing is ratified (do-g8r4); the expected-acts
    // bracket derives per item from its kind, filling the doc's declared
    // placeholder.
    let species_hint = match item.front.kind.as_deref() {
        Some("slice") => "a `vein` for each mechanism the next builder should stand on, and a `feature` receipt for the capability that now exists",
        Some("spike") => "a `reading` for each value taken toward the question (`measured` only where a standing instrument keeps it true), and your findings registered as a doc",
        Some("bug") | Some("debt") => "a `vein` where the repair extracted a mechanism, and an affirm on every claim your fix re-verified",
        _ => "claims species-explicit — `vein` (mechanism), `feature` (receipt), `measured` (instrumented fact), `reading` (one-off value), `contract` (a symbol's promise)",
    };
    let area_hint = area_ids.first().copied().unwrap_or("<area>");
    writeln!(s, "\nYOUR-WRITES (what this landing owes the graph):")?;
    writeln!(
        s,
        "  {}expected here: {} — each species-explicit and grounded in what keeps it true (q claim \"`name`: what it provides\" --kind <species> --about {} --source file:<path>; edges by q link <from> <rel> <to>). {}",
        crate::framings::YOUR_WRITES_OPEN,
        species_hint,
        area_hint,
        crate::framings::YOUR_WRITES_CLOSE
    )?;

    // THE BACKDROP (dc-xfgz, dc-hjad): the areas' accumulated record —
    // claims per species, then decisions, threads, docs — every section
    // framed in the ratified register and tiered against this work.
    if !area_ids.is_empty() {
        let ctx = crate::queries::shelf_ctx(&all, item);
        let hint = area_ids
            .iter()
            .map(|a| format!("q open {}", a))
            .collect::<Vec<_>>()
            .join(" · ");
        writeln!(s, "\nBACKDROP (the areas' accumulated record, tiered against this work — matched entries lead; the unmatched are counted, never hidden):")?;
        for aid in &area_ids {
            let Some(area) = all.iter().find(|n| n.front.id == *aid) else { continue };
            writeln!(s, "  · area {}", crate::surface::atom_line(&crate::surface::atom(&all, area)))?;
            if !area.body.trim().is_empty() {
                writeln!(s, "      {}", first_line(&crate::mention::unpack(&all, area, &area.body)))?;
            }
        }
        let in_backdrop = |n: &Node| {
            !n.front.archived
                && n.front.id != item.front.id
                && crate::coord::in_purview(n, &area_ids)
        };
        let live_claims: Vec<&Node> = all
            .iter()
            .filter(|n| {
                in_backdrop(n)
                    && n.front.ty == "claim"
                    && matches!(n.front.status.as_str(), "asserted" | "measured" | "ratified")
            })
            .collect();
        let shelf_hint = format!("the full shelf: {}", hint);
        for (kind, label, framing) in crate::framings::SPECIES {
            let group: Vec<&Node> = live_claims
                .iter()
                .filter(|n| n.front.kind.as_deref() == Some(kind))
                .copied()
                .collect();
            backdrop_section(&mut s, &all, &ctx, label, Some(framing), &group, &mut placed, &shelf_hint)?;
        }
        let mut other_kinds: Vec<String> = live_claims
            .iter()
            .filter_map(|n| n.front.kind.clone())
            .filter(|k| !crate::framings::SPECIES.iter().any(|(sk, _, _)| *sk == k.as_str()))
            .collect();
        other_kinds.sort();
        other_kinds.dedup();
        for k in &other_kinds {
            let group: Vec<&Node> = live_claims
                .iter()
                .filter(|n| n.front.kind.as_deref() == Some(k.as_str()))
                .copied()
                .collect();
            let label = format!("CLAIMS·{}", k.to_uppercase());
            backdrop_section(&mut s, &all, &ctx, &label, None, &group, &mut placed, &shelf_hint)?;
        }
        let kindless: Vec<&Node> = live_claims
            .iter()
            .filter(|n| n.front.kind.is_none())
            .copied()
            .collect();
        backdrop_section(&mut s, &all, &ctx, "CLAIMS (no species recorded)", None, &kindless, &mut placed, &shelf_hint)?;
        let record_hint = format!("the full record: {}", hint);
        let decisions: Vec<&Node> = all
            .iter()
            .filter(|n| in_backdrop(n) && n.front.ty == "decision" && n.front.status == "in-force")
            .collect();
        backdrop_section(&mut s, &all, &ctx, "DECISIONS", Some(crate::framings::DECISIONS), &decisions, &mut placed, &record_hint)?;
        let threads: Vec<&Node> = all
            .iter()
            .filter(|n| in_backdrop(n) && n.front.ty == "thread" && matches!(n.front.status.as_str(), "open" | "queued"))
            .collect();
        backdrop_section(&mut s, &all, &ctx, "THREADS", Some(crate::framings::THREADS), &threads, &mut placed, &record_hint)?;
        let docs: Vec<&Node> = all
            .iter()
            .filter(|n| in_backdrop(n) && n.front.ty == "doc" && n.front.status == "registered")
            .collect();
        backdrop_section(&mut s, &all, &ctx, "DOCS", Some(crate::framings::DOCS), &docs, &mut placed, &record_hint)?;
    }

    writeln!(s, "\nACTOR RULES:")?;
    writeln!(s, "  · Build from `vein`: the claims above are load-bearing structure — design from them before proposing new structure.")?;
    writeln!(s, "  · All graph writes go through q verbs; your work logs under QUARRY_ACTOR (auto-injected).")?;
    writeln!(s, "  · C3: never settle or supersede user-provenance nodes. When the work hits a call that is the user's — a fork that would contradict or overturn a user ruling — queue a thread, take the least-committal provisional path consistent with standing rulings, and keep building; flag the provisional call in your report. The thread surfaces the call; it never blocks your arc.")?;
    writeln!(s, "  · Cite what you build on (q link ... / q claim --source ...); harvest is judged from the diff, not the report.")?;
    writeln!(s, "  · Landing belongs to the dispatcher: report against the RETURN spec and stop — never flip status=done, never release the lease. \"Done\" from an agent is a stop signal, not a transition; the dispatcher judges at: q harvest {}", item.front.id)?;
    Ok(s)
}

/// The dispatch-kind wake lead (it-wub5): what a dispatcher owes, one
/// render for both wake surfaces (q session resume and the SessionStart
/// orient). Ready in purview is the feed, in-flight items each carry the
/// q harvest command that judges them, and homework residue closes the
/// tail — unharvested dispatches the in-flight shelf no longer shows, and
/// stale refs awaiting review. Threads are deliberately absent: not a
/// dispatch session's to settle (dc-wngq). Lines carry their own block
/// indentation; call sites prefix the wake indent. Selection follows
/// coord::wake_shape — this renders, it never consults the kind string.
pub fn dispatch_wake(store: &Store, all: &[Node], area_ids: &[&str]) -> Vec<String> {
    let atom_line = |n: &Node| crate::surface::atom_line(&crate::surface::atom(all, n));
    let mut out: Vec<String> = Vec::new();
    let ready: Vec<&Node> = crate::queries::ready(all)
        .into_iter()
        .filter(|n| crate::coord::in_purview(n, area_ids))
        .collect();
    if ready.is_empty() {
        out.push("ready to dispatch: none in purview — q query shaping for what is still forming".into());
    } else {
        out.push(format!("ready to dispatch ({}) — q dispatch <item> --files <globs>:", ready.len()));
        for n in &ready {
            out.push(format!("  {}", atom_line(n)));
        }
    }
    let inflight: Vec<&Node> = all
        .iter()
        .filter(|n| {
            n.front.ty == "item"
                && n.front.status == "in-flight"
                && crate::coord::in_purview(n, area_ids)
        })
        .collect();
    if !inflight.is_empty() {
        out.push("in-flight — each report is owed; judge and land:".into());
        for n in &inflight {
            out.push(format!("  {} — q harvest {}", atom_line(n), n.front.id));
        }
    }
    let mut homework: Vec<String> = Vec::new();
    if let Ok(log) = store.read_log() {
        for n in crate::queries::unharvested_dispatches(all, &log) {
            if crate::coord::in_purview(n, area_ids)
                && !inflight.iter().any(|i| i.front.id == n.front.id)
            {
                homework.push(format!(
                    "unharvested dispatch: {} — the report is owed; q harvest {}",
                    crate::surface::atom_ref(&crate::surface::atom(all, n)),
                    n.front.id
                ));
            }
        }
    }
    let behinds: Vec<crate::queries::Behind> = crate::queries::behind(store, all)
        .into_iter()
        .filter(|b| b.src.area_ids.iter().any(|id| area_ids.contains(&id.as_str())))
        .collect();
    for b in behinds.iter().take(5) {
        let target = b
            .to_atom
            .as_ref()
            .map(crate::surface::atom_ref)
            .unwrap_or_else(|| format!("\"{}\" ({})", b.to_title, b.to));
        homework.push(format!(
            "[sev {}] {} -[{}]→ {} ({})",
            b.severity,
            crate::surface::atom_ref(&b.src),
            b.rel,
            target,
            b.reason
        ));
    }
    if behinds.len() > 5 {
        homework.push(format!("…and {} more stale ref(s) — q query behind", behinds.len() - 5));
    }
    if !homework.is_empty() {
        out.push("homework residue:".into());
        for l in homework {
            out.push(format!("  {}", l));
        }
    }
    out
}

/// The harvest surface: the dispatcher's judgment seat at a dispatch's exit.
/// Observed-vs-leased, the acts stamped under the badge, the RETURN spec to
/// judge against, and the report-registration homework. Prints; never
/// transitions — landing stays the dispatcher's own act.
pub fn harvest(store: &Store, key: &str) -> Result<String> {
    let all = store.load_all()?;
    let item = store.find(&all, key)?;
    if item.front.ty != "item" {
        anyhow::bail!(
            "{} is not an item — harvest judges dispatched items",
            crate::surface::atom_ref(&crate::surface::atom(&all, item))
        );
    }
    let id = &item.front.id;
    let log = store.read_log()?;
    let mut s = String::new();
    writeln!(
        s,
        "══ HARVEST — {} ══",
        crate::surface::atom_line(&crate::surface::atom(&all, item))
    )?;
    writeln!(s, "the agent's \"done\" was a stop signal, never a transition — judge by outcome, land by your own hand.")?;

    let observed = crate::coord::touched_for(store, &format!("item:{}", id));
    let globs = crate::coord::load_leases(store)
        .iter()
        .find(|l| &l.item == id)
        .map(|l| l.globs.clone())
        .or_else(|| crate::coord::dispatch_for_item(store, id).map(|d| d.globs))
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
        crate::surface::title_raw(item), area, id
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
    writeln!(
        s,
        "dispatch trace for {}:",
        crate::surface::atom_ref(&crate::surface::atom(&all, item))
    )?;
    let mut any = false;
    for ev in log
        .iter()
        .filter(|ev| ev.get("dispatch").and_then(|v| v.as_str()) == Some(id.as_str()))
    {
        let node = ev.get("node").and_then(|v| v.as_str()).unwrap_or("?");
        let what = all
            .iter()
            .find(|n| n.front.id == node)
            .map(|n| crate::surface::atom_ref(&crate::surface::atom(&all, n)))
            .unwrap_or_else(|| format!("({})", node));
        let ts = ev.get("ts").and_then(|v| v.as_str()).unwrap_or("");
        let op = ev.get("op").and_then(|v| v.as_str()).unwrap_or("?");
        writeln!(s, "  {} {} {}", ts, op, what)?;
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
    writeln!(
        s,
        "history of {}:",
        crate::surface::atom_ref(&crate::surface::atom(&all, n))
    )?;
    for e in queries::node_log(store, &n.front.id)? {
        writeln!(s, "  {}", event_line(&e))?;
    }
    Ok(s)
}
