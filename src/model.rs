use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const NODE_TYPES: &[(&str, &str)] = &[
    ("area", "ar"),
    ("item", "it"),
    ("thread", "th"),
    ("decision", "dc"),
    ("claim", "cl"),
    ("doc", "do"),
];

pub fn prefix_of(ty: &str) -> Result<&'static str> {
    NODE_TYPES
        .iter()
        .find(|(t, _)| *t == ty)
        .map(|(_, p)| *p)
        .ok_or_else(|| {
            anyhow!(
                "unknown node type '{}' (one of: {})",
                ty,
                NODE_TYPES.iter().map(|(t, _)| *t).collect::<Vec<_>>().join(", ")
            )
        })
}

pub fn default_status(ty: &str) -> &'static str {
    match ty {
        "area" => "active",
        "item" => "sketch",
        "thread" => "open",
        "decision" => "in-force",
        "claim" => "asserted",
        "doc" => "registered",
        _ => "active",
    }
}

pub fn allowed_statuses(ty: &str) -> &'static [&'static str] {
    match ty {
        "area" => &["active", "retired"],
        "item" => &["sketch", "shaped", "ready", "in-flight", "done", "dropped"],
        "thread" => &["open", "queued", "resolved", "parked"],
        "decision" => &["in-force", "superseded"],
        "claim" => &["asserted", "measured", "ratified", "refuted", "superseded"],
        "doc" => &["registered", "superseded"],
        _ => &[],
    }
}

pub const RELS: &[&str] = &[
    "about", "part-of", "depends-on", "settles", "supports", "refutes", "supersedes", "source",
    "builds-on",
];

/// The legal shapes of each rel, in teaching order — the refusal message's
/// authority (C4 teaches at violation time; one line, where it fires).
pub fn legal_shapes(rel: &str) -> &'static str {
    match rel {
        "about" => "anything → area or file",
        "part-of" => "item → item, area → area",
        "depends-on" => "item/thread → item/thread/decision",
        "settles" => "decision → thread",
        "supports" => "claim/doc → decision/item/claim",
        "refutes" => "claim/doc → claim/decision",
        "supersedes" => "same type → same type",
        "source" => "claim → doc or file",
        "builds-on" => "builder → built-upon: decision → decision, doc → claim/decision (lineage, never status)",
        _ => "",
    }
}

/// One legality check for one (src, rel, dst) triple — the matrix's single
/// authority. `validate_edge` refuses through it; `legal_rels` derives the
/// refusal's redirect from it, so the two can never disagree.
fn edge_ok(src_ty: &str, rel: &str, dst_ty: Option<&str>) -> bool {
    match (rel, dst_ty) {
        ("about", None) => true,
        ("about", Some(d)) => d == "area",
        ("part-of", Some(d)) => (src_ty == "item" && d == "item") || (src_ty == "area" && d == "area"),
        ("depends-on", Some(d)) => {
            matches!(src_ty, "item" | "thread") && matches!(d, "item" | "thread" | "decision")
        }
        ("settles", Some(d)) => src_ty == "decision" && d == "thread",
        ("supports", Some(d)) => {
            matches!(src_ty, "claim" | "doc") && matches!(d, "decision" | "item" | "claim")
        }
        ("refutes", Some(d)) => matches!(src_ty, "claim" | "doc") && matches!(d, "claim" | "decision"),
        ("supersedes", Some(d)) => src_ty == d,
        ("source", Some(d)) => src_ty == "claim" && d == "doc",
        ("source", None) => src_ty == "claim",
        // Informational lineage (ruled 2026-08-10): builder → built-upon.
        // No status coupling anywhere — a refuted or superseded target never
        // flips its builders; staleness (behind, reverse blast) is the signal.
        ("builds-on", Some(d)) => {
            (src_ty == "decision" && d == "decision")
                || (src_ty == "doc" && matches!(d, "claim" | "decision"))
        }
        _ => false,
    }
}

/// The legal rels for an exact src → dst pair, in RELS order — the teaching
/// refusal's redirect (it-6349): a link refused for an illegal rel names
/// what IS legal for the very pair the linker holds, turning the dead end
/// into a one-step redirect instead of a silent downgrade to mention-only.
pub fn legal_rels(src_ty: &str, dst_ty: Option<&str>) -> Vec<&'static str> {
    RELS.iter().copied().filter(|r| edge_ok(src_ty, r, dst_ty)).collect()
}

/// The edge-type matrix. `dst_ty: None` means a `file:` target. A refusal
/// teaches twice (C4, it-6349): the attempted rel's legal shapes, then the
/// redirect — the legal rels for this exact pair, the reverse direction
/// when the lean runs the other way, or the mention channel when no rel
/// joins the types at all (mention-only is then correct, not a downgrade).
pub fn validate_edge(src_ty: &str, rel: &str, dst_ty: Option<&str>) -> Result<()> {
    if edge_ok(src_ty, rel, dst_ty) {
        return Ok(());
    }
    let shapes = legal_shapes(rel);
    let dst_name = dst_ty.unwrap_or("file");
    let forward = legal_rels(src_ty, dst_ty);
    let redirect = if !forward.is_empty() {
        format!(" — legal rels for {} → {}: {}", src_ty, dst_name, forward.join(", "))
    } else {
        // A file is never an edge's source, so a file target has no reverse.
        let reverse = dst_ty.map(|d| legal_rels(d, Some(src_ty))).unwrap_or_default();
        if !reverse.is_empty() {
            format!(
                " — no rel points {} → {}; the lean runs the other way: {} -[{}]-> {}",
                src_ty,
                dst_name,
                dst_name,
                reverse.join("|"),
                src_ty
            )
        } else {
            format!(
                " — no rel joins {} → {} in either direction: cite the id in the body instead (a mention references, an edge leans)",
                src_ty, dst_name
            )
        }
    };
    bail!(
        "edge not allowed: {} -[{}]-> {}{}{} — run `q guide` for the edge matrix",
        src_ty,
        rel,
        dst_name,
        if shapes.is_empty() {
            String::new()
        } else {
            format!(" ({} takes {})", rel, shapes)
        },
        redirect
    );
}

/// What an edge was written against: a node version or a file blob (12 hex chars).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum At {
    V(u64),
    Blob(String),
}

impl std::fmt::Display for At {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            At::V(v) => write!(f, "v{}", v),
            At::Blob(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Edge {
    pub rel: String,
    pub to: String,
    pub at: At,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ratified {
    pub by: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Front {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub title: String,
    pub v: u64,
    pub status: String,
    pub provenance: String,
    pub created: String,
    pub actor: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub blob: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ratified: Option<Ratified>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub acceptance: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub write_set: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub aliases: Vec<String>,
    /// Archived: settled and demoted from default surfaces — never gone.
    /// Ids resolve, edges hold, blast/behind always see everything.
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub archived: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub edges: Vec<Edge>,
    /// Project-declared fields (the protocol layer's registry): preserved
    /// verbatim, settable via `q set k=v` / `q new --field k=v`, owned by
    /// the host project's protocol — never by the engine.
    #[serde(flatten, skip_serializing_if = "std::collections::BTreeMap::is_empty", default)]
    pub extra: std::collections::BTreeMap<String, serde_yaml::Value>,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub front: Front,
    pub body: String,
    pub file: PathBuf,
}

impl Node {
    pub fn parse(path: &Path, raw: &str) -> Result<Node> {
        let content = raw.replace("\r\n", "\n");
        let rest = content
            .strip_prefix("---\n")
            .ok_or_else(|| anyhow!("{}: missing frontmatter", path.display()))?;
        let end = rest
            .find("\n---\n")
            .or_else(|| rest.strip_suffix("\n---").map(|s| s.len()))
            .ok_or_else(|| anyhow!("{}: unterminated frontmatter", path.display()))?;
        let yaml = &rest[..end];
        let body = rest
            .get(end + 5..)
            .unwrap_or("")
            .trim_start_matches('\n')
            .trim_end()
            .to_string();
        let front: Front = serde_yaml::from_str(yaml)
            .map_err(|e| anyhow!("{}: bad frontmatter: {}", path.display(), e))?;
        Ok(Node {
            front,
            body,
            file: path.to_path_buf(),
        })
    }

    pub fn serialize(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(&self.front)?;
        let mut s = format!("---\n{}---\n", yaml);
        if !self.body.trim().is_empty() {
            s.push('\n');
            s.push_str(self.body.trim_end());
            s.push('\n');
        }
        Ok(s)
    }

    pub fn slug(&self) -> Option<String> {
        self.file
            .file_stem()?
            .to_str()?
            .strip_prefix(&format!("{}-", self.front.id))
            .map(|s| s.to_string())
    }
}

pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    for c in title.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').chars().take(48).collect::<String>().trim_end_matches('-').to_string()
}
