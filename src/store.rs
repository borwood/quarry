use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::model::{Node, NODE_TYPES};

const GRAPH_README: &str = "# graph/\n\nThis directory is a quarry work graph. The files under `nodes/` and `log/`\nare the artifact of record; any index or rendered view is derived.\n\nDo not edit these files by hand — all writes go through the `q` verbs, which\nenforce the schema constraints, bump per-node versions, and stamp edges with\nthe version of their target. (Host repos should wire a hook denying freehand\nedits under this directory.)\n\nSuggested .gitignore lines: `graph/.index/` and `graph/view/`.\n";

pub struct Store {
    pub root: PathBuf,
}

impl Store {
    /// Walk up from cwd looking for a `graph/` directory (like git discovery).
    pub fn discover() -> Result<Store> {
        let mut dir = std::env::current_dir()?;
        loop {
            if dir.join("graph").join("nodes").is_dir() {
                return Ok(Store { root: dir });
            }
            if !dir.pop() {
                bail!("no graph/ found here or in any parent — run `q init` at the repo root");
            }
        }
    }

    pub fn init(root: &Path) -> Result<Store> {
        let g = root.join("graph");
        for (ty, _) in NODE_TYPES {
            fs::create_dir_all(g.join("nodes").join(ty))?;
        }
        fs::create_dir_all(g.join("log"))?;
        let readme = g.join("GRAPH.md");
        if !readme.exists() {
            fs::write(&readme, GRAPH_README)?;
        }
        Ok(Store {
            root: root.to_path_buf(),
        })
    }

    pub fn nodes_dir(&self) -> PathBuf {
        self.root.join("graph").join("nodes")
    }

    pub fn log_dir(&self) -> PathBuf {
        self.root.join("graph").join("log")
    }

    pub fn load_all(&self) -> Result<Vec<Node>> {
        let mut out = Vec::new();
        for (ty, _) in NODE_TYPES {
            let dir = self.nodes_dir().join(ty);
            if !dir.is_dir() {
                continue;
            }
            let mut entries: Vec<_> = fs::read_dir(&dir)?.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());
            for e in entries {
                let p = e.path();
                if p.extension().map_or(true, |x| x != "md") {
                    continue;
                }
                let content =
                    fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?;
                out.push(Node::parse(&p, &content)?);
            }
        }
        Ok(out)
    }

    /// Resolve a node by exact id, exact slug, or unique case-insensitive title fragment.
    pub fn find<'a>(&self, all: &'a [Node], key: &str) -> Result<&'a Node> {
        if let Some(n) = all.iter().find(|n| n.front.id == key) {
            return Ok(n);
        }
        let by_slug: Vec<&Node> = all
            .iter()
            .filter(|n| n.slug().map_or(false, |s| s == key))
            .collect();
        match by_slug.len() {
            1 => return Ok(by_slug[0]),
            n if n > 1 => bail!("slug '{}' is ambiguous", key),
            _ => {}
        }
        let kl = key.to_lowercase();
        let by_title: Vec<&Node> = all
            .iter()
            .filter(|n| crate::surface::title_raw(n).to_lowercase().contains(&kl))
            .collect();
        match by_title.len() {
            1 => Ok(by_title[0]),
            0 => bail!("no node matches '{}'", key),
            _ => bail!(
                "'{}' is ambiguous: {}",
                key,
                by_title
                    .iter()
                    .map(|n| crate::surface::atom_ref(&crate::surface::atom(all, n)))
                    .collect::<Vec<_>>()
                    .join(" · ")
            ),
        }
    }

    pub fn mint_id(&self, ty: &str, all: &[Node]) -> Result<String> {
        let prefix = crate::model::prefix_of(ty)?;
        const AB: &[u8] = b"23456789abcdefghjkmnpqrstuvwxyz";
        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos() as u64
            | 1;
        for _ in 0..256 {
            let mut chars = String::new();
            for _ in 0..4 {
                seed = seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                chars.push(AB[(seed >> 33) as usize % AB.len()] as char);
            }
            let id = format!("{}-{}", prefix, chars);
            if !all.iter().any(|n| n.front.id == id) {
                return Ok(id);
            }
        }
        bail!("could not mint a unique id");
    }

    pub fn save(&self, node: &Node) -> Result<()> {
        if let Some(dir) = node.file.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&node.file, node.serialize()?)
            .with_context(|| format!("writing {}", node.file.display()))?;
        Ok(())
    }

    pub fn log_event(&self, ev: serde_json::Value) -> Result<()> {
        let dir = self.log_dir();
        fs::create_dir_all(&dir)?;
        let now = OffsetDateTime::now_utc();
        let shard = dir.join(format!("{:04}-{:02}.jsonl", now.year(), u8::from(now.month())));
        let lock = dir.join(".lock");
        let mut waited = 0u64;
        loop {
            match fs::OpenOptions::new().write(true).create_new(true).open(&lock) {
                Ok(_) => break,
                Err(_) if waited < 2000 => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    waited += 50;
                }
                Err(_) => break, // stale lock — proceed rather than wedge
            }
        }
        let mut ev = ev;
        if let Some(obj) = ev.as_object_mut() {
            if !obj.contains_key("session") {
                if let Ok(s) = std::env::var("QUARRY_SESSION") {
                    if !s.trim().is_empty() {
                        obj.insert("session".into(), serde_json::json!(s));
                    }
                }
            }
            // The dispatch badge: every verb stamps it, so "what did this
            // dispatch write" is a query, not a reconstruction. Resolution is
            // per acting chat — another chat's badge never stamps this one.
            if !obj.contains_key("dispatch") {
                if let Some(b) = crate::coord::current_dispatch_badge(self) {
                    obj.insert("dispatch".into(), serde_json::json!(b));
                }
            }
            // A badged act from an identified chat teaches the machine which
            // chat acts under the badge — hook processes (blind to shell env)
            // resolve that chat's file writes through the association.
            crate::coord::note_acting_chat(self);
        }
        if let Some(sess) = ev.get("session").and_then(|v| v.as_str()) {
            crate::coord::touch_session(self, sess);
        }
        let res = (|| -> Result<()> {
            use std::io::Write;
            let mut f = fs::OpenOptions::new().create(true).append(true).open(&shard)?;
            writeln!(f, "{}", serde_json::to_string(&ev)?)?;
            Ok(())
        })();
        let _ = fs::remove_file(&lock);
        res
    }

    pub fn read_log(&self) -> Result<Vec<serde_json::Value>> {
        let mut out = Vec::new();
        let dir = self.log_dir();
        if !dir.is_dir() {
            return Ok(out);
        }
        let mut shards: Vec<_> = fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |x| x == "jsonl"))
            .collect();
        shards.sort();
        for s in shards {
            for line in fs::read_to_string(&s)?.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str(line) {
                    out.push(v);
                }
            }
        }
        Ok(out)
    }

    pub fn now() -> String {
        let t = OffsetDateTime::now_utc();
        let t = t.replace_nanosecond(0).unwrap_or(t);
        t.format(&Rfc3339).unwrap_or_default()
    }

    pub fn today() -> String {
        Self::now().split('T').next().unwrap_or_default().to_string()
    }

    pub fn actor() -> String {
        if let Ok(a) = std::env::var("QUARRY_ACTOR") {
            if !a.trim().is_empty() {
                return a;
            }
        }
        if let Ok(out) = Command::new("git").args(["config", "user.name"]).output() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
        "unknown".into()
    }

    /// Abbreviated git blob hash of a repo-relative path ("path" or "path:line").
    pub fn blob(&self, fileref: &str) -> Result<String> {
        let path = strip_line(fileref);
        let full = self.root.join(&path);
        if !full.exists() {
            bail!("file not found: {}", path);
        }
        let out = Command::new("git")
            .current_dir(&self.root)
            .args(["hash-object", "--"])
            .arg(&path)
            .output()
            .context("running git hash-object")?;
        if !out.status.success() {
            bail!("git hash-object failed for {}", path);
        }
        Ok(String::from_utf8_lossy(&out.stdout)
            .trim()
            .chars()
            .take(12)
            .collect())
    }
}

/// "src/x.rs:42" -> "src/x.rs"; leaves paths without a numeric suffix alone.
pub fn strip_line(fileref: &str) -> String {
    if let Some((path, tail)) = fileref.rsplit_once(':') {
        if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) {
            return path.to_string();
        }
    }
    fileref.to_string()
}
