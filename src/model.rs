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
];

/// The edge-type matrix. `dst_ty: None` means a `file:` target.
pub fn validate_edge(src_ty: &str, rel: &str, dst_ty: Option<&str>) -> Result<()> {
    let ok = match (rel, dst_ty) {
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
        _ => false,
    };
    if !ok {
        bail!(
            "edge not allowed: {} -[{}]-> {} — run `q guide` for the edge matrix",
            src_ty,
            rel,
            dst_ty.unwrap_or("file")
        );
    }
    Ok(())
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
