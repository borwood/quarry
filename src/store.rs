use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::model::{Node, NODE_TYPES};

const GRAPH_README: &str = "# graph/\n\nThis directory is a quarry work graph. The files under `nodes/` and `log/`\nare the artifact of record; any index or rendered view is derived.\n\nDo not edit these files by hand — all writes go through the `q` verbs, which\nenforce the schema constraints, bump per-node versions, and stamp edges with\nthe version of their target. (Host repos should wire a hook denying freehand\nedits under this directory.)\n\nSuggested .gitignore lines: `graph/.index/` and `graph/view/`.\n";

pub struct Store {
    /// The graph locale: where nodes, the log, and machine-local state live.
    /// Resolved ONLY by `resolve_store` (dc-g5x5).
    pub root: PathBuf,
    /// The repo the acting context WORKS in — the enclosing checkout of cwd.
    /// Under a store pin this may be a worktree fork of `root`: file content
    /// (blob stamps, existence) reads HERE, at the file the agent actually
    /// touched, while every graph write lands at `root`. Without a pin the
    /// two are the same directory.
    pub work_root: PathBuf,
}

/// The store pin's env transport (dc-g5x5): the session hook injects it for
/// badged shells, launcher env wins. Read at this one point — the injection
/// surface consults it through here for its launcher-wins check.
pub fn env_pin() -> Option<String> {
    std::env::var("QUARRY_STORE").ok().filter(|s| !s.trim().is_empty())
}

/// Walk up from `start` looking for a `graph/` directory (like git
/// discovery). The fallback arm of the resolver — private, so no call site
/// can resolve a store around `resolve_store` (the dc-nnf5 lint pattern).
fn discover_from(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("graph").join("nodes").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// The machine-local, user-level pin file: acting identity key → the graph
/// root its badge pins. User-level because hook processes must read it from
/// ANY cwd (a worktree fork has no machine-local graph state to consult —
/// gitignored files are never checked out). QUARRY_HOME overrides the
/// location (tests; multi-profile machines).
fn pins_path() -> Option<PathBuf> {
    let home = std::env::var("QUARRY_HOME")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| std::env::var("USERPROFILE").ok().filter(|s| !s.trim().is_empty()))
        .or_else(|| std::env::var("HOME").ok().filter(|s| !s.trim().is_empty()))?;
    Some(PathBuf::from(home).join(".quarry").join("store-pins.json"))
}

fn load_pins() -> serde_json::Map<String, serde_json::Value> {
    pins_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

fn save_pins(pins: &serde_json::Map<String, serde_json::Value>) {
    let Some(p) = pins_path() else { return };
    if let Some(dir) = p.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if pins.is_empty() {
        let _ = fs::remove_file(&p);
        return;
    }
    if let Ok(s) = serde_json::to_string_pretty(&serde_json::Value::Object(pins.clone())) {
        let _ = fs::write(&p, s + "\n");
    }
}

/// Record the pin: this identity's badged acts land at `root`. Written at
/// q join (the bind is the pin's birth); best-effort — a failed write leaves
/// the env transport and the spawn line's explicit pin standing.
pub fn pin_identity(key: &str, root: &Path, item: &str) {
    let mut pins = load_pins();
    pins.insert(
        key.to_string(),
        serde_json::json!({
            "root": root.display().to_string(),
            "item": item,
            "since": Store::now()
        }),
    );
    save_pins(&pins);
}

/// The first identity key carrying a pin whose root still holds a graph —
/// keys in caller order (most specific first, the acting_key precedence).
/// A vanished root is skipped, never an error: the pin is a pointer, and a
/// dead pointer falls through to discovery.
pub fn pinned_root(keys: &[String]) -> Option<PathBuf> {
    let pins = load_pins();
    for k in keys {
        if let Some(entry) = pins.get(k) {
            if let Some(r) = entry.get("root").and_then(|v| v.as_str()) {
                let root = PathBuf::from(r);
                if root.join("graph").join("nodes").is_dir() {
                    return Some(root);
                }
            }
        }
    }
    None
}

/// Prune every pin the item's badge planted at this store — called when the
/// dispatch clears (harvest, land, steal-takeover), so the pin lives and
/// dies with the badge it serves.
pub fn clear_pins(root: &Path, item: &str) {
    let mut pins = load_pins();
    let root_s = root.display().to_string();
    let before = pins.len();
    pins.retain(|_, v| {
        !(v.get("item").and_then(|x| x.as_str()) == Some(item)
            && v.get("root").and_then(|x| x.as_str()) == Some(root_s.as_str()))
    });
    if pins.len() != before {
        save_pins(&pins);
    }
}

fn pinned(root: PathBuf, work: Option<PathBuf>) -> Store {
    let work_root = work.unwrap_or_else(|| root.clone());
    Store { root, work_root }
}

/// THE resolver (dc-g5x5): the ONE function through which every verb and
/// every hook obtains the graph locale — swapping locale logic (a gitignored
/// graph, a fully out-of-repo graph) touches only this function. Precedence:
///   1. the explicit pin — `q join --store`, stamped into the spawn line at
///      dispatch beside the join token;
///   2. the env pin — QUARRY_STORE; launcher env wins, and the session hook
///      injects it for badged shells;
///   3. the acting identity's recorded pin — caller-supplied keys first
///      (hook input, whose process env the shell injection never reaches),
///      then the hook-injected env identity (agent → chat → session);
///   [future: the configured-location slot resolves HERE when it exists]
///   4. cwd discovery — the walk-up, the fallback arm.
/// `cwd_hint` seeds discovery and the work root (hooks pass the input's cwd;
/// verbs pass None for the process cwd). The work root is the enclosing
/// checkout of cwd whatever arm resolves the root: under a pin it may be a
/// worktree fork, where file content is read and hashed while every graph
/// write lands at the pinned root.
pub fn resolve_store(
    explicit: Option<&str>,
    identity: &[String],
    cwd_hint: Option<&Path>,
) -> Result<Store> {
    let cwd = cwd_hint
        .map(|p| p.to_path_buf())
        .or_else(|| std::env::current_dir().ok());
    let discovered = cwd.as_deref().and_then(discover_from);
    if let Some(p) = explicit {
        let root = PathBuf::from(p);
        if !root.join("graph").join("nodes").is_dir() {
            bail!(
                "--store {} holds no graph (expected {}/graph/nodes) — the spawn line stamps the canonical root at dispatch; check the path against your dispatcher's line.",
                p, p
            );
        }
        return Ok(pinned(root, discovered));
    }
    if let Some(p) = env_pin() {
        let root = PathBuf::from(&p);
        if !root.join("graph").join("nodes").is_dir() {
            bail!(
                "QUARRY_STORE={} holds no graph (expected {}/graph/nodes) — the pin is stamped at dispatch and injected per shell. If the graph moved, re-dispatch (a fresh spawn line carries the new root); to fall back to discovery, unset QUARRY_STORE.",
                p, p
            );
        }
        return Ok(pinned(root, discovered));
    }
    let mut keys: Vec<String> = identity.to_vec();
    if let Some(k) = crate::coord::acting_key(
        crate::coord::current_agent().as_deref(),
        crate::coord::current_chat().as_deref(),
        crate::coord::current_session().as_deref(),
    ) {
        if !keys.contains(&k) {
            keys.push(k);
        }
    }
    if let Some(root) = pinned_root(&keys) {
        return Ok(pinned(root, discovered));
    }
    // [future: configured location — a registered graph locale (config file
    // or per-repo setting) resolves here, between the pin and discovery.]
    match discovered {
        Some(root) => Ok(Store { work_root: root.clone(), root }),
        None => bail!("no graph/ found here or in any parent — run `q init` at the repo root"),
    }
}

/// The dc-g5x5 lint, CI half (tests/store_lint.rs) and wrap half (q wrap):
/// outside store.rs no source line may resolve a store around the resolver —
/// the forbidden tokens are the discovery walk-up and the pin reads. The
/// injection surface composes the env NAME into shell prefixes; only the
/// resolution reads (`var(`) are fenced.
pub fn lint_locale_sources(src_root: &Path) -> Vec<(String, usize)> {
    const FORBIDDEN: [&str; 4] = [
        "Store::discover",
        "discover_from(",
        "var(\"QUARRY_STORE\"",
        "var(\"QUARRY_HOME\"",
    ];
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(src_root) else {
        return out;
    };
    for e in entries.filter_map(|e| e.ok()) {
        let p = e.path();
        if p.extension().map_or(true, |x| x != "rs") {
            continue;
        }
        if p.file_stem().map_or(false, |s| s == "store") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&p) else {
            continue;
        };
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("?").to_string();
        for (i, line) in text.lines().enumerate() {
            if FORBIDDEN.iter().any(|t| line.contains(t)) {
                out.push((name.clone(), i + 1));
            }
        }
    }
    out
}

impl Store {
    /// The verb-side road into the one resolver: no explicit pin, env
    /// identity, process cwd. Every q verb obtains its store here; hooks
    /// call `resolve_store` directly with the identity their input carries.
    pub fn resolve() -> Result<Store> {
        resolve_store(None, &[], None)
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
            work_root: root.to_path_buf(),
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
            // A badged act from an identified context teaches the machine
            // which agent (or chat) acts under the badge — the env override's
            // road into the association map; q join is the constructed road.
            crate::coord::note_acting(self);
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
    /// Hashed at the WORK root (dc-g5x5): under a store pin the file the agent
    /// actually touched lives in its working checkout (a worktree fork), and
    /// the stamp must hash that content — the store-relative path names it in
    /// the graph either way.
    pub fn blob(&self, fileref: &str) -> Result<String> {
        let path = strip_line(fileref);
        let full = self.work_root.join(&path);
        if !full.exists() {
            bail!("file not found: {}", path);
        }
        let out = Command::new("git")
            .current_dir(&self.work_root)
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

/// Store-relative resolution for observation and lease judgment — ONE point
/// (it-bj3b: two hook arms each carried their own strip, and a divergence
/// would silently discard accounting). Both sides normalize separators and
/// case-fold before the strip: Windows tool hands deliver absolute paths in
/// mixed case and mixed separators, and a byte-compare would drop them. The
/// WORK root strips first (dc-g5x5): a worktree fork mirrors the store's
/// layout, so the fork-relative path IS the store-relative path. The strip
/// holds a component boundary (root + '/' + rel — "quarry2" never resolves
/// under "quarry"). None = the path lives outside both roots; under a badge
/// the CALLER must record that outcome, never drop it.
pub fn store_relative(work_root: &str, root: &str, path: &str) -> Option<String> {
    let norm = |s: &str| s.replace('\\', "/").to_lowercase();
    let p = norm(path);
    let strip = |base: &str| -> Option<String> {
        let b = norm(base);
        let b = b.trim_end_matches('/');
        p.strip_prefix(b)
            .and_then(|r| r.strip_prefix('/'))
            .filter(|r| !r.is_empty())
            .map(String::from)
    };
    strip(work_root).or_else(|| strip(root))
}

impl Store {
    /// `store_relative` with this store's own roots in hand.
    pub fn relative(&self, path: &str) -> Option<String> {
        store_relative(
            &self.work_root.to_string_lossy(),
            &self.root.to_string_lossy(),
            path,
        )
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
