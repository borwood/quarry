use quarry::model::At;
use quarry::ops::{self, NewArgs};
use quarry::queries;
use quarry::store::Store;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_store() -> Store {
    std::env::set_var("QUARRY_ACTOR", "test-user");
    // The session hook injects these into bound chats; inherited values flip
    // session-keyed behavior (injection no-ops, watermark keys shift).
    std::env::remove_var("QUARRY_SESSION");
    std::env::remove_var("QUARRY_DISPATCH");
    std::env::remove_var("QUARRY_CHAT");
    std::env::remove_var("QUARRY_AGENT");
    std::env::remove_var("QUARRY_STORE");
    // One per-process pin-file home (dc-g5x5): in-process ops::join calls
    // write identity pins, and without this they land in the REAL user
    // file, redirecting the next run's identically-keyed joins to dead
    // temp stores. Stable across threads: every call sets the same value.
    static HOME: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    let home = HOME.get_or_init(|| {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("quarry-test-home-{}-{}", std::process::id(), nanos))
    });
    std::env::set_var("QUARRY_HOME", home);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("quarry-test-{}-{}-{}", std::process::id(), n, nanos));
    std::fs::create_dir_all(&dir).unwrap();
    Store::init(&dir).unwrap()
}

#[test]
fn create_and_reload() {
    let s = temp_store();
    let a = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    assert!(a.front.id.starts_with("ar-"));
    assert_eq!(a.front.v, 1);
    let mut args = NewArgs::bare("item", "S11 spike");
    args.kind = Some("slice".into());
    args.acceptance = vec!["halo measured".into()];
    let i = ops::new_node(&s, args).unwrap();
    assert!(i.front.id.starts_with("it-"));
    assert_eq!(i.front.status, "sketch");

    let all = s.load_all().unwrap();
    assert_eq!(all.len(), 2);
    let found = s.find(&all, "water").unwrap();
    assert_eq!(found.front.id, a.front.id);
    let by_slug = s.find(&all, "s11-spike").unwrap();
    assert_eq!(by_slug.front.id, i.front.id);
}

#[test]
fn stamped_edges_go_behind_and_affirm() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    // user-provenance claim needs no source (C2 exemption)
    let c = ops::claim(
        &s,
        "the halo is bounded", None, None,
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    assert_eq!(c.front.edges.len(), 1);
    assert_eq!(c.front.edges[0].at, At::V(1));

    // bump the area; the claim's edge is now behind
    ops::set(&s, &area.front.id, &["title=hydrology".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let behind = queries::behind(&s, &all);
    assert_eq!(behind.len(), 1);
    assert_eq!(behind[0].src.id, c.front.id);
    assert_eq!(behind[0].severity, 3);

    // affirm restamps
    let n = ops::affirm(&s, &c.front.id, None).unwrap();
    assert_eq!(n, 1);
    let all = s.load_all().unwrap();
    assert!(queries::behind(&s, &all).is_empty());
}

#[test]
fn c3_denies_assistant_settling_user_thread() {
    let s = temp_store();
    let mut t = NewArgs::bare("thread", "ocean: finite or pinned?");
    t.provenance = Some("user".into());
    let t = ops::new_node(&s, t).unwrap();

    let mut d = NewArgs::bare("decision", "pinned");
    d.provenance = Some("assistant".into());
    let d = ops::new_node(&s, d).unwrap();

    let err = ops::link(&s, &d.front.id, "settles", &t.front.id, false, None).unwrap_err();
    assert!(err.to_string().contains("C3"), "got: {}", err);

    // rule with actor test-user derives user provenance and succeeds
    let dec = ops::rule(&s, &t.front.id, "pinned: level set by the world", Some("user".into()), None).unwrap();
    assert_eq!(dec.front.provenance, "user");
    assert!(dec.front.ratified.is_some());
    let all = s.load_all().unwrap();
    let t2 = s.find(&all, &t.front.id).unwrap();
    assert_eq!(t2.front.status, "resolved");
    // the decision inherits nothing here (thread had no about), but settles edge exists
    let dec2 = s.find(&all, &dec.front.id).unwrap();
    assert!(dec2.front.edges.iter().any(|e| e.rel == "settles" && e.to == t.front.id));
}

#[test]
fn refute_blast_and_c5() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let src_doc = ops::new_node(&s, NewArgs::bare("doc", "S11 results")).unwrap();
    let evidence = ops::new_node(&s, NewArgs::bare("doc", "S14 remeasurement")).unwrap();

    let c = ops::claim(
        &s,
        "halo is 4-11 cells", None, None,
        vec![area.front.id.clone()],
        Some(src_doc.front.id.clone()),
        Some("ring differencing".into()),
        None,
        None,
    )
    .unwrap();
    assert_eq!(c.front.provenance, "measured");

    let mut d = NewArgs::bare("decision", "persist bodies, derive voxels");
    d.provenance = Some("user".into());
    let d = ops::new_node(&s, d).unwrap();
    ops::link(&s, &c.front.id, "supports", &d.front.id, false, None).unwrap();

    let mut it = NewArgs::bare("item", "water body graph");
    it.status = Some("ready".into());
    it.acceptance = vec!["bodies persist".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &d.front.id, false, None).unwrap();

    let (c2, blast) = ops::refute(&s, &c.front.id, &evidence.front.id, None).unwrap();
    assert_eq!(c2.front.status, "refuted");
    let ids: Vec<&str> = blast.iter().map(|n| n.front.id.as_str()).collect();
    assert!(ids.contains(&d.front.id.as_str()), "decision in blast: {:?}", ids);
    assert!(ids.contains(&it.front.id.as_str()), "item in blast: {:?}", ids);

    // C5: citing the refuted claim now requires --acknowledge
    let c5 = ops::link(&s, &src_doc.front.id, "supports", &c2.front.id, false, None).unwrap_err();
    assert!(c5.to_string().contains("C5"), "got: {}", c5);
    ops::link(&s, &src_doc.front.id, "supports", &c2.front.id, true, None).unwrap();
}

#[test]
fn ready_respects_thread_blockers() {
    let s = temp_store();
    let mut it = NewArgs::bare("item", "attitude recording");
    it.status = Some("ready".into());
    it.acceptance = vec!["attitudes recorded".into()];
    let it = ops::new_node(&s, it).unwrap();

    let all = s.load_all().unwrap();
    assert_eq!(queries::ready(&all).len(), 1);

    let mut th = NewArgs::bare("thread", "fact or interpretation?");
    th.provenance = Some("user".into());
    th.status = Some("queued".into());
    let th = ops::new_node(&s, th).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &th.front.id, false, None).unwrap();

    let all = s.load_all().unwrap();
    assert!(queries::ready(&all).is_empty(), "item blocked by open thread");
    assert_eq!(queries::queue(&all).len(), 1);

    ops::rule(&s, &th.front.id, "recorded in deeptime", Some("user".into()), None).unwrap();
    let all = s.load_all().unwrap();
    assert_eq!(queries::ready(&all).len(), 1, "ruling unblocks the item");
    assert!(queries::queue(&all).is_empty());
}

#[test]
fn thread_blocked_on_spike_leaves_queue() {
    let s = temp_store();
    let mut th = NewArgs::bare("thread", "which storage layout?");
    th.provenance = Some("user".into());
    th.status = Some("queued".into());
    let th = ops::new_node(&s, th).unwrap();

    let mut spike = NewArgs::bare("item", "spike: measure both layouts");
    spike.status = Some("in-flight".into());
    let spike = ops::new_node(&s, spike).unwrap();
    ops::link(&s, &th.front.id, "depends-on", &spike.front.id, false, None).unwrap();

    let all = s.load_all().unwrap();
    assert!(queries::queue(&all).is_empty(), "thread waits on its spike");

    ops::set(&s, &spike.front.id, &["status=done".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    assert_eq!(queries::queue(&all).len(), 1, "spike landed; thread answerable");
}

#[test]
fn hook_guard_denies_graph_writes_only() {
    let windows_path = "{\"tool_name\":\"Edit\",\"tool_input\":{\"file_path\":\"B:\\\\repos\\\\x\\\\graph\\\\nodes\\\\claim\\\\cl-1.md\"}}";
    let deny = quarry::teach::guard(windows_path);
    assert!(deny.is_some());
    assert!(deny.unwrap().contains("C6"));
    let log = quarry::teach::guard(
        r#"{"tool_name":"Write","tool_input":{"file_path":"B:/repos/x/graph/log/2026-08.jsonl"}}"#,
    );
    assert!(log.is_some());
    let src = quarry::teach::guard(
        r#"{"tool_name":"Edit","tool_input":{"file_path":"B:/repos/x/src/main.rs"}}"#,
    );
    assert!(src.is_none());
    let read = quarry::teach::guard(
        r#"{"tool_name":"Read","tool_input":{"file_path":"B:/repos/x/graph/nodes/a.md"}}"#,
    );
    assert!(read.is_none());
    let garbage = quarry::teach::guard("not json");
    assert!(garbage.is_none());
}

#[test]
fn view_renders_every_node() {
    let s = temp_store();
    let a = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let mut th = NewArgs::bare("thread", "finite or pinned?");
    th.provenance = Some("user".into());
    th.about = vec![a.front.id.clone()];
    let th = ops::new_node(&s, th).unwrap();
    let html = quarry::view::render(&s).unwrap();
    assert!(html.contains(&a.front.id));
    assert!(html.contains(&th.front.id));
    assert!(html.contains("finite or pinned?"));
    assert!(html.contains("__QUARRY_DATA__") == false);
}

#[test]
fn affirm_only_restamps_does_not_bump() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let c = ops::claim(
        &s,
        "halo bounded", None, None,
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    ops::set(&s, &area.front.id, &["title=hydrology".to_string()], None).unwrap();
    let v_before = c.front.v;
    let n = ops::affirm(&s, &c.front.id, None).unwrap();
    assert_eq!(n, 1);
    let all = s.load_all().unwrap();
    let c2 = s.find(&all, &c.front.id).unwrap();
    assert_eq!(c2.front.v, v_before, "affirm-only restamp must not bump v");
    assert!(queries::behind(&s, &all).is_empty());
}

#[test]
fn homework_helpers() {
    let s = temp_store();
    let mut th = NewArgs::bare("thread", "which layout?");
    th.provenance = Some("user".into());
    th.status = Some("queued".into());
    let th = ops::new_node(&s, th).unwrap();
    let mut it = NewArgs::bare("item", "build the layout");
    it.status = Some("ready".into());
    it.acceptance = vec!["the layout stands".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &th.front.id, false, None).unwrap();

    // bumping the thread puts the item's stamp behind
    ops::set(&s, &th.front.id, &["title=which storage layout?".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let behind = queries::citers_behind(&all, &th.front.id);
    assert_eq!(behind.len(), 1);
    assert_eq!(behind[0].0.front.id, it.front.id);
    // thread not resolved: nothing unblocked yet
    assert!(queries::unblocked_by(&all, &th.front.id).is_empty());

    ops::rule(&s, &th.front.id, "flat files", Some("user".into()), None).unwrap();
    let all = s.load_all().unwrap();
    let un = queries::unblocked_by(&all, &th.front.id);
    assert_eq!(un.len(), 1);
    assert_eq!(un[0].front.id, it.front.id);
}

#[test]
fn homework_rederives_on_demand_the_act_print_is_only_a_delivery() {
    // The it-8tcy resolution: homework is DERIVED from current graph state,
    // never stored — render::homework is the one derivation point, the
    // act-time print delivers it, and `q query homework <node>` re-derives
    // it, so a pipe that dies mid-print loses nothing but the delivery.
    let s = temp_store();
    let mut th = NewArgs::bare("thread", "which layout?");
    th.provenance = Some("user".into());
    th.status = Some("queued".into());
    let th = ops::new_node(&s, th).unwrap();
    let mut it = NewArgs::bare("item", "build the layout");
    it.status = Some("ready".into());
    it.acceptance = vec!["the layout stands".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &th.front.id, false, None).unwrap();

    // bumping the thread: the derivation carries the behind line with the
    // affirm command that clears it after review
    ops::set(&s, &th.front.id, &["title=which storage layout?".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let hw = quarry::render::homework(&all, &[th.front.id.as_str()]);
    assert_eq!(hw.len(), 1, "one behind line: {:?}", hw);
    assert!(
        hw[0].contains("behind:")
            && hw[0].contains(&format!("q affirm {} --to {}", it.front.id, th.front.id)),
        "the behind line carries the affirm command: {:?}",
        hw
    );

    // resolving the thread: the same derivation now carries the unblock line
    ops::rule(&s, &th.front.id, "flat files", Some("user".into()), None).unwrap();
    let all = s.load_all().unwrap();
    let hw = quarry::render::homework(&all, &[th.front.id.as_str()]);
    assert!(
        hw.iter().any(|l| l.contains("dispatchable")),
        "resolution derives the unblock line: {:?}",
        hw
    );

    // a node with no homework derives empty — the query's honest silence
    let quiet = ops::new_node(&s, NewArgs::bare("area", "quiet town")).unwrap();
    let all = s.load_all().unwrap();
    assert!(
        quarry::render::homework(&all, &[quiet.front.id.as_str()]).is_empty(),
        "no citers, nothing unblocked: empty derivation"
    );
}

#[test]
fn doc_markdown_content_embeds_in_view() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::write(s.root.join("NOTES.md"), "# Heading One

| a | b |
|---|---|
| 1 | 2 |
").unwrap();
    let mut d = NewArgs::bare("doc", "notes");
    d.path = Some("NOTES.md".into());
    ops::new_node(&s, d).unwrap();
    let html = quarry::view::render(&s).unwrap();
    assert!(html.contains("Heading One"), "md content embedded");
    assert!(html.contains("doc_content"));
}

#[test]
fn glob_overlap_heuristic() {
    use quarry::coord::globs_overlap;
    assert!(globs_overlap("crates/**", "crates/dc-sim/**"));
    assert!(globs_overlap("crates/dc-sim/**", "crates/**"));
    assert!(!globs_overlap("crates/dc-worldgen/**", "crates/dc-sim/**"));
    assert!(globs_overlap("docs/design/*.md", "docs/**"));
    assert!(globs_overlap("*.rs", "src/main.rs"), "bare wildcard is conservative");
    assert!(!globs_overlap("docs/a/**", "docs/b/**"));
}

#[test]
fn leases_exclusive_shared_steal() {
    let s = temp_store();
    let a = ops::new_node(&s, NewArgs::bare("item", "geo pass work")).unwrap();
    let b = ops::new_node(&s, NewArgs::bare("item", "bodies gait work")).unwrap();
    let c = ops::new_node(&s, NewArgs::bare("item", "sdk docs from geo")).unwrap();
    let d = ops::new_node(&s, NewArgs::bare("item", "sdk docs from bodies")).unwrap();

    quarry::coord::reserve(&s, &a, "geo", "t", vec!["crates/dc-worldgen/**".into()], false, false, None).unwrap();
    // disjoint: fine
    quarry::coord::reserve(&s, &b, "bodies", "t", vec!["crates/dc-sim/body/**".into()], false, false, None).unwrap();
    // overlapping exclusive from another session: denied, names holder
    let mut e = NewArgs::bare("item", "bodies wants worldgen");
    e.status = Some("sketch".into());
    let e = ops::new_node(&s, e).unwrap();
    let err = quarry::coord::reserve(&s, &e, "bodies", "t", vec!["crates/dc-worldgen/deep/**".into()], false, false, None)
        .unwrap_err();
    assert!(err.to_string().contains("C7"), "got: {}", err);
    assert!(err.to_string().contains("geo"));
    // shared + shared coexist with visibility
    quarry::coord::reserve(&s, &c, "geo", "t", vec!["docs/sdk/**".into()], true, false, None).unwrap();
    let out = quarry::coord::reserve(&s, &d, "bodies", "t", vec!["docs/sdk/**".into()], true, false, None).unwrap();
    assert_eq!(out.co_holders.len(), 1);
    assert_eq!(out.co_holders[0].session, "geo");
    // steal is allowed and reported
    let out = quarry::coord::reserve(&s, &e, "bodies", "t", vec!["crates/dc-worldgen/deep/**".into()], false, true, Some("test steal")).unwrap();
    assert_eq!(out.stolen.len(), 1);
    assert_eq!(out.stolen[0].session, "geo");
    // release: own lease only
    let err = quarry::coord::release(&s, &b, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("bodies"));
    quarry::coord::release(&s, &b, "bodies", "t").unwrap();
}

#[test]
fn purview_scoping() {
    let s = temp_store();
    let geo = ops::new_node(&s, NewArgs::bare("area", "worldgen passes")).unwrap();
    let bod = ops::new_node(&s, NewArgs::bare("area", "bodies")).unwrap();
    let mut i1 = NewArgs::bare("item", "erosion pass");
    i1.about = vec![geo.front.id.clone()];
    i1.status = Some("ready".into());
    i1.acceptance = vec!["the pass lands".into()];
    ops::new_node(&s, i1).unwrap();
    let mut i2 = NewArgs::bare("item", "gait clip");
    i2.about = vec![bod.front.id.clone()];
    i2.status = Some("ready".into());
    i2.acceptance = vec!["the clip lands".into()];
    ops::new_node(&s, i2).unwrap();

    quarry::coord::save_session(&s, "geo", vec![geo.front.id.clone()], None, None, false).unwrap();
    let reg = quarry::coord::load_sessions(&s);
    assert!(reg.contains_key("geo"));
    let all = s.load_all().unwrap();
    let ids: Vec<&str> = reg["geo"].areas.iter().map(|x| x.as_str()).collect();
    let mine: Vec<_> = queries::ready(&all)
        .into_iter()
        .filter(|n| quarry::coord::in_purview(n, &ids))
        .collect();
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0].front.title, "erosion pass");
}

#[test]
fn c2_grounding_forms() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::write(s.root.join("evidence.rs"), "fn observed() {}
").unwrap();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();

    // free-floating assistant claim: denied
    let err = ops::claim(&s, "vibes", None, None, vec![area.front.id.clone()], None, None, Some("assistant".into()), None)
        .unwrap_err();
    assert!(err.to_string().contains("C2"), "got: {}", err);
    // method alone grounds it
    ops::claim(&s, "measured thing", None, None, vec![area.front.id.clone()], None,
        Some("log inspection".into()), None, None).unwrap();
    // a file: source grounds it (matrix widened), blob-stamped
    let c = ops::claim(&s, "read off the code", None, None, vec![area.front.id.clone()],
        Some("file:evidence.rs".into()), None, Some("assistant".into()), None).unwrap();
    assert!(c.front.edges.iter().any(|e| e.rel == "source" && e.to == "file:evidence.rs"));
}

#[test]
fn protocol_gate_memoizes_and_resumes() {
    let s = temp_store();
    // a protocol entry: gate journal-kind doc creation
    let mut p = NewArgs::bare("doc", "journal charter");
    p.kind = Some("protocol".into());
    p.fields = vec!["on=new".into(), "node_type=doc".into(), "node_kind=journal".into(), "tier=gate".into()];
    p.body = "write narrative, not changelog".into();
    ops::new_node(&s, p).unwrap();
    let all = s.load_all().unwrap();

    let args = serde_json::json!({"any": "intent"});
    let g = quarry::protocol::gate_if_needed(&s, &all, "new", Some("doc"), Some("journal"), args.clone())
        .unwrap()
        .expect("first attempt gates");
    assert_eq!(g.rules.len(), 1);
    assert!(g.rules[0].1.contains("narrative"));
    // memoized: second attempt in same session does not gate
    let again = quarry::protocol::gate_if_needed(&s, &all, "new", Some("doc"), Some("journal"), args.clone()).unwrap();
    assert!(again.is_none(), "session memo clears the gate");
    // non-matching kind never gates
    let other = quarry::protocol::gate_if_needed(&s, &all, "new", Some("doc"), Some("spike"), args).unwrap();
    assert!(other.is_none());
    // the token resumes exactly once
    let intent = quarry::protocol::take_intent(&s, &g.token).unwrap();
    assert_eq!(intent.verb, "new");
    assert!(quarry::protocol::take_intent(&s, &g.token).is_err(), "single use");
}

#[test]
fn extra_fields_roundtrip() {
    let s = temp_store();
    let mut d = NewArgs::bare("doc", "charter");
    d.kind = Some("protocol".into());
    d.fields = vec!["on=new".into(), "tier=inline".into()];
    let d = ops::new_node(&s, d).unwrap();
    ops::set(&s, &d.front.id, &["lenses=ai-native".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let d2 = s.find(&all, &d.front.id).unwrap();
    assert_eq!(d2.front.extra.get("on").and_then(|v| v.as_str()), Some("new"));
    assert_eq!(d2.front.extra.get("lenses").and_then(|v| v.as_str()), Some("ai-native"));
}

#[test]
fn session_heartbeat_roundtrip() {
    let s = temp_store();
    assert!(quarry::coord::last_seen(&s, "geo").is_none());
    quarry::coord::touch_session(&s, "geo");
    let ts = quarry::coord::last_seen(&s, "geo").expect("touched");
    assert!(ts.contains('T'));
    assert!(quarry::coord::last_seen(&s, "bodies").is_none());
}

#[test]
fn charter_at_wake_line() {
    // it-sumw: one render, both wake surfaces — q session resume and the
    // SessionStart orient print this same text beneath the purview line,
    // so a session meets its own kind at wake (dc-ydvb consumed).
    let s = temp_store();
    quarry::coord::save_session(
        &s,
        "geo",
        vec![],
        None,
        Some("decisions session: rulings, design, the thread queue".into()),
        false,
    )
    .unwrap();
    let reg = quarry::coord::load_sessions(&s);
    assert_eq!(
        quarry::coord::charter_line(&reg["geo"]).as_deref(),
        Some("charter: decisions session: rulings, design, the thread queue")
    );
    // a session without a charter wakes exactly as before: no line at all
    quarry::coord::save_session(&s, "bare", vec![], None, None, false).unwrap();
    let reg = quarry::coord::load_sessions(&s);
    assert!(quarry::coord::charter_line(&reg["bare"]).is_none());
}

#[test]
fn session_kind_is_registry_data() {
    // it-skpa / dc-ad8b: kind parses and validates in ONE place
    // (coord::parse_kind); surfaces render the kind they find; kindless
    // entries stay legal and render as today.
    assert_eq!(quarry::coord::parse_kind("design").unwrap(), "design");
    assert_eq!(quarry::coord::parse_kind(" Dispatch ").unwrap(), "dispatch");
    let err = quarry::coord::parse_kind("audit").unwrap_err().to_string();
    assert!(err.contains("design") && err.contains("dispatch"), "the refusal names the known kinds: {}", err);

    let s = temp_store();
    quarry::coord::save_session(
        &s,
        "geo",
        vec![],
        Some("design".into()),
        Some("rulings and the thread queue".into()),
        false,
    )
    .unwrap();
    let reg = quarry::coord::load_sessions(&s);
    assert_eq!(quarry::coord::kind_line(&reg["geo"]).as_deref(), Some("kind: design"));
    assert_eq!(reg["geo"].kind.as_deref(), Some("design"));

    // a kindless session renders exactly as before: no kind line at all
    quarry::coord::save_session(&s, "bare", vec![], None, None, false).unwrap();
    let reg = quarry::coord::load_sessions(&s);
    assert!(quarry::coord::kind_line(&reg["bare"]).is_none());

    // a pre-field sessions.json entry (no kind key on disk) stays legal
    let path = s.root.join("graph").join("sessions.json");
    std::fs::write(&path, r#"{"legacy": {"areas": []}}"#).unwrap();
    let reg = quarry::coord::load_sessions(&s);
    assert!(reg["legacy"].kind.is_none());
    assert!(quarry::coord::kind_line(&reg["legacy"]).is_none());
}

#[test]
fn wake_shape_follows_kind_at_one_match_point() {
    // it-wub5 / dc-ad8b: the kind string maps to a wake shape in exactly one
    // place (coord::wake_shape); surfaces render the shape they are handed.
    // Dispatch leads with the dispatcher's owes and drops the owed-threads
    // block (dc-wngq: threads are not a dispatch session's to settle).
    let d = quarry::coord::wake_shape(Some("dispatch"));
    assert!(d.dispatcher_lead);
    assert!(!d.owed_threads);
    // design, kindless, and UNKNOWN kinds all keep the generic brief — the
    // set is open, and the catch-all is the openness.
    for k in [Some("design"), None, Some("audit")] {
        let g = quarry::coord::wake_shape(k);
        assert!(!g.dispatcher_lead, "generic shape for {:?}", k);
        assert!(g.owed_threads, "owed threads render for {:?}", k);
    }
}

#[test]
fn dispatch_wake_leads_with_ready_inflight_and_homework() {
    // it-wub5: one render, both wake surfaces — ready in purview, in-flight
    // each with its q harvest command, homework residue; threads absent.
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let aid = area.front.id.clone();
    let mut r = NewArgs::bare("item", "ready pass");
    r.status = Some("ready".into());
    r.about = vec![aid.clone()];
    r.acceptance = vec!["the pass lands".into()];
    let r = ops::new_node(&s, r).unwrap();
    let mut f = NewArgs::bare("item", "flying pass");
    f.status = Some("ready".into());
    f.about = vec![aid.clone()];
    f.acceptance = vec!["the flight lands".into()];
    let f = ops::new_node(&s, f).unwrap();
    ops::dispatch(&s, &f.front.id, vec!["src/geo/**".into()], false, false, None, None, "geo", "t").unwrap();
    let mut th = NewArgs::bare("thread", "which datum wins?");
    th.status = Some("queued".into());
    th.about = vec![aid.clone()];
    th.provenance = Some("user".into());
    let th = ops::new_node(&s, th).unwrap();
    // homework residue: a claim citing the area at v1, then the area bumps
    let c = ops::claim(
        &s,
        "strata are layered", None, None,
        vec![aid.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    ops::set(&s, &aid, &["title=stratigraphy".to_string()], None).unwrap();

    let all = s.load_all().unwrap();
    let out = quarry::render::dispatch_wake(&s, &all, &[aid.as_str()]);
    let text = out.join("\n");
    // leads with ready in purview
    assert!(out[0].starts_with("ready to dispatch (1)"), "ready leads the wake: {:?}", out);
    assert!(text.contains(&r.front.id), "the ready item is enumerated: {}", text);
    // in-flight rides its harvest command
    assert!(
        out.iter().any(|l| l.contains(&f.front.id) && l.contains(&format!("q harvest {}", f.front.id))),
        "the in-flight item carries its q harvest command: {}",
        text
    );
    // a live-dispatched in-flight item is NOT doubled as unharvested residue
    assert!(!text.contains("unharvested dispatch"), "in-flight shelf already carries it: {}", text);
    // homework residue names the stale ref
    assert!(text.contains("homework residue:"), "{}", text);
    assert!(
        out.iter().any(|l| l.contains(&c.front.id) && l.contains("[sev")),
        "the behind ref surfaces as homework: {}",
        text
    );
    // threads are not a dispatch session's to settle (dc-wngq): no owed
    // block renders, and the thread surfaces only as ref hygiene (its own
    // stale about-edge is the dispatcher's homework, not its question)
    assert!(!text.contains("owed to the user"), "no owed block in a dispatch wake: {}", text);
    assert!(
        out.iter().filter(|l| l.contains(&th.front.id)).all(|l| l.contains("[sev")),
        "the thread appears only as a stale-ref source: {}",
        text
    );

    // a dispatch whose item left in-flight without a harvest is residue
    ops::set(&s, &f.front.id, &["status=ready".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let out = quarry::render::dispatch_wake(&s, &all, &[aid.as_str()]);
    assert!(
        out.iter().any(|l| l.contains("unharvested dispatch") && l.contains(&f.front.id)),
        "the report stays owed after the status moved: {:?}",
        out
    );

    // a purview with nothing owed says so once about ready, and the plea
    // channel closes every dispatch wake (dc-mpg8) — a standing teach,
    // never a pressure count
    let quiet = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let out = quarry::render::dispatch_wake(&s, &all, &[quiet.front.id.as_str()]);
    assert_eq!(out.len(), 2, "quiet purview: ready line plus the plea channel: {:?}", out);
    assert!(out[0].starts_with("ready to dispatch: none in purview"));
    assert!(
        out[1].contains("the plea channel (dc-mpg8)") && out[1].contains("q new thread"),
        "the dispatch wake names the plea channel: {:?}",
        out
    );
}

#[test]
fn session_injection_binds_and_rewrites() {
    let s = temp_store();
    quarry::coord::write_adopt_request(&s, "geo").unwrap();
    let input = r#"{"session_id":"chat-abc","tool_name":"Bash","tool_input":{"command":"q wrap","description":"lint"}}"#;
    let out = quarry::teach::session_hook_output(&s, input).expect("injects after adopt");
    let cmd = out["hookSpecificOutput"]["updatedInput"]["command"].as_str().unwrap();
    assert_eq!(cmd, "export QUARRY_SESSION='geo'; export QUARRY_CHAT='chat-abc'; q wrap");
    assert_eq!(out["hookSpecificOutput"]["updatedInput"]["description"].as_str().unwrap(), "lint");
    // binding persisted: no pending request, still injects; PowerShell prefix
    let input2 = r#"{"session_id":"chat-abc","tool_name":"PowerShell","tool_input":{"command":"q view"}}"#;
    let out2 = quarry::teach::session_hook_output(&s, input2).expect("bound");
    let cmd2 = out2["hookSpecificOutput"]["updatedInput"]["command"].as_str().unwrap();
    assert_eq!(cmd2, "$env:QUARRY_SESSION='geo'; $env:QUARRY_CHAT='chat-abc'; q view");
    // unbound chat: no session, but the chat identity still injects — per-chat
    // machine-local state (the dispatch badge) resolves by it in q processes
    let input3 = r#"{"session_id":"chat-other","tool_name":"Bash","tool_input":{"command":"ls"}}"#;
    let out3 = quarry::teach::session_hook_output(&s, input3).expect("chat identity injects unbound");
    let cmd3 = out3["hookSpecificOutput"]["updatedInput"]["command"].as_str().unwrap();
    assert_eq!(cmd3, "export QUARRY_CHAT='chat-other'; ls");
    // non-shell tools: no-op even when bound
    let input4 = r#"{"session_id":"chat-abc","tool_name":"Write","tool_input":{"file_path":"x"}}"#;
    assert!(quarry::teach::session_hook_output(&s, input4).is_none());
    // subagent shape (dc-zbxj): the harness names an agent id — QUARRY_AGENT
    // injects alongside the rest. The subagent's session_id is the PARENT
    // chat's; the agent id is its only distinguishing mark.
    let input5 = r#"{"session_id":"chat-abc","agent_id":"ag-42","tool_name":"Bash","tool_input":{"command":"q log x"}}"#;
    let out5 = quarry::teach::session_hook_output(&s, input5).expect("agent id injects");
    let cmd5 = out5["hookSpecificOutput"]["updatedInput"]["command"].as_str().unwrap();
    assert_eq!(
        cmd5,
        "export QUARRY_SESSION='geo'; export QUARRY_CHAT='chat-abc'; export QUARRY_AGENT='ag-42'; q log x"
    );
}

#[test]
fn hook_agent_id_defensive_shapes() {
    // The harness fact (dc-zbxj): hooks run in a subagent receive an agent
    // id field. The exact key name is read defensively across plausible
    // spellings; absence, emptiness, and non-strings resolve nothing.
    use quarry::teach::hook_agent_id;
    let cases: &[(&str, Option<&str>)] = &[
        (r#"{"agent_id":"ag-1"}"#, Some("ag-1")),
        (r#"{"agentId":"ag-2"}"#, Some("ag-2")),
        (r#"{"agent_session_id":"ag-3"}"#, Some("ag-3")),
        (r#"{"agentSessionId":"ag-4"}"#, Some("ag-4")),
        (r#"{"subagent_id":"ag-5"}"#, Some("ag-5")),
        (r#"{"session_id":"chat-1"}"#, None),
        (r#"{"agent_id":""}"#, None),
        (r#"{"agent_id":42}"#, None),
    ];
    for (input, expect) in cases {
        let v: serde_json::Value = serde_json::from_str(input).unwrap();
        assert_eq!(hook_agent_id(&v).as_deref(), *expect, "input: {}", input);
    }
}

#[test]
fn purview_overlap_detection() {
    let s = temp_store();
    let a = ops::new_node(&s, NewArgs::bare("area", "worldgen")).unwrap();
    let b = ops::new_node(&s, NewArgs::bare("area", "materials sdk")).unwrap();
    let c = ops::new_node(&s, NewArgs::bare("area", "bodies")).unwrap();
    quarry::coord::save_session(&s, "geo", vec![a.front.id.clone(), b.front.id.clone()], None, None, false).unwrap();
    let overlaps = quarry::coord::purview_overlaps(&s, &[b.front.id.clone(), c.front.id.clone()]);
    assert_eq!(overlaps.len(), 1);
    assert_eq!(overlaps[0].0, "geo");
    assert_eq!(overlaps[0].1, vec![b.front.id.clone()]);
    let none = quarry::coord::purview_overlaps(&s, &[c.front.id.clone()]);
    assert!(none.is_empty());
}

#[test]
fn alert_computation_closed_list() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "materials sdk")).unwrap();
    quarry::coord::save_session(&s, "geo", vec![area.front.id.clone()], None, None, false).unwrap();
    // an arrival filed into geo's purview by bodies (stays sketch), plus a
    // dependency that bodies lands, unblocking geo's item
    let mut arrival = NewArgs::bare("item", "density convention");
    arrival.about = vec![area.front.id.clone()];
    let arrival = ops::new_node(&s, arrival).unwrap();
    let dep = ops::new_node(&s, NewArgs::bare("item", "materials api shape")).unwrap();
    let mut mine = NewArgs::bare("item", "gait bake");
    mine.about = vec![area.front.id.clone()];
    mine.status = Some("ready".into());
    mine.acceptance = vec!["the bake lands".into()];
    let mine = ops::new_node(&s, mine).unwrap();
    ops::link(&s, &mine.front.id, "depends-on", &dep.front.id, false, None).unwrap();
    // the landing actually happens (event stream and node state agree in production)
    ops::set(&s, &dep.front.id, &["status=done".to_string()], None).unwrap();

    let all = s.load_all().unwrap();
    let ids: Vec<&str> = vec![area.front.id.as_str()];
    let log = vec![
        serde_json::json!({"ts":"2000-01-01T00:00:00Z","op":"create","node":arrival.front.id,"session":"bodies"}),
        serde_json::json!({"ts":"2099-01-01T00:00:01Z","op":"create","node":arrival.front.id,"session":"bodies","title":"density convention"}),
        serde_json::json!({"ts":"2099-01-01T00:00:02Z","op":"steal","node":"it-zzzz","session":"bodies","from_session":"geo","from_item":mine.front.id,"reason":"urgent hotfix"}),
        serde_json::json!({"ts":"2099-01-01T00:00:03Z","op":"set","node":dep.front.id,"session":"bodies","fields":["status=done"]}),
    ];
    // cursor is a log INDEX: position 1 skips the pre-cursor event exactly
    let lines = quarry::teach::alerts_between(&all, &log, "geo", &ids, 1);
    assert_eq!(lines.len(), 3, "got: {:?}", lines);
    assert!(lines[0].contains("new from bodies"));
    assert!(lines[1].contains("your lease") && lines[1].contains("urgent hotfix"));
    assert!(lines[2].contains("unblocked") && lines[2].contains("gait bake"));
    // own-session events never alert
    let own = quarry::teach::alerts_between(&all, &log, "bodies", &ids, 1);
    assert!(own.iter().all(|l| !l.contains("new from bodies")), "got: {:?}", own);
    // legacy timestamp cursors convert by counting events at-or-before
    let legacy = quarry::coord::Cursor::Ts("2098-12-31T00:00:00Z".into());
    assert_eq!(quarry::coord::cursor_index(&legacy, &log), 1);
}

#[test]
fn ephemeral_flag_roundtrip() {
    let s = temp_store();
    let a = ops::new_node(&s, NewArgs::bare("area", "scratch zone")).unwrap();
    quarry::coord::save_session(&s, "audit", vec![a.front.id.clone()], None, None, true).unwrap();
    quarry::coord::save_session(&s, "geo2", vec![a.front.id.clone()], None, None, false).unwrap();
    let reg = quarry::coord::load_sessions(&s);
    assert!(reg["audit"].ephemeral);
    assert!(!reg["geo2"].ephemeral);
}

#[test]
fn archive_rules_and_hierarchy() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let doc = ops::new_node(&s, NewArgs::bare("doc", "S11 results")).unwrap();
    // areas and docs never archive
    assert!(ops::archive(&s, &area.front.id, false).is_err());
    assert!(ops::archive(&s, &doc.front.id, false).is_err());
    // live status refuses
    let mut live = NewArgs::bare("item", "in flight work");
    live.status = Some("in-flight".into());
    let live = ops::new_node(&s, live).unwrap();
    let err = ops::archive(&s, &live.front.id, false).unwrap_err();
    assert!(err.to_string().contains("settled"), "got: {}", err);
    // parent with live child refuses; child then parent succeeds
    let mut parent = NewArgs::bare("item", "the arc");
    parent.status = Some("done".into());
    let parent = ops::new_node(&s, parent).unwrap();
    let mut child = NewArgs::bare("item", "the slice");
    child.status = Some("done".into());
    let child = ops::new_node(&s, child).unwrap();
    ops::link(&s, &child.front.id, "part-of", &parent.front.id, false, None).unwrap();
    let err = ops::archive(&s, &parent.front.id, false).unwrap_err();
    assert!(err.to_string().contains("live child"), "got: {}", err);
    let all = s.load_all().unwrap();
    let v_before = s.find(&all, &child.front.id).unwrap().front.v;
    let c2 = ops::archive(&s, &child.front.id, false).unwrap();
    assert!(c2.front.archived);
    assert_eq!(c2.front.v, v_before, "archiving must not bump v");
    ops::archive(&s, &parent.front.id, false).unwrap();
    // undo restores
    let c3 = ops::archive(&s, &child.front.id, true).unwrap();
    assert!(!c3.front.archived);
}

#[test]
fn session_retire_removes_registry_and_leases() {
    let s = temp_store();
    let a = ops::new_node(&s, NewArgs::bare("area", "scratch")).unwrap();
    quarry::coord::save_session(&s, "audit", vec![a.front.id.clone()], None, None, true).unwrap();
    let mut it = NewArgs::bare("item", "audit probe");
    it.status = Some("in-flight".into());
    let it = ops::new_node(&s, it).unwrap();
    quarry::coord::reserve(&s, &it, "audit", "t", vec!["docs/audit/**".into()], false, false, None).unwrap();
    assert_eq!(quarry::coord::load_leases(&s).len(), 1);
    quarry::coord::retire_session(&s, "audit", "t").unwrap();
    assert!(!quarry::coord::load_sessions(&s).contains_key("audit"));
    assert!(quarry::coord::load_leases(&s).is_empty());
    assert!(quarry::coord::retire_session(&s, "audit", "t").is_err(), "double retire errors");
}

#[test]
fn vein_presence_check() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::write(s.root.join("resolve.rs"), "fn vein() {}\n").unwrap();
    let area = ops::new_node(&s, NewArgs::bare("area", "materials")).unwrap();
    let all = s.load_all().unwrap();
    assert!(!queries::files_cited(&all, &["resolve.rs".to_string()]), "nothing cites yet");
    ops::claim(
        &s,
        "materials resolve per-voxel through layered override stacks", None, None,
        vec![area.front.id.clone()],
        Some("file:resolve.rs".into()),
        None,
        Some("assistant".into()),
        None,
    )
    .unwrap();
    let all = s.load_all().unwrap();
    assert!(queries::files_cited(&all, &["resolve.rs".to_string()]), "the vein claim cites it");
    assert!(queries::files_cited(&all, &["resolve.rs:12".to_string().replace(":12", "")]),);
    assert!(!queries::files_cited(&all, &["src/**".to_string()]), "unrelated globs stay uncited");
}

#[test]
fn topic_queue_roundtrip_and_prune() {
    let s = temp_store();
    let mut a = NewArgs::bare("thread", "naming register");
    a.provenance = Some("user".into());
    a.status = Some("queued".into());
    let a = ops::new_node(&s, a).unwrap();
    let mut b = NewArgs::bare("thread", "journal ambition");
    b.provenance = Some("user".into());
    b.status = Some("queued".into());
    let b = ops::new_node(&s, b).unwrap();
    quarry::coord::save_topic_queue(&s, &[a.front.id.clone(), b.front.id.clone()]).unwrap();
    let all = s.load_all().unwrap();
    let (q, pruned) = quarry::coord::topic_queue_pruned(&s, &all);
    assert_eq!(q.len(), 2);
    assert!(pruned.is_empty());
    // resolving a thread prunes it from the topic queue
    ops::rule(&s, &a.front.id, "settled", Some("user".into()), None).unwrap();
    let all = s.load_all().unwrap();
    let (q, pruned) = quarry::coord::topic_queue_pruned(&s, &all);
    assert_eq!(q, vec![b.front.id.clone()]);
    assert_eq!(pruned, vec![a.front.id.clone()]);
    // and the prune persisted
    assert_eq!(quarry::coord::load_topic_queue(&s), vec![b.front.id.clone()]);
}

#[test]
fn area_watermark_lifecycle() {
    use quarry::coord::{record_area_read, touch_area, AreaTouch};
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let aid = area.front.id.clone();
    let all = s.load_all().unwrap();
    // first touch: no cursor recorded for this session
    assert!(matches!(touch_area(&s, &all, "geo", &aid), AreaTouch::FirstTouch));
    record_area_read(&s, "geo", &aid);
    assert!(matches!(touch_area(&s, &all, "geo", &aid), AreaTouch::Current));
    // a foreign (unbound) write lands in the area — SAME SECOND as the
    // cursor: the log-index cursor catches what a timestamp cursor missed
    let mut it = NewArgs::bare("item", "erosion pass");
    it.about = vec![aid.clone()];
    ops::new_node(&s, it).unwrap();
    let all = s.load_all().unwrap();
    match touch_area(&s, &all, "geo", &aid) {
        AreaTouch::Drift(lines) => {
            assert!(
                lines.iter().any(|l| l.contains("erosion pass") && l.contains("unbound")),
                "got {:?}",
                lines
            );
        }
        AreaTouch::FirstTouch => panic!("cursor was recorded"),
        AreaTouch::Current => panic!("foreign drift must surface"),
    }
    // delivery advanced the cursor: quiet again, said once
    assert!(matches!(touch_area(&s, &all, "geo", &aid), AreaTouch::Current));
}

#[test]
fn attention_rides_the_actor_watermarks_and_deliveries_are_badge_scoped() {
    // it-csm3 under dc-pwyd: attention state keys on the ACTING identity.
    // A joined agent's reads spend badge-scoped watermarks and deliveries;
    // the holding session's rows survive its arcs untouched.
    use quarry::coord::{
        badge_attention_key, clear_dispatch, has_area_read, record_area_read, touch_area,
        AreaTouch,
    };
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let aid = area.front.id.clone();
    let mut it = NewArgs::bare("item", "geo pass");
    it.status = Some("ready".into());
    it.about = vec![aid.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    let out =
        ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "design", "t")
            .unwrap();
    // the holding session read the area with its own eyes before dispatching
    record_area_read(&s, "design", &aid);
    ops::join(&s, &out.token, Some("agent:ag-att".into())).unwrap();
    let badge = badge_attention_key(&it.front.id);
    // join-as-delivery: the brief rendered the item's areas' record, so the
    // BADGE holds the read — recorded badge-scoped, not on the session
    assert!(has_area_read(&s, &badge, &aid), "the join delivered the area to the badge");
    // a foreign session's write lands in the area
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": aid, "v": 1, "op": "set", "session": "bodies"
    }))
    .unwrap();
    let all = s.load_all().unwrap();
    // the joined agent's read consumes ITS OWN delivery...
    match touch_area(&s, &all, &badge, &aid) {
        AreaTouch::Drift(lines) => assert!(
            lines.iter().any(|l| l.contains("session bodies")),
            "foreign drift delivers to the badge: {:?}",
            lines
        ),
        _ => panic!("foreign drift must deliver to the badge"),
    }
    assert!(matches!(touch_area(&s, &all, &badge, &aid), AreaTouch::Current));
    // ...and the holding session's delivery SURVIVES the agent's consumption
    // (the it-hjed incident: a delivery spent on the agent's screen was owed
    // to a chat that never saw it)
    match touch_area(&s, &all, "design", &aid) {
        AreaTouch::Drift(lines) => assert!(
            lines.iter().any(|l| l.contains("session bodies")),
            "got {:?}",
            lines
        ),
        _ => panic!("the session's own delivery survives the agent's arc"),
    }
    // the agent's badged act — session env inherited, dispatch stamped —
    // is FOREIGN to the holding session's eyes, attributed to the dispatch
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": aid, "v": 1, "op": "set",
        "session": "design", "dispatch": it.front.id
    }))
    .unwrap();
    match touch_area(&s, &all, "design", &aid) {
        AreaTouch::Drift(lines) => assert!(
            lines.iter().any(|l| l.contains(&format!("dispatch {}", it.front.id))),
            "a badged act delivers to the session under the dispatch's name: {:?}",
            lines
        ),
        _ => panic!("a badged act is foreign to the holding session"),
    }
    // ...and OWN to the badge: silent cursor advance, no self-drift
    assert!(matches!(touch_area(&s, &all, &badge, &aid), AreaTouch::Current));
    // the badge's attention dies with the dispatch; the session's stands
    clear_dispatch(&s, &it.front.id);
    assert!(!has_area_read(&s, &badge, &aid), "badge attention cleared with the badge");
    assert!(has_area_read(&s, "design", &aid), "the session's rows outlive the arc");
}

#[test]
fn joined_agent_spares_the_holding_sessions_watermarks_end_to_end() {
    // The CLI chain of it-csm3: a joined agent's verbs pass the area gate on
    // the join's own delivery, the holding session's first write into the
    // area still gates (its eyes never read), and the agent's badged writes
    // come back to the session as drift named by the dispatch.
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let aid = area.front.id.clone();
    let mut it = NewArgs::bare("item", "geo pass");
    it.status = Some("ready".into());
    it.about = vec![aid.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    let out =
        ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "design", "t")
            .unwrap();
    // the agent joins from a shell wearing the holding session's env plus
    // its own agent id — the real dispatched shape
    let agent = [("QUARRY_SESSION", "design"), ("QUARRY_AGENT", "ag-cli")];
    let j = run(&agent, &["join", &out.token]);
    assert!(j.status.success(), "join: {}", String::from_utf8_lossy(&j.stderr));
    // the agent's first mint into the item's area does NOT gate: the join
    // delivered the area record badge-scoped
    let t = run(&agent, &["new", "thread", "agent finding", "--about", &aid]);
    assert!(t.status.success(), "agent mint: {}", String::from_utf8_lossy(&t.stderr));
    assert!(
        !String::from_utf8_lossy(&t.stdout).contains("gated"),
        "the join's delivery covers the badge: {}",
        String::from_utf8_lossy(&t.stdout)
    );
    // the holding session's watermark survived the agent's whole arc: its
    // OWN first write into the area still gates
    let g = run(&[("QUARRY_SESSION", "design")], &["new", "thread", "design question", "--about", &aid]);
    assert_eq!(
        g.status.code(),
        Some(2),
        "the session's own first write gates: {}",
        String::from_utf8_lossy(&g.stdout)
    );
    assert!(
        String::from_utf8_lossy(&g.stdout).contains("gated"),
        "got: {}",
        String::from_utf8_lossy(&g.stdout)
    );
    // the gate delivered the session's read; the agent writes again; the
    // session's next mint sees the badged act as drift, named truthfully
    let t2 = run(&agent, &["new", "thread", "second finding", "--about", &aid]);
    assert!(t2.status.success(), "agent mint 2: {}", String::from_utf8_lossy(&t2.stderr));
    let m = run(&[("QUARRY_SESSION", "design")], &["new", "thread", "design question", "--about", &aid]);
    assert!(m.status.success(), "session mint: {}", String::from_utf8_lossy(&m.stderr));
    let out_s = String::from_utf8_lossy(&m.stdout);
    assert!(out_s.contains("since your last read"), "drift delivers to the session: {}", out_s);
    assert!(
        out_s.contains(&format!("dispatch {}", it.front.id)),
        "badged drift names the dispatch, not the inherited session: {}",
        out_s
    );
}

#[test]
fn wrap_session_touched_review() {
    use quarry::queries::session_touched;
    use serde_json::json;
    let log = vec![
        json!({"ts":"t1","op":"create","node":"it-a","session":"geo"}),
        json!({"ts":"t2","op":"wrap","node":"session:geo","session":"geo"}),
        json!({"ts":"t3","op":"set","node":"it-b","session":"geo"}),
        json!({"ts":"t4","op":"link","node":"it-b","session":"geo"}),
        json!({"ts":"t5","op":"create","node":"it-c","session":"bodies"}),
        json!({"ts":"t6","op":"create","node":"it-d"}),
    ];
    let geo = session_touched(&log, Some("geo"));
    assert_eq!(
        geo,
        vec![("it-b".to_string(), "link".to_string())],
        "own events since the last own wrap, latest op wins"
    );
    let unbound = session_touched(&log, None);
    assert_eq!(unbound, vec![("it-d".to_string(), "create".to_string())]);
    let bodies = session_touched(&log, Some("bodies"));
    assert_eq!(bodies.len(), 1, "no wrap cursor yet: everything shows");
}

#[test]
fn brief_renders_neighborhood_and_return_spec() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut d = NewArgs::bare("decision", "bodies persist");
    d.provenance = Some("user".into());
    d.about = vec![area.front.id.clone()];
    let d = ops::new_node(&s, d).unwrap();
    let mut it = NewArgs::bare("item", "water body graph");
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["bodies persist across reload".into()];
    it.body = "build the graph".into();
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &d.front.id, false, None).unwrap();
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(text.contains("DISPATCH BRIEF"));
    assert!(text.contains("bodies persist across reload"));
    assert!(text.contains("depends on \"bodies persist\" — decision [in-force]"));
    assert!(text.contains("leaseless"), "no lease yet: research dispatch");
    assert!(text.contains("hydrology"));
    assert!(quarry::render::brief(&s, &area.front.id).is_err(), "briefs dispatch items only");
}

#[test]
fn c8_briefed_gate_and_lease_check() {
    let s = temp_store();
    let it = ops::new_node(&s, NewArgs::bare("item", "geo pass")).unwrap();
    assert!(!quarry::coord::briefed_this_session(&s, &it.front.id, "geo"));
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": it.front.id, "v": 1, "op": "brief", "session": "geo"
    }))
    .unwrap();
    assert!(quarry::coord::briefed_this_session(&s, &it.front.id, "geo"));
    assert!(!quarry::coord::briefed_this_session(&s, &it.front.id, "bodies"));

    use quarry::teach::{lease_check, LeaseCheck};
    let leases = vec![quarry::coord::Lease {
        item: it.front.id.clone(),
        item_title: "geo pass".into(),
        session: "geo".into(),
        actor: "t".into(),
        globs: vec!["src/geo/**".into()],
        shared: false,
        since: "now".into(),
    }];
    // SOLO lease (no live dispatch): today's behavior holds throughout
    let no_disp: Vec<String> = vec![];
    assert!(matches!(lease_check(&leases, Some("bodies"), None, &no_disp, "src/geo/pass.rs"), LeaseCheck::Deny(_)), "foreign exclusive zone denies");
    assert!(matches!(lease_check(&leases, None, None, &no_disp, "src/geo/pass.rs"), LeaseCheck::Deny(_)), "unbound writes into leased zones deny");
    assert!(matches!(lease_check(&leases, Some("geo"), None, &no_disp, "src/geo/pass.rs"), LeaseCheck::Allow), "a solo holder works its own zone");
    assert!(matches!(lease_check(&leases, Some("geo"), None, &no_disp, "src/other.rs"), LeaseCheck::Warn(_)), "scope creep warns the solo holder");
    assert!(matches!(lease_check(&leases, Some("geo"), Some(it.front.id.as_str()), &no_disp, "src/other.rs"), LeaseCheck::Deny(_)), "badged write outside the write-set denies");
    assert!(matches!(lease_check(&leases, Some("bodies"), None, &no_disp, "graph/sessions.json"), LeaseCheck::Allow));
    assert!(matches!(lease_check(&[], None, None, &no_disp, "src/x.rs"), LeaseCheck::Allow));
    // Out-of-repo paths are never judged: no scope creep, no deny — for
    // holders, foreigners, the unbound, and even a badge.
    assert!(matches!(
        lease_check(&leases, Some("geo"), None, &no_disp, "c:/users/x/appdata/local/temp/scratchpad/notes.md"),
        LeaseCheck::Allow
    ));
    assert!(matches!(lease_check(&leases, Some("bodies"), None, &no_disp, "c:/tmp/elsewhere/src/geo/pass.rs"), LeaseCheck::Allow));
    assert!(matches!(lease_check(&leases, None, None, &no_disp, "/tmp/notes.md"), LeaseCheck::Allow));
    assert!(matches!(
        lease_check(&leases, Some("geo"), Some(it.front.id.as_str()), &no_disp, "/tmp/outside.rs"),
        LeaseCheck::Allow
    ));
    // THE JOIN GATE (dc-zbxj): once a LIVE DISPATCH holds the lease, the
    // zone belongs to the JOINED agent. The holder session's Allow flips to
    // a teaching deny (the dispatcher's chores live outside its dispatched
    // zone), and the unbound context is taught q join — the C8 move applied
    // to the hand-off.
    let dispatched = vec![it.front.id.clone()];
    match lease_check(&leases, Some("geo"), None, &dispatched, "src/geo/pass.rs") {
        LeaseCheck::Deny(msg) => {
            assert!(msg.contains("q join"), "teaches the join: {}", msg);
            assert!(msg.contains(it.front.id.as_str()), "names the item: {}", msg);
        }
        _ => panic!("the holder session's write into its dispatched zone denies (semantic flip)"),
    }
    match lease_check(&leases, None, None, &dispatched, "src/geo/pass.rs") {
        LeaseCheck::Deny(msg) => assert!(msg.contains("q join"), "the unjoined context is taught the join: {}", msg),
        _ => panic!("unjoined writes into a dispatched zone deny"),
    }
    // a genuinely foreign session still reads C7 — the holder is its answer
    match lease_check(&leases, Some("bodies"), None, &dispatched, "src/geo/pass.rs") {
        LeaseCheck::Deny(msg) => assert!(msg.contains("C7"), "foreign sessions keep C7: {}", msg),
        _ => panic!("the foreign exclusive deny stands"),
    }
    // the resolved badge IS the join's product: inside allows with no
    // session identity at all; outside keeps the contract deny
    assert!(
        matches!(lease_check(&leases, None, Some(it.front.id.as_str()), &dispatched, "src/geo/pass.rs"), LeaseCheck::Allow),
        "a joined agent needs no session of its own"
    );
    assert!(matches!(
        lease_check(&leases, None, Some(it.front.id.as_str()), &dispatched, "src/other.rs"),
        LeaseCheck::Deny(_)
    ));
    // the dispatcher's chores OUTSIDE the dispatched zone are ordinary
    // leaseless observation (dc-cc76) — no scope-creep warn rides a lease
    // that belongs to the dispatched agent
    assert!(matches!(lease_check(&leases, Some("geo"), None, &dispatched, "src/elsewhere.rs"), LeaseCheck::Allow));
}

#[test]
fn relatedness_forward_and_reverse() {
    let s = temp_store();
    // forward: a sketch item exists; a new decision's body names its concept —
    // the it-37q5 incident, mechanized
    let zombie = ops::new_node(&s, NewArgs::bare("item", "q handoff and q wrap")).unwrap();
    let mut d = NewArgs::bare("decision", "handoffs are derived at wake");
    d.body = "the planned q handoff verb is retired; resume renders the wake brief".into();
    d.provenance = Some("user".into());
    let d = ops::new_node(&s, d).unwrap();
    let all = s.load_all().unwrap();
    let dn = s.find(&all, &d.front.id).unwrap();
    let rel = queries::relatedness(&all, dn);
    assert!(
        rel.iter().any(|(n, _)| n.front.id == zombie.front.id),
        "the obsoleted sketch surfaces at mint: {:?}",
        rel.iter().map(|(n, _)| &n.front.title).collect::<Vec<_>>()
    );

    // reverse: an old body mentions a concept that only now earns a node
    let mut old = NewArgs::bare("doc", "worldgen notes");
    old.body = "the deepcell pipeline feeds refinement through core-sample stages".into();
    let old = ops::new_node(&s, old).unwrap();
    let core = ops::new_node(&s, NewArgs::bare("item", "core-sample pipeline")).unwrap();
    let all = s.load_all().unwrap();
    let cn = s.find(&all, &core.front.id).unwrap();
    let rel = queries::relatedness(&all, cn);
    assert!(
        rel.iter().any(|(n, _)| n.front.id == old.front.id),
        "prior mentions surface when the concept earns nodehood: {:?}",
        rel.iter().map(|(n, _)| &n.front.title).collect::<Vec<_>>()
    );
    // never self, never boundary-crossing token embeddings
    assert!(rel.iter().all(|(n, _)| n.front.id != cn.front.id));

    // silence default: an unrelated node surfaces nothing
    let quiet = ops::new_node(&s, NewArgs::bare("thread", "unrelated topic entirely")).unwrap();
    let all = s.load_all().unwrap();
    let qn = s.find(&all, &quiet.front.id).unwrap();
    assert!(queries::relatedness(&all, qn).is_empty(), "silence is the default");
}

#[test]
fn dispatch_state_and_touched_accrual() {
    let s = temp_store();
    let d = quarry::coord::DispatchState {
        item: "it-test".into(),
        item_title: "the work".into(),
        session: "geo".into(),
        holder: "chat:chat-a".into(),
        globs: vec!["src/**".into()],
        acceptance: vec!["it lands".into()],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
        token: None,
        joined: None,
        model: None,
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    assert_eq!(quarry::coord::dispatch_for_item(&s, "it-test").unwrap().holder, "chat:chat-a");
    assert_eq!(quarry::coord::held_dispatches(&s, "chat:chat-a").len(), 1);
    assert!(quarry::coord::held_dispatches(&s, "chat:chat-b").is_empty());
    // FLIP (dc-qyr5, was: one held entry per chat): MULTI-HELD — the same
    // chat holds a second live dispatch alongside the first
    let mut dm = d.clone();
    dm.item = "it-more".into();
    quarry::coord::save_dispatch(&s, &dm).unwrap();
    let held = quarry::coord::held_dispatches(&s, "chat:chat-a");
    assert_eq!(held.len(), 2, "a chat holds many (dc-qyr5)");
    // per-item clear stays exact under multi-held: the sibling survives
    quarry::coord::clear_dispatch(&s, "it-more");
    assert_eq!(quarry::coord::held_dispatches(&s, "chat:chat-a").len(), 1);
    assert_eq!(quarry::coord::dispatch_for_item(&s, "it-test").unwrap().item, "it-test");
    // WORK-ONLY STAMPING (dc-zbxj — the semantic flip from the per-chat
    // design): a HELD entry resolves NO stamping badge for anyone, holder
    // included. Stamping reads env and the association map only; held
    // entries are refusal and boundary material.
    assert!(
        quarry::coord::badge_for(&s, None, Some("chat-a"), None).is_none(),
        "the holding chat's own acts stamp nothing"
    );
    assert!(quarry::coord::badge_for(&s, None, Some("chat-b"), None).is_none());
    assert!(quarry::coord::badge_for(&s, None, None, None).is_none());
    assert!(
        quarry::coord::current_dispatch_badge(&s).is_none(),
        "no identity, no badge — and the machine-global fallback stays gone"
    );
    // a second chat dispatches in parallel: entries keyed apart (by item,
    // holder riding each); the holding session resolves nothing either
    // (work-only-stamping)
    let mut d2 = d.clone();
    d2.item = "it-two".into();
    d2.session = "docs".into();
    d2.holder = "session:docs".into();
    quarry::coord::save_dispatch(&s, &d2).unwrap();
    assert!(quarry::coord::badge_for(&s, None, None, Some("docs")).is_none());
    assert_eq!(quarry::coord::dispatch_for_item(&s, "it-two").unwrap().holder, "session:docs");
    // acting associations tie identities to a LIVE badge only — agent-keyed
    // and chat-keyed alike, the agent key winning (a subagent's chat id may
    // be the parent's)
    quarry::coord::record_acting(&s, "agent:ag-1", "it-test");
    quarry::coord::record_acting_chat(&s, "chat-agent", "it-two");
    assert_eq!(quarry::coord::badge_for(&s, Some("ag-1"), None, None).as_deref(), Some("it-test"));
    assert_eq!(quarry::coord::badge_for(&s, None, Some("chat-agent"), None).as_deref(), Some("it-two"));
    assert_eq!(
        quarry::coord::badge_for(&s, Some("ag-1"), Some("chat-agent"), None).as_deref(),
        Some("it-test"),
        "the agent key outranks the chat key"
    );
    quarry::coord::record_acting_chat(&s, "chat-x", "it-gone");
    assert!(quarry::coord::badge_for(&s, None, Some("chat-x"), None).is_none(), "a dead badge never records");
    // clearing an item releases its held entry AND its acting associations,
    // leaving the other chat's dispatch alone
    quarry::coord::clear_dispatch(&s, "it-other");
    assert!(quarry::coord::dispatch_for_item(&s, "it-test").is_some());
    quarry::coord::clear_dispatch(&s, "it-test");
    assert!(quarry::coord::dispatch_for_item(&s, "it-test").is_none());
    assert!(quarry::coord::badge_for(&s, Some("ag-1"), None, None).is_none());
    assert_eq!(quarry::coord::badge_for(&s, None, Some("chat-agent"), None).as_deref(), Some("it-two"));
    quarry::coord::clear_dispatch(&s, "it-two");
    assert!(quarry::coord::load_dispatches(&s).held.is_empty());
    // accrual: distinct per key, ordered, isolated, clearable
    assert_eq!(quarry::coord::touch_key(Some("it-x"), Some("geo")), "item:it-x");
    assert_eq!(quarry::coord::touch_key(None, Some("geo")), "session:geo");
    assert_eq!(quarry::coord::touch_key(None, None), "session:unbound");
    quarry::coord::accrue_touch(&s, "session:geo", "src/a.rs");
    quarry::coord::accrue_touch(&s, "session:geo", "src/b.rs");
    quarry::coord::accrue_touch(&s, "item:it-x", "src/c.rs");
    assert_eq!(quarry::coord::touched_for(&s, "session:geo"), vec!["src/a.rs", "src/b.rs"]);
    assert_eq!(quarry::coord::touched_for(&s, "item:it-x"), vec!["src/c.rs"]);
    quarry::coord::clear_touched(&s, "session:geo");
    assert!(quarry::coord::touched_for(&s, "session:geo").is_empty());
    assert_eq!(quarry::coord::touched_for(&s, "item:it-x"), vec!["src/c.rs"], "clear is key-scoped");
}

#[test]
fn legacy_single_slot_dispatch_state_migrates() {
    let s = temp_store();
    // A pre-per-chat .dispatch.json: one bare DispatchState at top level —
    // possibly live mid-upgrade. It must not crash, and must keep resolving.
    let legacy = serde_json::json!({
        "item": "it-old", "item_title": "pre-per-chat arc", "session": "geo",
        "globs": ["src/**"], "acceptance": ["lands"],
        "since": "2026-01-01T00:00:00Z", "cursor": 3, "checked": "2026-01-01T00:00:00Z"
    });
    std::fs::write(s.root.join("graph").join(".dispatch.json"), legacy.to_string()).unwrap();
    let m = quarry::coord::load_dispatches(&s);
    // FLIP (dc-qyr5, was: keyed under session): held entries key by ITEM,
    // the old key living on as the holder field
    let d = m.held.get("it-old").expect("legacy slot migrates under its item key");
    assert_eq!(d.item, "it-old");
    assert_eq!(d.holder, "session:geo", "the session key became the holder");
    assert_eq!(d.cursor, 3);
    // a pre-token entry parses with no token and no joined — nothing to
    // join, but harvest, trace, and the guard's contract echo all still work
    assert!(d.token.is_none() && d.joined.is_none());
    // work-only-stamping applies to legacy entries too: the holding session
    // resolves no stamping badge (semantic flip from the held-entry design);
    // a foreign chat never did
    assert!(quarry::coord::badge_for(&s, None, None, Some("geo")).is_none(), "held entries never stamp (dc-zbxj)");
    assert!(quarry::coord::badge_for(&s, None, Some("chat-b"), None).is_none());
    assert_eq!(quarry::coord::dispatch_for_item(&s, "it-old").unwrap().holder, "session:geo");
    // joining a live legacy dispatch teaches: it has no token to match
    let err = ops::join(&s, "nosuchtok42", Some("agent:ag-l".into())).unwrap_err();
    assert!(err.to_string().contains("unknown join token"), "got: {}", err);
    // a save persists the new shape without clobbering the migrated entry
    let mut d2 = d.clone();
    d2.item = "it-new".into();
    d2.holder = "chat:chat-b".into();
    quarry::coord::save_dispatch(&s, &d2).unwrap();
    let m = quarry::coord::load_dispatches(&s);
    assert_eq!(m.held.len(), 2);
    assert_eq!(m.held.get("it-old").unwrap().holder, "session:geo");
    // clear releases the migrated entry like any other
    quarry::coord::clear_dispatch(&s, "it-old");
    assert!(quarry::coord::load_dispatches(&s).held.get("it-old").is_none());
    assert_eq!(quarry::coord::load_dispatches(&s).held.len(), 1);
}

#[test]
fn per_chat_keyed_dispatch_state_migrates() {
    let s = temp_store();
    // The intermediate shape (per-chat badge, pre-multi-held): held keyed by
    // dispatching chat, no holder field — possibly live mid-upgrade. The key
    // must move into the holder field and the entry re-key by item.
    let v2 = serde_json::json!({
        "held": {
            "chat:chat-a": {
                "item": "it-mid", "item_title": "mid-upgrade arc", "session": "geo",
                "globs": ["src/**"], "acceptance": ["lands"],
                "since": "2026-01-01T00:00:00Z", "cursor": 5,
                "checked": "2026-01-01T00:00:00Z", "token": "tok0ldkey1", "joined": "agent:ag-m"
            },
            "session:docs": {
                "item": "it-par", "item_title": "parallel arc", "session": "docs",
                "globs": ["docs/**"], "acceptance": [],
                "since": "2026-01-01T00:00:00Z", "cursor": 0,
                "checked": "2026-01-01T00:00:00Z"
            }
        },
        "acting": { "agent:ag-m": "it-mid" }
    });
    std::fs::write(s.root.join("graph").join(".dispatch.json"), v2.to_string()).unwrap();
    let m = quarry::coord::load_dispatches(&s);
    assert_eq!(m.held.len(), 2);
    let d = m.held.get("it-mid").expect("chat-keyed entry re-keys by item");
    assert_eq!(d.holder, "chat:chat-a", "the chat key became the holder");
    assert_eq!(d.token.as_deref(), Some("tok0ldkey1"), "token survives the re-key");
    assert_eq!(m.held.get("it-par").unwrap().holder, "session:docs");
    // the joined agent's association still resolves through the re-key
    assert_eq!(quarry::coord::badge_for(&s, Some("ag-m"), None, None).as_deref(), Some("it-mid"));
    // per-item clear is exact on migrated entries
    quarry::coord::clear_dispatch(&s, "it-mid");
    assert!(quarry::coord::dispatch_for_item(&s, "it-mid").is_none());
    assert_eq!(quarry::coord::dispatch_for_item(&s, "it-par").unwrap().holder, "session:docs");
}

#[test]
fn log_events_stamp_the_badge_from_state() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let d = quarry::coord::DispatchState {
        item: "it-bdg".into(),
        item_title: "badged".into(),
        session: "geo".into(),
        holder: "chat:chat-a".into(),
        globs: vec![],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
        token: None,
        joined: None,
        model: None,
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    // env transport is per child process — the threaded suite never sets
    // these vars in-process (see docs/reports/2026-08-11-fast-follows).
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            // Isolate the identity pin file (dc-g5x5): without this, a join
            // in one run plants a REAL user-level pin that redirects the
            // next run's identically-keyed identity to a dead temp store.
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        let out = c.output().unwrap();
        assert!(
            out.status.success(),
            "{:?} failed: {}{}",
            args,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    };
    let last_create = || {
        let log = s.read_log().unwrap();
        log.iter()
            .rev()
            .find(|ev| ev.get("op").and_then(|v| v.as_str()) == Some("create"))
            .cloned()
            .unwrap()
    };
    // WORK-ONLY STAMPING (dc-zbxj — the semantic flip): the dispatching
    // chat's own q acts no longer stamp; the held entry is refusal and
    // boundary material only. The defect this kills: the dispatcher's
    // unrelated mid-flight acts landing in the dispatch trace.
    run(&[("QUARRY_CHAT", "chat-a")], &["new", "thread", "t one"]);
    assert!(last_create().get("dispatch").is_none(), "the holding chat's acts stamp nothing");
    // a foreign chat's acts never stamped and still do not
    run(&[("QUARRY_CHAT", "chat-z")], &["new", "thread", "t two"]);
    assert!(last_create().get("dispatch").is_none(), "foreign chat stays unstamped");
    // session-keyed holdings stamp nothing either
    let mut d2 = d.clone();
    d2.item = "it-se55".into();
    d2.session = "docs".into();
    d2.holder = "session:docs".into();
    quarry::coord::save_dispatch(&s, &d2).unwrap();
    run(&[("QUARRY_SESSION", "docs")], &["new", "thread", "t three"]);
    assert!(last_create().get("dispatch").is_none(), "the holding session's acts stamp nothing");
    // env badge (the out-of-hook-coverage override) stamps AND teaches an
    // association under the BEST identity key — agent over chat, because a
    // subagent's chat id is the parent's (probed 2026-08-13)
    run(
        &[("QUARRY_DISPATCH", "it-bdg"), ("QUARRY_AGENT", "ag-1"), ("QUARRY_CHAT", "chat-a")],
        &["new", "thread", "t four"],
    );
    assert_eq!(last_create().get("dispatch").and_then(|v| v.as_str()), Some("it-bdg"));
    let m = quarry::coord::load_dispatches(&s);
    assert_eq!(m.acting.get("agent:ag-1").map(|b| b.as_str()), Some("it-bdg"));
    assert!(
        m.acting.get("chat:chat-a").is_none(),
        "with an agent id present the chat key never records — it may be the parent's"
    );
    // the association ALONE now stamps — the same road q join constructs;
    // no env badge in this shell
    run(&[("QUARRY_AGENT", "ag-1")], &["new", "thread", "t five"]);
    assert_eq!(
        last_create().get("dispatch").and_then(|v| v.as_str()),
        Some("it-bdg"),
        "an associated agent stamps without the env override"
    );
    // chat-keyed association (the no-agent-id fallback) stamps the same way
    quarry::coord::record_acting_chat(&s, "chat-agent", "it-bdg");
    run(&[("QUARRY_CHAT", "chat-agent")], &["new", "thread", "t six"]);
    assert_eq!(last_create().get("dispatch").and_then(|v| v.as_str()), Some("it-bdg"));
    // clear kills the held entry and every association: later acts unstamped
    quarry::coord::clear_dispatch(&s, "it-bdg");
    run(&[("QUARRY_AGENT", "ag-1")], &["new", "thread", "t seven"]);
    assert!(last_create().get("dispatch").is_none());
    assert!(quarry::coord::badge_for(&s, Some("ag-1"), None, None).is_none());
    assert!(quarry::coord::badge_for(&s, None, Some("chat-agent"), None).is_none());
}

#[test]
fn observe_write_contract_echo_and_drift() {
    use quarry::teach::observe_write;
    let s = temp_store();
    let d = quarry::coord::DispatchState {
        item: "it-bdg".into(),
        item_title: "guard growth".into(),
        session: "geo".into(),
        holder: "chat:chat-disp".into(),
        globs: vec!["src/**".into()],
        acceptance: vec!["a".into(), "b".into()],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: Store::now(),
        token: None,
        joined: None,
        model: None,
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    // first badged write echoes the contract once
    let out = observe_write(&s, &[], Some("geo"), Some("it-bdg"), "src/main.rs");
    assert!(
        out.iter().any(|l| l.contains("first write under dispatch") && l.contains("guard growth")),
        "got {:?}",
        out
    );
    let out2 = observe_write(&s, &[], Some("geo"), Some("it-bdg"), "src/main.rs");
    assert!(out2.is_empty(), "echo is first-touch only, got {:?}", out2);
    assert_eq!(quarry::coord::touched_for(&s, "item:it-bdg"), vec!["src/main.rs"]);
    // drift: an event lands on the item, the throttle expires — noticed once
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": "it-bdg", "v": 2, "op": "set", "actor": "someone-else"
    }))
    .unwrap();
    let mut d2 = quarry::coord::dispatch_for_item(&s, "it-bdg").unwrap();
    assert_eq!(d2.holder, "chat:chat-disp", "the entry keeps its dispatching-chat holder");
    d2.checked = "2000-01-01T00:00:00Z".into();
    quarry::coord::save_dispatch(&s, &d2).unwrap();
    let out3 = observe_write(&s, &[], Some("geo"), Some("it-bdg"), "src/other.rs");
    assert!(out3.iter().any(|l| l.contains("dispatch drift")), "got {:?}", out3);
    // cursor advanced by the delivery — saved back under the same item key
    let mut d3 = quarry::coord::dispatch_for_item(&s, "it-bdg").unwrap();
    assert_eq!(d3.holder, "chat:chat-disp");
    d3.checked = "2000-01-01T00:00:00Z".into();
    quarry::coord::save_dispatch(&s, &d3).unwrap();
    let out4 = observe_write(&s, &[], Some("geo"), Some("it-bdg"), "src/third.rs");
    assert!(!out4.iter().any(|l| l.contains("dispatch drift")), "said once, got {:?}", out4);
}

#[test]
fn observe_write_leaseless_threshold_nudge() {
    use quarry::teach::observe_write;
    let s = temp_store();
    let it = ops::new_node(&s, NewArgs::bare("item", "guard growth arc")).unwrap();
    ops::set(&s, &it.front.id, &["write-set+=src/**".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let m = queries::items_matching_files(&all, &["src/lease.rs".to_string()]);
    assert_eq!(m.len(), 1, "write-set glob matches the touched file");
    // graph paths and paths outside the repo root never accrue
    assert!(observe_write(&s, &[], Some("solo"), None, "graph/sessions.json").is_empty());
    assert!(observe_write(&s, &[], Some("solo"), None, "c:/users/x/scratch/notes.md").is_empty());
    assert!(observe_write(&s, &[], Some("solo"), None, "/tmp/notes.md").is_empty());
    assert!(quarry::coord::touched_for(&s, "session:solo").is_empty());
    // two distinct files: quiet; a duplicate does not advance the count
    assert!(observe_write(&s, &[], Some("solo"), None, "src/a.rs").is_empty());
    assert!(observe_write(&s, &[], Some("solo"), None, "src/b.rs").is_empty());
    assert!(observe_write(&s, &[], Some("solo"), None, "src/b.rs").is_empty());
    // the third distinct file crosses the threshold: nudge, with the match
    let out = observe_write(&s, &[], Some("solo"), None, "src/c.rs");
    assert!(
        out.iter().any(|l| l.contains("arc is forming") && l.contains("guard growth arc")),
        "got {:?}",
        out
    );
    // once per session: the fourth is silent
    assert!(observe_write(&s, &[], Some("solo"), None, "src/d.rs").is_empty());
    // a session holding a lease is not leaseless — no nudge from this layer
    let holder = ops::new_node(&s, NewArgs::bare("item", "held work")).unwrap();
    quarry::coord::reserve(&s, &holder, "lessee", "t", vec!["docs/**".into()], false, false, None).unwrap();
    let leases = quarry::coord::load_leases(&s);
    assert!(observe_write(&s, &leases, Some("lessee"), None, "src/e.rs").is_empty());
    assert!(observe_write(&s, &leases, Some("lessee"), None, "src/f.rs").is_empty());
    let third = observe_write(&s, &leases, Some("lessee"), None, "src/g.rs");
    assert!(third.is_empty(), "lease holders get the scope-creep warn, not the nudge: {:?}", third);
}

#[test]
fn dispatch_one_act_then_harvest() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "geo pass");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    let out = ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "geo", "t").unwrap();
    // the hand-off is a FETCH (dc-zbxj): one line, token inside, nothing
    // hand-carried — the manual-export instruction is dead
    assert_eq!(out.spawn.lines().count(), 1, "spawn prompt is one line: {}", out.spawn);
    assert!(out.spawn.contains(&format!("q join {}", out.token)), "spawn prompt carries the fetch line");
    assert!(!out.spawn.contains("QUARRY_DISPATCH"), "per-shell export died from the payload");
    assert!(!out.spawn.contains("DISPATCH BRIEF"), "the brief renders at join, never in the hand-off");
    assert!(!out.reused_lease);
    assert!(out.stolen_from.is_none());
    let leases = quarry::coord::load_leases(&s);
    assert_eq!(leases.len(), 1);
    assert_eq!(leases[0].item, it.front.id);
    assert_eq!(leases[0].session, "geo");
    let all = s.load_all().unwrap();
    assert_eq!(s.find(&all, &it.front.id).unwrap().front.status, "in-flight");
    // env unset in-process: the holder falls back to the session key
    let d = quarry::coord::dispatch_for_item(&s, &it.front.id).unwrap();
    assert_eq!(d.holder, "session:geo");
    // the write-set as fired, plus the arc's own report path (it-3prx)
    assert!(d.globs.contains(&"src/geo/**".to_string()), "{:?}", d.globs);
    assert!(quarry::coord::arc_report_in(&d.globs).is_some(), "{:?}", d.globs);
    assert_eq!(d.acceptance.len(), 1, "the contract rides the state file");
    assert_eq!(d.token.as_deref(), Some(out.token.as_str()), "the join token rides the held entry");
    assert!(d.joined.is_none(), "unconsumed until an agent joins");
    assert!(quarry::coord::briefed_this_session(&s, &it.front.id, "geo"), "dispatch briefs (C8)");
    // re-dispatch keeps the lease but is a NEW hand-off: fresh token
    let again = ops::dispatch(&s, &it.front.id, vec![], false, false, None, None, "geo", "t").unwrap();
    assert!(again.reused_lease);
    assert_ne!(again.token, out.token, "a re-dispatch mints a fresh token");
    // FLIP (dc-qyr5, was: "a second dispatch refuses from the chat already
    // holding one"): MULTI-HELD — the same chat dispatches a second item
    // freely; fire-them-all-off from one chat is literal
    let mut other = NewArgs::bare("item", "other work");
    other.acceptance = vec!["the docs land".into()];
    let other = ops::new_node(&s, other).unwrap();
    let par = ops::dispatch(&s, &other.front.id, vec!["docs/**".into()], false, false, None, None, "geo", "t").unwrap();
    assert!(!par.reused_lease);
    let held = quarry::coord::held_dispatches(&s, "session:geo");
    assert_eq!(held.len(), 2, "one chat, two live dispatches (dc-qyr5)");
    // …what refuses now is PER-ITEM ownership: another chat dispatching a
    // live-dispatched item is turned away naming the holding chat and
    // session, with the loud road advertised
    let err = ops::dispatch(&s, &other.front.id, vec![], false, false, None, None, "geo2", "t").unwrap_err();
    assert!(err.to_string().contains("already dispatched"), "got: {}", err);
    assert!(err.to_string().contains("session:geo"), "holding chat named: {}", err);
    assert!(err.to_string().contains("session geo"), "holding session named: {}", err);
    assert!(err.to_string().contains("--steal"), "the loud road advertised: {}", err);
    // an unidentified process amid TWO live badges inherits neither — the
    // machine-global fallback is gone; env transport is the agent's stamp
    let c = ops::claim(
        &s,
        "`geo-pass`: emits layered strata", None, None,
        vec![area.front.id.clone()],
        None,
        Some("read off the pass".into()),
        None,
        None,
    )
    .unwrap();
    let log = s.read_log().unwrap();
    let ev = log
        .iter()
        .rev()
        .find(|e| e.get("node").and_then(|v| v.as_str()) == Some(c.front.id.as_str()))
        .unwrap();
    assert!(ev.get("dispatch").is_none(), "no chat identity, no badge: {}", ev);
    // a badge-stamped act (explicit transport, as an agent's env provides)
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": c.front.id, "v": 1, "op": "create", "type": "claim",
        "actor": "t", "dispatch": it.front.id
    }))
    .unwrap();
    // observed writes accrue per item key — parallel agents land apart
    quarry::coord::accrue_touch(&s, &format!("item:{}", it.front.id), "src/geo/pass.rs");
    quarry::coord::accrue_touch(&s, &format!("item:{}", other.front.id), "docs/other.md");
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(h.contains("src/geo/pass.rs"), "observed file listed");
    assert!(!h.contains("docs/other.md"), "the parallel dispatch's files stay out of this seat");
    assert!(h.contains("stop signal"), "harness done never transitions");
    assert!(h.contains(&format!("q query dispatch {}", it.front.id)), "trace advertised");
    assert!(h.contains("--supports"), "one-command report registration");
    assert!(h.contains("1 claim(s) minted"), "badge-stamped acts counted: {}", h);
    assert!(h.contains("the pass lands"), "RETURN spec re-listed for judging");
    let t = quarry::render::dispatch_trace(&s, &it.front.id).unwrap();
    assert!(t.contains("src/geo/pass.rs"));
    assert!(!t.contains("docs/other.md"), "traces stay per item under parallel badges");
    assert!(t.contains(&c.front.id), "stamped claim in the trace");
    // wrap sees BOTH unharvested dispatches; a harvest event clears only its own
    let all = s.load_all().unwrap();
    let log = s.read_log().unwrap();
    let un = queries::unharvested_dispatches(&all, &log);
    assert!(un.iter().any(|n| n.front.id == it.front.id));
    assert!(un.iter().any(|n| n.front.id == other.front.id));
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": it.front.id, "v": 1, "op": "harvest", "actor": "t"
    }))
    .unwrap();
    let log2 = s.read_log().unwrap();
    let un2 = queries::unharvested_dispatches(&all, &log2);
    assert!(!un2.iter().any(|n| n.front.id == it.front.id));
    assert!(un2.iter().any(|n| n.front.id == other.front.id), "the parallel arc stays owed");
    // clearing one badge is exact under multi-held: the SAME chat's other
    // live dispatch stands untouched
    quarry::coord::clear_dispatch(&s, &it.front.id);
    assert!(quarry::coord::dispatch_for_item(&s, &it.front.id).is_none());
    let left = quarry::coord::held_dispatches(&s, "session:geo");
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].item, other.front.id);
}

#[test]
fn dispatch_refuses_settled_and_foreign_lease() {
    let s = temp_store();
    let mut done = NewArgs::bare("item", "landed work");
    done.status = Some("done".into());
    let done = ops::new_node(&s, done).unwrap();
    let err = ops::dispatch(&s, &done.front.id, vec!["src/**".into()], false, false, None, None, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("[done]"), "got: {}", err);
    // a foreign SOLO lease (no live dispatch) still blocks dispatch with the
    // holder named — the dispatch steal takes dispatches, not solo leases
    let mut it = NewArgs::bare("item", "contested work");
    it.acceptance = vec!["the work lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    quarry::coord::reserve(&s, &it, "bodies", "t", vec!["src/x/**".into()], false, false, None).unwrap();
    let err = ops::dispatch(&s, &it.front.id, vec!["src/x/**".into()], false, false, None, None, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("bodies"), "got: {}", err);
    // no globs anywhere refuses with the teaching line
    let mut bare = NewArgs::bare("item", "bare work");
    bare.acceptance = vec!["the work lands".into()];
    let bare = ops::new_node(&s, bare).unwrap();
    let err = ops::dispatch(&s, &bare.front.id, vec![], false, false, None, None, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("--files"), "got: {}", err);
}

#[test]
fn dispatch_steal_takes_the_dispatch_whole() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "contested pass");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    let out = ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "geo", "t").unwrap();
    // an agent joined the original dispatch and works under it
    ops::join(&s, &out.token, Some("agent:ag-old".into())).unwrap();
    assert_eq!(quarry::coord::badge_for(&s, Some("ag-old"), None, None).as_deref(), Some(it.front.id.as_str()));
    // --steal without --reason refuses: the reason is required (dc-qyr5),
    // and nothing moved
    let err = ops::dispatch(&s, &it.front.id, vec![], false, true, None, None, "ops", "t").unwrap_err();
    assert!(err.to_string().contains("--reason"), "got: {}", err);
    assert_eq!(quarry::coord::dispatch_for_item(&s, &it.front.id).unwrap().holder, "session:geo");
    // steal with the reason takes the dispatch WHOLE
    let st = ops::dispatch(&s, &it.front.id, vec![], false, true, Some("holder went dark"), None, "ops", "t").unwrap();
    assert_eq!(
        st.stolen_from,
        Some(("session:geo".to_string(), "geo".to_string())),
        "the take-over names where it came from"
    );
    assert_ne!(st.token, out.token, "a steal is a new hand-off: fresh token");
    let d = quarry::coord::dispatch_for_item(&s, &it.front.id).unwrap();
    assert_eq!(d.holder, "session:ops", "the held entry moved to the stealing chat");
    assert_eq!(d.session, "ops");
    assert!(d.joined.is_none(), "joined reset — the new agent joins fresh");
    // the lease moved whole: same globs, re-homed session
    let leases = quarry::coord::load_leases(&s);
    assert_eq!(leases.len(), 1);
    assert_eq!(leases[0].session, "ops");
    assert!(leases[0].globs.contains(&"src/geo/**".to_string()), "globs survive the take-over");
    assert!(
        quarry::coord::arc_report_in(&leases[0].globs).is_some(),
        "and the arc's report path rides across it (it-3prx): {:?}",
        leases[0].globs
    );
    // the old agent's association died with the steal: its acts stop
    // stamping into an arc it no longer works
    assert!(quarry::coord::badge_for(&s, Some("ag-old"), None, None).is_none());
    // the old token is dead; the new one binds the new agent
    let err = ops::join(&s, &out.token, Some("agent:ag-old".into())).unwrap_err();
    assert!(err.to_string().contains("unknown join token"), "got: {}", err);
    let j = ops::join(&s, &st.token, Some("agent:ag-new".into())).unwrap();
    assert_eq!(j.bound.as_deref(), Some("agent:ag-new"));
    // loud and logged: the steal event carries reason and provenance
    let log = s.read_log().unwrap();
    let ev = log
        .iter()
        .rev()
        .find(|e| e.get("op").and_then(|v| v.as_str()) == Some("steal"))
        .expect("steal logged");
    assert_eq!(ev.get("node").and_then(|v| v.as_str()), Some(it.front.id.as_str()));
    assert_eq!(ev.get("from_chat").and_then(|v| v.as_str()), Some("session:geo"));
    assert_eq!(ev.get("from_session").and_then(|v| v.as_str()), Some("geo"));
    assert_eq!(ev.get("from_joined").and_then(|v| v.as_str()), Some("agent:ag-old"));
    assert_eq!(ev.get("reason").and_then(|v| v.as_str()), Some("holder went dark"));
}

#[test]
fn join_consumes_token_binds_and_renders() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "geo pass");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    let out = ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "geo", "t").unwrap();
    // no identity refuses WITHOUT consuming — the retry stays possible
    let err = ops::join(&s, &out.token, None).unwrap_err();
    assert!(err.to_string().contains("no identity"), "got: {}", err);
    assert!(
        quarry::coord::dispatch_for_item(&s, &it.front.id).unwrap().joined.is_none(),
        "an identity-less join spends nothing"
    );
    // join consumes the token, binds the identity, records the association,
    // and renders the brief FRESH from the graph
    let j = ops::join(&s, &out.token, Some("agent:ag-1".into())).unwrap();
    assert!(!j.rejoined);
    assert_eq!(j.bound.as_deref(), Some("agent:ag-1"));
    assert_eq!(j.item_id, it.front.id);
    assert!(j.brief.contains("DISPATCH BRIEF"), "the brief derives at join: {}", j.brief);
    assert!(j.brief.contains("the pass lands"), "RETURN spec rides the derived brief");
    let m = quarry::coord::load_dispatches(&s);
    assert_eq!(m.acting.get("agent:ag-1").map(|b| b.as_str()), Some(it.front.id.as_str()));
    assert_eq!(m.held.get(it.front.id.as_str()).unwrap().joined.as_deref(), Some("agent:ag-1"));
    // the joined identity now resolves the stamping badge
    assert_eq!(
        quarry::coord::badge_for(&s, Some("ag-1"), None, None).as_deref(),
        Some(it.front.id.as_str())
    );
    // the join logged under the badge
    let log = s.read_log().unwrap();
    let ev = log
        .iter()
        .rev()
        .find(|e| e.get("op").and_then(|v| v.as_str()) == Some("join"))
        .expect("join logged");
    assert_eq!(ev.get("dispatch").and_then(|v| v.as_str()), Some(it.front.id.as_str()));
    assert_eq!(ev.get("joined").and_then(|v| v.as_str()), Some("agent:ag-1"));
    // idempotent re-join by the same identity: re-prints, no second act
    let j2 = ops::join(&s, &out.token, Some("agent:ag-1".into())).unwrap();
    assert!(j2.rejoined && j2.bound.is_none());
    assert!(j2.brief.contains("DISPATCH BRIEF"));
    let joins = s
        .read_log()
        .unwrap()
        .iter()
        .filter(|e| e.get("op").and_then(|v| v.as_str()) == Some("join"))
        .count();
    assert_eq!(joins, 1, "a re-join is a read, not an act");
    // a second DIFFERENT identity refuses — single-use, one badge one agent
    let err = ops::join(&s, &out.token, Some("agent:ag-2".into())).unwrap_err();
    assert!(err.to_string().contains("already consumed"), "got: {}", err);
    // unknown token teaches the mint path
    let err = ops::join(&s, "zzzzzzzzzz", Some("agent:ag-1".into())).unwrap_err();
    assert!(err.to_string().contains("unknown join token"), "got: {}", err);
    // a re-dispatch is a new hand-off: fresh token, joined reset, old dead
    let again = ops::dispatch(&s, &it.front.id, vec![], false, false, None, None, "geo", "t").unwrap();
    assert!(quarry::coord::dispatch_for_item(&s, &it.front.id).unwrap().joined.is_none());
    let err = ops::join(&s, &out.token, Some("agent:ag-1".into())).unwrap_err();
    assert!(err.to_string().contains("unknown join token"), "the old token died with the re-dispatch: {}", err);
    // harvest-side clear kills the token with the entry
    quarry::coord::clear_dispatch(&s, &it.front.id);
    let err = ops::join(&s, &again.token, Some("agent:ag-1".into())).unwrap_err();
    assert!(err.to_string().contains("unknown join token"), "got: {}", err);
}

#[test]
fn join_cli_env_identity_transport() {
    // Positive env transport lives in child processes (threaded-suite rule):
    // the CLI resolves identity from hook-injected env and joins.
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "geo pass cli");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    let out = ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "geo", "t").unwrap();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            // Isolate the identity pin file (dc-g5x5): without this, a join
            // in one run plants a REAL user-level pin that redirects the
            // next run's identically-keyed identity to a dead temp store.
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    // an identity-less shell refuses with the teaching error, spending nothing
    let out0 = run(&[], &["join", &out.token]);
    assert!(!out0.status.success(), "no identity, no join");
    assert!(
        String::from_utf8_lossy(&out0.stderr).contains("no identity"),
        "taught: {}",
        String::from_utf8_lossy(&out0.stderr)
    );
    // hook-injected agent identity binds agent-keyed and prints the brief
    let out1 = run(&[("QUARRY_AGENT", "ag-cli")], &["join", &out.token]);
    assert!(out1.status.success(), "join succeeds: {}", String::from_utf8_lossy(&out1.stderr));
    let stdout = String::from_utf8_lossy(&out1.stdout);
    assert!(stdout.contains("joined"), "bind confirmed: {}", stdout);
    assert!(stdout.contains("DISPATCH BRIEF"), "brief rendered fresh: {}", stdout);
    assert_eq!(
        quarry::coord::load_dispatches(&s).acting.get("agent:ag-cli").map(|b| b.as_str()),
        Some(it.front.id.as_str())
    );
    // re-join is idempotent through the CLI too
    let out2 = run(&[("QUARRY_AGENT", "ag-cli")], &["join", &out.token]);
    assert!(out2.status.success());
    assert!(String::from_utf8_lossy(&out2.stdout).contains("already joined"));
    // chat identity is the fallback key where no agent id reaches the shell
    let mut it2 = NewArgs::bare("item", "second work");
    it2.status = Some("ready".into());
    it2.about = vec![area.front.id.clone()];
    it2.acceptance = vec!["the second lands".into()];
    let it2 = ops::new_node(&s, it2).unwrap();
    let out4 = ops::dispatch(&s, &it2.front.id, vec!["docs/**".into()], false, false, None, None, "geo2", "t").unwrap();
    let out5 = run(&[("QUARRY_CHAT", "chat-f")], &["join", &out4.token]);
    assert!(out5.status.success(), "{}", String::from_utf8_lossy(&out5.stderr));
    assert_eq!(
        quarry::coord::load_dispatches(&s).acting.get("chat:chat-f").map(|b| b.as_str()),
        Some(it2.front.id.as_str())
    );
}

#[test]
fn join_refuses_a_second_live_badge_naming_the_roads_out() {
    // it-tanf: one agent, one badge — the sequential multi-join shape that
    // silently re-pointed stamping (the lexicon-trio incident) refuses at
    // the join, before the token is spent, and the first arc stands whole.
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mk = |title: &str, accept: &str| {
        let mut it = NewArgs::bare("item", title);
        it.status = Some("ready".into());
        it.about = vec![area.front.id.clone()];
        it.acceptance = vec![accept.into()];
        ops::new_node(&s, it).unwrap()
    };
    let one = mk("first pass", "the first lands");
    let two = mk("second pass", "the second lands");
    let d1 = ops::dispatch(&s, &one.front.id, vec!["src/one/**".into()], false, false, None, None, "geo", "t").unwrap();
    let d2 = ops::dispatch(&s, &two.front.id, vec!["src/two/**".into()], false, false, None, None, "geo", "t").unwrap();
    // the first join binds as ever
    let j1 = ops::join(&s, &d1.token, Some("agent:ag-multi".into())).unwrap();
    assert_eq!(j1.bound.as_deref(), Some("agent:ag-multi"));
    // the second join REFUSES: the live badge is named with the roads out,
    // and nothing is spent — no bind, no join event, no re-point
    let err = ops::join(&s, &d2.token, Some("agent:ag-multi".into())).unwrap_err().to_string();
    assert!(err.contains("one agent, one badge"), "the rule is named: {}", err);
    assert!(err.contains(&one.front.id), "the live badge is named: {}", err);
    assert!(err.contains("first pass"), "…with its title: {}", err);
    assert!(err.contains("stays live"), "the token survives the refusal: {}", err);
    assert!(err.contains("th-zzqv"), "the bundle road is named: {}", err);
    assert!(
        quarry::coord::dispatch_for_item(&s, &two.front.id).unwrap().joined.is_none(),
        "a refused join spends nothing"
    );
    // the acting map still points at the FIRST badge — the overwrite path
    // is dead, so stamping keeps landing on the item served
    assert_eq!(
        quarry::coord::badge_for(&s, Some("ag-multi"), None, None).as_deref(),
        Some(one.front.id.as_str())
    );
    // attention rides along (it-csm3 caveat): the refused join delivered no
    // area record to the second badge — badge-scoped attention can no
    // longer inherit a last-join blur
    assert!(
        quarry::coord::has_area_read(&s, &quarry::coord::badge_attention_key(&one.front.id), &area.front.id),
        "the first join delivered the area to its badge"
    );
    assert!(
        !quarry::coord::has_area_read(&s, &quarry::coord::badge_attention_key(&two.front.id), &area.front.id),
        "the refused join delivered nothing to the second badge"
    );
    // re-join of the identity's OWN badge stays the idempotent read
    let j1b = ops::join(&s, &d1.token, Some("agent:ag-multi".into())).unwrap();
    assert!(j1b.rejoined && j1b.bound.is_none());
    // the separate-dispatches road: the untouched token binds a fresh agent
    let j2 = ops::join(&s, &d2.token, Some("agent:ag-second".into())).unwrap();
    assert_eq!(j2.bound.as_deref(), Some("agent:ag-second"));
    assert_eq!(
        quarry::coord::badge_for(&s, Some("ag-second"), None, None).as_deref(),
        Some(two.front.id.as_str())
    );
    // harvest frees the identity: the refusal is scoped to LIVE badges
    quarry::coord::clear_dispatch(&s, &one.front.id);
    let three = mk("third pass", "the third lands");
    let d3 = ops::dispatch(&s, &three.front.id, vec!["src/three/**".into()], false, false, None, None, "geo", "t").unwrap();
    let j3 = ops::join(&s, &d3.token, Some("agent:ag-multi".into())).unwrap();
    assert_eq!(j3.bound.as_deref(), Some("agent:ag-multi"), "a harvested arc frees its identity");
    assert_eq!(
        quarry::coord::badge_for(&s, Some("ag-multi"), None, None).as_deref(),
        Some(three.front.id.as_str())
    );
}

#[test]
fn acting_map_holds_one_badge_per_identity_by_construction() {
    // it-tanf, the env road: record_acting (note_acting's writer) declines
    // to re-point an identity bound to a different live badge — the map's
    // single writer never overwrites a live binding, whichever road writes.
    let s = temp_store();
    let held = |item: &str| quarry::coord::DispatchState {
        item: item.into(),
        item_title: format!("work {}", item),
        session: "geo".into(),
        holder: "session:geo".into(),
        globs: vec![],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
        token: None,
        joined: None,
        model: None,
    };
    quarry::coord::save_dispatch(&s, &held("it-one1")).unwrap();
    quarry::coord::save_dispatch(&s, &held("it-two2")).unwrap();
    quarry::coord::record_acting(&s, "agent:ag-c", "it-one1");
    assert_eq!(quarry::coord::badge_for(&s, Some("ag-c"), None, None).as_deref(), Some("it-one1"));
    // a second live badge declines the learn: the row stands untouched
    quarry::coord::record_acting(&s, "agent:ag-c", "it-two2");
    assert_eq!(
        quarry::coord::badge_for(&s, Some("ag-c"), None, None).as_deref(),
        Some("it-one1"),
        "the acting map never re-points a live binding"
    );
    // re-recording the SAME badge stays a no-op, never an error
    quarry::coord::record_acting(&s, "agent:ag-c", "it-one1");
    assert_eq!(quarry::coord::badge_for(&s, Some("ag-c"), None, None).as_deref(), Some("it-one1"));
    // the badge's death frees the identity: clear prunes the row, and the
    // next learn binds
    quarry::coord::clear_dispatch(&s, "it-one1");
    quarry::coord::record_acting(&s, "agent:ag-c", "it-two2");
    assert_eq!(quarry::coord::badge_for(&s, Some("ag-c"), None, None).as_deref(), Some("it-two2"));
    // a stale row pointing at a badge nobody holds is residue, never a
    // binding — it blocks nothing and the fresh learn overwrites it
    let v = serde_json::json!({
        "held": {
            "it-live": {
                "item": "it-live", "item_title": "live work", "session": "geo",
                "holder": "session:geo", "globs": [], "acceptance": [],
                "since": "2026-01-01T00:00:00Z", "cursor": 0,
                "checked": "2026-01-01T00:00:00Z"
            }
        },
        "acting": { "agent:ag-stale": "it-gone" }
    });
    std::fs::write(s.root.join("graph").join(".dispatch.json"), v.to_string()).unwrap();
    quarry::coord::record_acting(&s, "agent:ag-stale", "it-live");
    assert_eq!(
        quarry::coord::badge_for(&s, Some("ag-stale"), None, None).as_deref(),
        Some("it-live"),
        "a dead badge's row never blocks the identity's next arc"
    );
}

#[test]
fn work_only_stamping_held_entry_alone_stamps_nothing() {
    // The fourth acceptance line, isolated: held entries resolve refusal
    // and boundary only; stamping resolves env and association, never the
    // holding chat (dc-zbxj).
    let s = temp_store();
    let d = quarry::coord::DispatchState {
        item: "it-wrk".into(),
        item_title: "the work".into(),
        session: "geo".into(),
        holder: "chat:chat-h".into(),
        globs: vec!["src/**".into()],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
        token: Some("tok4wrk123".into()),
        joined: None,
        model: None,
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    // the held entry, alone, stamps nothing — for its chat, its session, or
    // anyone else
    assert!(quarry::coord::badge_for(&s, None, Some("chat-h"), None).is_none());
    assert!(quarry::coord::badge_for(&s, None, None, Some("geo")).is_none());
    assert!(quarry::coord::current_dispatch_badge(&s).is_none());
    // the WORK's identity stamps — even resolved alongside the holding
    // chat's own identity (the association outranks nothing held)
    quarry::coord::record_acting(&s, "agent:ag-w", "it-wrk");
    assert_eq!(
        quarry::coord::badge_for(&s, Some("ag-w"), Some("chat-h"), Some("geo")).as_deref(),
        Some("it-wrk")
    );
    // and dies with the dispatch
    quarry::coord::clear_dispatch(&s, "it-wrk");
    assert!(quarry::coord::badge_for(&s, Some("ag-w"), None, None).is_none());
}

#[test]
fn brief_carries_dispatch_citizenship_sections() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut vein = NewArgs::bare("claim", "`body-graph`: bodies keep identity");
    vein.about = vec![area.front.id.clone()];
    vein.body = "water bodies keep identity across chunk regeneration by graph persistence".into();
    vein.provenance = Some("user".into());
    ops::new_node(&s, vein).unwrap();
    let mut d = NewArgs::bare("decision", "bodies persist");
    d.provenance = Some("user".into());
    d.about = vec![area.front.id.clone()];
    let d = ops::new_node(&s, d).unwrap();
    let mut it = NewArgs::bare("item", "water body graph");
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["bodies persist across reload".into()];
    it.body = "build the graph".into();
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &d.front.id, false, None).unwrap();
    let mut leaner = NewArgs::bare("item", "river deltas");
    leaner.about = vec![area.front.id.clone()];
    let leaner = ops::new_node(&s, leaner).unwrap();
    ops::link(&s, &leaner.front.id, "depends-on", &it.front.id, false, None).unwrap();
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(text.contains("REFLECTIONS"), "reflections instruction present");
    assert!(text.contains("STOP-REPORTS"), "stop-report instruction present");
    assert!(text.contains(&format!("q harvest {}", it.front.id)), "landing points at harvest");
    assert!(text.contains(&format!("q dispatch {}", it.front.id)), "leaseless brief advertises dispatch");
    assert!(!text.contains("status=done, release any lease"), "the stale landing rule is gone");
    assert!(
        text.contains("graph persistence"),
        "vein shelf renders bodies, not titles: {}",
        text
    );
    assert!(text.contains("who leans on this landing"), "backlinks considered");
    assert!(text.contains("river deltas"));
    assert!(!text.contains("BEHIND CHECK"), "nothing behind yet");
    // bump the cited decision: the render-time behind check confronts the dispatcher
    ops::set(&s, &d.front.id, &["title=bodies persist, voxels derive".to_string()], None).unwrap();
    let text2 = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(text2.contains("DISPATCHER — BEHIND CHECK"), "got: {}", text2);
    assert!(text2.contains(&format!("q affirm {} --to {}", it.front.id, d.front.id)));
}

#[test]
fn builds_on_matrix_shapes_stamp_and_no_status_coupling() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut d1 = NewArgs::bare("decision", "vein landmarks");
    d1.provenance = Some("user".into());
    let d1 = ops::new_node(&s, d1).unwrap();
    let mut d2 = NewArgs::bare("decision", "intent rides items");
    d2.provenance = Some("user".into());
    let d2 = ops::new_node(&s, d2).unwrap();
    // decision → decision, stamped at the target's version like any edge
    let e = ops::link(&s, &d2.front.id, "builds-on", &d1.front.id, false, None).unwrap();
    assert_eq!(e.at, At::V(1));
    // doc → claim and doc → decision
    let spec = ops::new_node(&s, NewArgs::bare("doc", "water spec")).unwrap();
    let c = ops::claim(
        &s,
        "`body-graph`: bodies keep identity", None, None,
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    ops::link(&s, &spec.front.id, "builds-on", &c.front.id, false, None).unwrap();
    ops::link(&s, &spec.front.id, "builds-on", &d1.front.id, false, None).unwrap();
    // other shapes refuse with the teaching error naming the matrix
    let it = ops::new_node(&s, NewArgs::bare("item", "some work")).unwrap();
    let err = ops::link(&s, &it.front.id, "builds-on", &d1.front.id, false, None).unwrap_err();
    assert!(err.to_string().contains("edge matrix"), "got: {}", err);
    assert!(err.to_string().contains("builder → built-upon"), "shapes taught: {}", err);
    let err2 = ops::link(&s, &c.front.id, "builds-on", &d1.front.id, false, None).unwrap_err();
    assert!(err2.to_string().contains("edge not allowed"), "got: {}", err2);
    // NO status coupling: refuting the built-upon claim leaves the builder registered
    let ev = ops::new_node(&s, NewArgs::bare("doc", "remeasurement")).unwrap();
    ops::refute(&s, &c.front.id, &ev.front.id, None).unwrap();
    // and superseding the built-upon decision leaves its builders untouched
    let mut d4 = NewArgs::bare("decision", "landmarks v2");
    d4.provenance = Some("user".into());
    let d4 = ops::new_node(&s, d4).unwrap();
    ops::link(&s, &d4.front.id, "supersedes", &d1.front.id, false, None).unwrap();
    let all = s.load_all().unwrap();
    assert_eq!(s.find(&all, &spec.front.id).unwrap().front.status, "registered");
    assert_eq!(s.find(&all, &d2.front.id).unwrap().front.status, "in-force");
    assert_eq!(
        s.find(&all, &d1.front.id).unwrap().front.status,
        "superseded",
        "the supersedes coupling itself still fires"
    );
}

#[test]
fn builds_on_behind_and_reverse_blast() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "cli")).unwrap();
    let c = ops::claim(
        &s,
        "`geo-pass`: emits layered strata", None, None,
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    let spec = ops::new_node(&s, NewArgs::bare("doc", "geo spec")).unwrap();
    ops::link(&s, &spec.front.id, "builds-on", &c.front.id, false, None).unwrap();
    // an ordinary bump puts the builder behind at ordinary severity
    ops::set(&s, &c.front.id, &["method=read off the pass".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let b = queries::behind(&s, &all);
    let e = b
        .iter()
        .find(|x| x.src.id == spec.front.id && x.rel == "builds-on")
        .expect("builds-on ref reports behind");
    assert_eq!(e.severity, 3, "ordinary severity — content changed");
    // blast from the built-upon walks reverse builds-on to the builders
    let ids = queries::blast(&all, &c.front.id);
    assert!(ids.contains(&spec.front.id), "spec stands on the claim: {:?}", ids);
    // transitive along a decision chain
    let mut da = NewArgs::bare("decision", "root ruling");
    da.provenance = Some("user".into());
    let da = ops::new_node(&s, da).unwrap();
    let mut db = NewArgs::bare("decision", "middle ruling");
    db.provenance = Some("user".into());
    let db = ops::new_node(&s, db).unwrap();
    let mut dc = NewArgs::bare("decision", "leaf ruling");
    dc.provenance = Some("user".into());
    let dc = ops::new_node(&s, dc).unwrap();
    ops::link(&s, &db.front.id, "builds-on", &da.front.id, false, None).unwrap();
    ops::link(&s, &dc.front.id, "builds-on", &db.front.id, false, None).unwrap();
    let all = s.load_all().unwrap();
    let ids = queries::blast(&all, &da.front.id);
    assert!(ids.contains(&db.front.id) && ids.contains(&dc.front.id), "lineage walks transitively: {:?}", ids);
    // a refuted vein enumerates its builders — and flips no builder status
    let ev = ops::new_node(&s, NewArgs::bare("doc", "remeasurement")).unwrap();
    let (_, blast) = ops::refute(&s, &c.front.id, &ev.front.id, None).unwrap();
    assert!(
        blast.iter().any(|n| n.front.id == spec.front.id),
        "a killed capability enumerates the specs standing on it"
    );
    let all = s.load_all().unwrap();
    assert_eq!(s.find(&all, &spec.front.id).unwrap().front.status, "registered");
    let b = queries::behind(&s, &all);
    let e = b
        .iter()
        .find(|x| x.src.id == spec.front.id && x.rel == "builds-on")
        .expect("still behind after the kill");
    assert_eq!(e.severity, 1, "dead target severity, like any rel");
}

#[test]
fn intent_delta_both_directions_join_on_shared_areas() {
    let s = temp_store();
    let hydro = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let geo = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "water body graph");
    it.about = vec![hydro.front.id.clone()];
    it.status = Some("ready".into());
    it.acceptance = vec![
        "lands `body-graph`: bodies persist across reload".into(),
        "lands `halo-check`: the halo stays bounded".into(),
    ];
    let it = ops::new_node(&s, it).unwrap();
    // a vein in the shared area lands one intent
    let bg = ops::claim(
        &s,
        "`body-graph`: bodies keep identity across regen", None, None,
        vec![hydro.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    // the same name in a foreign area lands nothing (and is unintended THERE)
    let foreign = ops::claim(
        &s,
        "`halo-check`: bounded halo", None, None,
        vec![geo.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    // an emergent vein in the item's area that no intent named
    ops::claim(
        &s,
        "`chunk-cache`: regen hits a warm cache", None, None,
        vec![hydro.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    let all = s.load_all().unwrap();
    let d = queries::intent_delta(&all, None);
    assert_eq!(
        d.unlanded.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
        vec!["halo-check"],
        "body-graph landed; halo-check did not"
    );
    assert_eq!(d.unlanded[0].1.front.id, it.front.id);
    let un: Vec<&str> = d.unintended.iter().map(|(n, _)| n.as_str()).collect();
    assert!(un.contains(&"chunk-cache"), "emergent scope surfaces: {:?}", un);
    assert!(un.contains(&"halo-check"), "the foreign-area vein is unintended there: {:?}", un);
    assert!(!un.contains(&"body-graph"), "intended and landed is quiet: {:?}", un);
    // area scoping excludes the foreign claim
    let scoped = queries::intent_delta(&all, Some(hydro.front.id.as_str()));
    assert!(scoped.unintended.iter().all(|(_, c)| c.front.id != foreign.front.id));
    assert_eq!(scoped.unlanded.len(), 1);
    // a refuted vein no longer lands its name
    let ev = ops::new_node(&s, NewArgs::bare("doc", "remeasurement")).unwrap();
    ops::refute(&s, &bg.front.id, &ev.front.id, None).unwrap();
    let all = s.load_all().unwrap();
    let d = queries::intent_delta(&all, None);
    assert!(
        d.unlanded.iter().any(|(n, _)| n == "body-graph"),
        "a refuted vein no longer lands the intent"
    );
    // settling the item removes its names from unlanded — intent settled is
    // no longer owed — but the acceptance still counts as intent
    ops::set(&s, &it.front.id, &["status=done".to_string()], None).unwrap();
    let late = ops::claim(
        &s,
        "`halo-check`: halo bounded, measured late", None, None,
        vec![hydro.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    let all = s.load_all().unwrap();
    let d = queries::intent_delta(&all, None);
    assert!(d.unlanded.is_empty(), "settled intent is no longer owed: {:?}",
        d.unlanded.iter().map(|(n, _)| n).collect::<Vec<_>>());
    assert!(
        !d.unintended.iter().any(|(n, c)| n == "halo-check" && c.front.id == late.front.id),
        "landing does not un-intend: the done item's acceptance still names it"
    );
    assert!(
        d.unintended.iter().any(|(n, _)| n == "chunk-cache"),
        "never-named scope stays visible"
    );
}

#[test]
fn prose_id_scan_shapes_and_code_spans() {
    use quarry::mention::cited_ids;
    // the closed shape, code spans skipped, first-appearance order, dedup
    let ids = cited_ids(
        "builds on dc-wwnk, then `cl-aaaa` in code; th-read as hyphenated prose, do-over too; dc-wwnk again",
    );
    assert_eq!(ids, vec!["dc-wwnk".to_string(), "th-read".into(), "do-over".into()]);
    // embedded shapes never match: word char before or after breaks \b
    assert!(cited_ids("growth-reading and th-abcde and xth-abcd").is_empty());
    // case-sensitive: ids are lowercase
    assert!(cited_ids("DC-WWNK and Th-read and ar-XYZW").is_empty());
    // wrong prefix or wrong length never match
    assert!(cited_ids("zz-abcd and ar-abc and ar-abcde").is_empty());
    // punctuation boundaries match
    assert_eq!(cited_ids("(see it-1a2b)."), vec!["it-1a2b".to_string()]);
}

#[test]
fn unpack_expands_labels_dead_and_skips_code() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let c = ops::claim(
        &s,
        "halo bounded", None, None,
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    let all = s.load_all().unwrap();
    let body = format!(
        "leans on {id}; `{id}` stays literal; th-read is prose",
        id = c.front.id
    );
    // citing node shares the claim's area, so the unpack stays home-quiet
    let un = quarry::mention::unpack(&all, &c, &body);
    assert!(
        un.contains(&format!("leans on {} [claim asserted: `halo bounded`]", c.front.id)),
        "resolved id expands to id [type status: `title`] — live status always renders: {}",
        un
    );
    assert!(
        un.contains(&format!("`{}` stays literal", c.front.id)),
        "code spans skipped: {}",
        un
    );
    assert!(un.contains("th-read is prose"), "danglers stay as written: {}", un);
    // a dead target carries its status label
    let ev = ops::new_node(&s, NewArgs::bare("doc", "remeasurement")).unwrap();
    ops::refute(&s, &c.front.id, &ev.front.id, None).unwrap();
    let all = s.load_all().unwrap();
    let un2 = quarry::mention::unpack(&all, &c, &body);
    assert!(
        un2.contains(&format!("{} [claim, refuted: `halo bounded`]", c.front.id)),
        "dead targets labeled: {}",
        un2
    );
}

#[test]
fn open_unpacks_body_and_lists_derived_mentions_never_blast() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut d = NewArgs::bare("decision", "bodies persist");
    d.provenance = Some("user".into());
    d.about = vec![area.front.id.clone()];
    let d = ops::new_node(&s, d).unwrap();
    let mut it = NewArgs::bare("item", "water body graph");
    it.about = vec![area.front.id.clone()];
    it.body = format!("stands on {} for persistence", d.front.id);
    let it = ops::new_node(&s, it).unwrap();
    // forward: the citing body unpacks in open
    let text = quarry::render::open(&s, &it.front.id, false).unwrap();
    assert!(
        text.contains(&format!("stands on {} [decision in-force: `bodies persist`] for persistence", d.front.id)),
        "open unpacks bare ids with live status: {}",
        text
    );
    // reverse: the cited node lists the mentioner under the derived label
    let td = quarry::render::open(&s, &d.front.id, false).unwrap();
    assert!(td.contains("mentioned by (derived"), "derived label present: {}", td);
    assert!(td.contains(&it.front.id), "mentioner listed: {}", td);
    // and the label is distinct from edges: the mention is not in backlinks
    // (the item has no edge to the decision at all)
    assert!(!td.contains("← it-"), "no edge backlink from the item: {}", td);
    // mentions never traverse: blast from the decision is empty, and the
    // mention leaves no stamped ref for behind
    let all = s.load_all().unwrap();
    assert!(
        queries::blast(&all, &d.front.id).is_empty(),
        "blast stays real-edge-only"
    );
    assert!(
        queries::behind(&s, &all).is_empty(),
        "a mention carries no stamp, so nothing goes behind"
    );
    // bumping the mentioned decision leaves the mentioner untouched
    ops::set(&s, &d.front.id, &["title=bodies persist, voxels derive".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    assert!(queries::behind(&s, &all).is_empty(), "mentions never age");
    let text2 = quarry::render::open(&s, &it.front.id, false).unwrap();
    assert!(
        text2.contains("bodies persist, voxels derive"),
        "unpack always shows the current title: {}",
        text2
    );
}

#[test]
fn brief_unpacks_ids_in_work_body() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut d = NewArgs::bare("decision", "bodies persist");
    d.provenance = Some("user".into());
    d.about = vec![area.front.id.clone()];
    let d = ops::new_node(&s, d).unwrap();
    let mut it = NewArgs::bare("item", "water body graph");
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["bodies persist across reload".into()];
    it.body = format!("build shape ratified in {} — read its body as the spec", d.front.id);
    let it = ops::new_node(&s, it).unwrap();
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(
        text.contains(&format!("{} [decision in-force: `bodies persist`]", d.front.id)),
        "THE WORK unpacks bare ids with live status: {}",
        text
    );
}

#[test]
fn view_carries_hyperlinked_unpack_and_mention_index() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut d = NewArgs::bare("decision", "bodies persist");
    d.provenance = Some("user".into());
    d.about = vec![area.front.id.clone()];
    let d = ops::new_node(&s, d).unwrap();
    let mut it = NewArgs::bare("item", "water body graph");
    it.about = vec![area.front.id.clone()];
    it.body = format!("stands on {} for persistence", d.front.id);
    let it = ops::new_node(&s, it).unwrap();
    let html = quarry::view::render(&s).unwrap();
    // body_html carries the anchor to the node (JSON-escaped in the data blob);
    // the whole expansion is the click target, marked as an unpack span (it-6gj9)
    assert!(
        html.contains(&format!(
            "<a class=\\\"unpack\\\" href=\\\"#/n/{id}\\\">{id} [decision in-force: <code>bodies persist<\\/code>]<\\/a>",
            id = d.front.id
        )),
        "view unpack anchors the whole expansion, marked"
    );
    // the derived mention index is embedded, target -> mentioners
    assert!(
        html.contains(&format!("\"{}\":[\"{}\"]", d.front.id, it.front.id)),
        "mention index embedded"
    );
    assert!(html.contains("Mentioned by"), "derived mentions section present");
}

#[test]
fn dangling_id_shapes_lint() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let mut it = NewArgs::bare("item", "naming pass");
    it.status = Some("done".into());
    it.body = format!("a th-read of the do-over, next to {}", area.front.id);
    let it = ops::new_node(&s, it).unwrap();
    let all = s.load_all().unwrap();
    let dang = quarry::mention::danglers(&all);
    let shapes: Vec<&str> = dang.iter().map(|(_, id)| id.as_str()).collect();
    assert_eq!(shapes, vec!["th-read", "do-over"], "danglers listed in order");
    assert!(
        dang.iter().all(|(n, _)| n.front.id == it.front.id),
        "attributed to the citing node"
    );
    // resolving ids are never danglers; archived bodies leave the lint
    ops::archive(&s, &it.front.id, false).unwrap();
    let all = s.load_all().unwrap();
    assert!(quarry::mention::danglers(&all).is_empty(), "archived bodies are not lint");
    // an archived target carries status AND the archived flag in the unpack
    let un = quarry::mention::unpack(&all, &it, &format!("see {}", it.front.id));
    assert!(
        un.contains(&format!("{} [item done, archived: `naming pass`]", it.front.id)),
        "archived targets labeled with status: {}",
        un
    );
}

#[test]
fn actor_recording_and_safety_prefix() {
    let s = temp_store();
    assert!(quarry::coord::chat_actor(&s, "chat-1").is_none());
    quarry::coord::record_chat_actor(&s, "chat-1", "Fable 5");
    assert_eq!(quarry::coord::chat_actor(&s, "chat-1").as_deref(), Some("Fable 5"));
    assert_eq!(quarry::coord::safe_actor("claude-fable-5"), "claude-fable-5");
    assert_eq!(quarry::coord::safe_actor("Fable 5"), "claude:Fable 5");
}

#[test]
fn open_and_view_show_outbound_mentions() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut d = NewArgs::bare("decision", "bodies persist");
    d.provenance = Some("user".into());
    d.about = vec![area.front.id.clone()];
    let d = ops::new_node(&s, d).unwrap();
    let mut it = NewArgs::bare("item", "water body graph");
    it.about = vec![area.front.id.clone()];
    it.body = format!("stands on {} for persistence; th-read is prose", d.front.id);
    let it = ops::new_node(&s, it).unwrap();
    // open on the MENTIONING node: outbound section lists the cited node,
    // labeled derived, parallel to mentioned-by on the mentioned node
    let text = quarry::render::open(&s, &it.front.id, false).unwrap();
    assert!(text.contains("mentions → (derived"), "outbound section labeled derived: {}", text);
    assert!(
        text.contains(&format!("\"bodies persist\" [decision in-force] ({})", d.front.id)),
        "cited node listed outbound as its atom_ref: {}",
        text
    );
    assert_eq!(
        text.matches("th-read").count(),
        1,
        "a dangling shape stays in the body only — never a listed mention: {}",
        text
    );
    // the mentioned node keeps its inbound section; the mentioning node
    // does NOT list itself inbound
    let td = quarry::render::open(&s, &d.front.id, false).unwrap();
    assert!(td.contains("← mentioned by (derived"), "inbound stays: {}", td);
    assert!(!td.contains("mentions → (derived"), "no outbound on a body that cites nothing: {}", td);
    // an archived mention hides with a count, reachable via --all
    ops::set(&s, &d.front.id, &["status=superseded".to_string()], None).unwrap();
    ops::archive(&s, &d.front.id, false).unwrap();
    let text = quarry::render::open(&s, &it.front.id, false).unwrap();
    assert!(
        text.contains("(1 archived mention(s) hidden"),
        "archived outbound mention counted, never silent: {}",
        text
    );
    let text_all = quarry::render::open(&s, &it.front.id, true).unwrap();
    assert!(
        text_all.contains(&format!("\"bodies persist\" [decision, superseded, archived] ({})", d.front.id)),
        "--all reaches it, archived riding the status slot: {}",
        text_all
    );
    // the view template carries the outbound section, computed from the
    // same embedded mention index the inbound section rides
    let html = quarry::view::render(&s).unwrap();
    assert!(html.contains("Mentions <span class=\"arrow\">→</span>"), "view outbound section present");
    assert!(html.contains("Mentioned by"), "view inbound section stays");
}

#[test]
fn boundary_verbs_refuse_only_for_the_badged_chat() {
    let s = temp_store();
    // no badge anywhere: no refusal
    assert!(quarry::coord::boundary_refusal(&s, "q wrap").is_none());
    let d = quarry::coord::DispatchState {
        item: "it-b0nd".into(),
        item_title: "badged work".into(),
        session: "disp".into(),
        holder: "chat:chat-disp".into(),
        globs: vec!["src/**".into()],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
        token: None,
        joined: None,
        model: None,
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    // ANOTHER chat's live badge no longer captures this context's boundary:
    // this process has no identity (env unset in tests), resolves no badge,
    // and wraps freely — the decisions session keeps its boundary while a
    // steward's work is in flight (dc-ydvb). The refusal for the badged
    // chat itself is covered by wrap_refuses_badged_then_regenerates_view
    // (env transport needs a child process in the threaded suite).
    assert!(
        quarry::coord::boundary_refusal(&s, "q wrap").is_none(),
        "a chat with no badge is free while another chat's dispatch flies"
    );
    // work-only-stamping (dc-zbxj): the holding chat's entry resolves NO
    // stamping badge — its boundary capture reads the held map directly
    // (child-tested in wrap_refuses_badged_then_regenerates_view)
    assert!(
        quarry::coord::badge_for(&s, None, Some("chat-disp"), None).is_none(),
        "held entries stamp nothing"
    );
    // a joined agent's association resolves for stamping AND boundary alike
    quarry::coord::record_acting(&s, "agent:ag-b", "it-b0nd");
    assert_eq!(quarry::coord::badge_for(&s, Some("ag-b"), None, None).as_deref(), Some("it-b0nd"));
    quarry::coord::record_acting_chat(&s, "chat-agent", "it-b0nd");
    assert_eq!(quarry::coord::badge_for(&s, None, Some("chat-agent"), None).as_deref(), Some("it-b0nd"));
    quarry::coord::clear_dispatch(&s, "it-b0nd");
    assert!(quarry::coord::boundary_refusal(&s, "q wrap").is_none());
}

#[test]
fn wrap_refuses_badged_then_regenerates_view_when_clear() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            // Isolate the identity pin file (dc-g5x5): without this, a join
            // in one run plants a REAL user-level pin that redirects the
            // next run's identically-keyed identity to a dead temp store.
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    // env badge: wrap and session resume refuse, teaching. Retire left
    // this loop with it-e6wq — its guard is scoped to the retiree and
    // pinned by session_retire_refuses_only_when_the_retiree_is_implicated.
    for args in [&["wrap"][..], &["session", "resume"][..]] {
        let out = run(&[("QUARRY_DISPATCH", "it-t3st")], args);
        assert!(!out.status.success(), "{:?} must refuse under a badge", args);
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("boundary-verb capture"), "{:?} names the incident class: {}", args, err);
        assert!(err.contains("q harvest it-t3st"), "{:?} teaches the exit: {}", args, err);
    }
    assert!(
        !s.root.join("graph").join("view").join("index.html").exists(),
        "a refused wrap regenerates nothing"
    );
    // two parallel dispatches live machine-locally: one session-held, one
    // chat-held — each captures only its own chat's boundary
    let d = quarry::coord::DispatchState {
        item: "it-loc4".into(),
        item_title: "state-file badge".into(),
        session: "disp".into(),
        holder: "session:disp".into(),
        globs: vec![],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
        token: None,
        joined: None,
        model: None,
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    let mut d2 = d.clone();
    d2.item = "it-ch4t".into();
    d2.holder = "chat:chat-a".into();
    quarry::coord::save_dispatch(&s, &d2).unwrap();
    // BOUNDARY HARVESTS ALL (dc-qyr5): the dispatching session holds a
    // SECOND live dispatch — the refusal enumerates every one with its
    // q harvest command, never first-found
    let mut d3 = d.clone();
    d3.item = "it-mult".into();
    quarry::coord::save_dispatch(&s, &d3).unwrap();
    let out = run(&[("QUARRY_SESSION", "disp")], &["wrap"]);
    assert!(!out.status.success(), "the badged session's wrap refuses without env badge");
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(err.contains("q harvest it-loc4"), "first held dispatch enumerated: {}", err);
    assert!(err.contains("q harvest it-mult"), "second held dispatch enumerated: {}", err);
    assert!(err.contains("2 live dispatch"), "the count is named: {}", err);
    assert!(!err.contains("q harvest it-ch4t"), "another chat's dispatch stays out of this refusal: {}", err);
    // the dispatching chat refuses, resolved by chat identity alone
    let out = run(&[("QUARRY_CHAT", "chat-a")], &["wrap"]);
    assert!(!out.status.success(), "the badged chat's wrap refuses");
    assert!(String::from_utf8_lossy(&out.stderr).contains("q harvest it-ch4t"));
    // a JOINED agent's boundary is captured through its association — the
    // dc-zbxj shape: no env badge, no held entry, just the join's binding
    quarry::coord::record_acting(&s, "agent:ag-w", "it-loc4");
    let out = run(&[("QUARRY_AGENT", "ag-w")], &["wrap"]);
    assert!(!out.status.success(), "a joined agent's wrap refuses");
    assert!(String::from_utf8_lossy(&out.stderr).contains("q harvest it-loc4"));
    assert!(
        !s.root.join("graph").join("view").join("index.html").exists(),
        "refused wraps regenerated nothing"
    );
    // ANOTHER session wraps freely while both dispatches fly — the decisions
    // session keeps its boundary during stewardship (dc-ydvb, the ruling's
    // whole point); an unidentified chat is free too
    let out = run(&[("QUARRY_SESSION", "decisions"), ("QUARRY_CHAT", "chat-d")], &["wrap"]);
    assert!(
        out.status.success(),
        "an unbadged chat wraps while other chats' dispatches are live: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("view regenerated"));
    quarry::coord::clear_dispatch(&s, "it-loc4");
    quarry::coord::clear_dispatch(&s, "it-mult");
    quarry::coord::clear_dispatch(&s, "it-ch4t");
    // badges clear: the once-badged session wraps and regenerates the view
    let out = run(&[("QUARRY_SESSION", "disp")], &["wrap"]);
    assert!(out.status.success(), "unbadged wrap runs: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("view regenerated"), "wrap names the regen: {}", stdout);
    assert!(
        s.root.join("graph").join("view").join("index.html").exists(),
        "the view page exists after wrap — the stale-view class dies"
    );
}

#[test]
fn session_retire_refuses_only_when_the_retiree_is_implicated() {
    // The it-e6wq scoping: every retire was treated as the chat closing its
    // own arc; the guard now reads the RETIREE. Env transport needs a child
    // process in the threaded suite.
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    // Three registered sessions: the dispatching chat's own, a worker with
    // a dispatch of its own in flight, and idle third sessions.
    quarry::coord::save_session(&s, "disp", vec![], None, None, false).unwrap();
    quarry::coord::save_session(&s, "worker", vec![], None, None, false).unwrap();
    quarry::coord::save_session(&s, "eph", vec![], None, None, true).unwrap();
    quarry::coord::save_session(&s, "eph2", vec![], None, None, true).unwrap();
    // The chat's own live dispatch (held by chat-disp, session disp) …
    let d = quarry::coord::DispatchState {
        item: "it-own".into(),
        item_title: "the chat's own arc".into(),
        session: "disp".into(),
        holder: "chat:chat-disp".into(),
        globs: vec!["src/**".into()],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
        token: None,
        joined: None,
        model: None,
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    // … and the worker's dispatch in flight, fired from another chat.
    let mut d2 = d.clone();
    d2.item = "it-wrk".into();
    d2.item_title = "the worker's arc".into();
    d2.session = "worker".into();
    d2.holder = "chat:chat-w".into();
    quarry::coord::save_dispatch(&s, &d2).unwrap();
    // (1) The chat's own session under a live badge refuses, as today —
    // the full boundary capture with its harvest enumeration.
    let out = run(
        &[("QUARRY_SESSION", "disp"), ("QUARRY_CHAT", "chat-disp")],
        &["session", "retire", "disp"],
    );
    assert!(!out.status.success(), "own-session retire refuses under a live badge");
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(err.contains("boundary-verb capture"), "the own-arc refusal names the incident class: {}", err);
    assert!(err.contains("q harvest it-own"), "the own-arc refusal teaches the exit: {}", err);
    // (2) A retiree with a dispatch of its own in flight refuses toward
    // THAT dispatch's q harvest — retire would rip the lease out from
    // under a working agent.
    let out = run(
        &[("QUARRY_SESSION", "disp"), ("QUARRY_CHAT", "chat-disp")],
        &["session", "retire", "worker"],
    );
    assert!(!out.status.success(), "retiring a mid-dispatch session refuses");
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(err.contains("retire refused"), "the refusal is the retiree's, not the asker's: {}", err);
    assert!(err.contains("q harvest it-wrk"), "the refusal points at the retiree's dispatch: {}", err);
    assert!(
        !err.contains("it-own"),
        "the asker's own unrelated dispatch stays out of the retiree's refusal: {}",
        err
    );
    // … and an unbadged, unbound chat meets the same refusal — the guard
    // reads the retiree, never the asker.
    let out = run(&[], &["session", "retire", "worker"]);
    assert!(!out.status.success(), "the retiree's dispatch refuses whoever asks");
    assert!(String::from_utf8_lossy(&out.stderr).contains("q harvest it-wrk"));
    // (3) A third session with no live dispatch retires clean while
    // unrelated badges fly — from the badge-holding chat itself …
    let out = run(
        &[("QUARRY_SESSION", "disp"), ("QUARRY_CHAT", "chat-disp")],
        &["session", "retire", "eph"],
    );
    assert!(
        out.status.success(),
        "a third session with no live dispatch retires while this chat's badge flies: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("session eph retired"));
    // … and from an env-badged context: the active badge belongs to an
    // unrelated arc and the retiree is a third session — the do-jn4s doubt
    // scenario, now proceeding.
    let out = run(&[("QUARRY_DISPATCH", "it-own")], &["session", "retire", "eph2"]);
    assert!(
        out.status.success(),
        "an unrelated badge no longer captures a third session's retire: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let reg = quarry::coord::load_sessions(&s);
    assert!(!reg.contains_key("eph") && !reg.contains_key("eph2"), "registry entries removed");
    // Both dispatches still fly, untouched by the third-session retires.
    let m = quarry::coord::load_dispatches(&s);
    assert!(m.held.contains_key("it-own") && m.held.contains_key("it-wrk"), "live dispatches untouched");
    quarry::coord::clear_dispatch(&s, "it-own");
    quarry::coord::clear_dispatch(&s, "it-wrk");
}

#[test]
fn unlink_retires_edge_logged_without_bump() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "process")).unwrap();
    let mut a = NewArgs::bare("decision", "the cap");
    a.about = vec![area.front.id.clone()];
    let a = ops::new_node(&s, a).unwrap();
    let b = ops::new_node(&s, NewArgs::bare("decision", "prose ids")).unwrap();
    ops::link(&s, &a.front.id, "builds-on", &b.front.id, false, None).unwrap();
    let all = s.load_all().unwrap();
    let src_v = s.find(&all, &a.front.id).unwrap().front.v;
    let dst_v = s.find(&all, &b.front.id).unwrap().front.v;
    // another chat's badge on the machine no longer stamps this context's
    // acts — badge resolution is per acting chat, and this process carries
    // no identity (env unset in tests)
    let d = quarry::coord::DispatchState {
        item: "it-unlk".into(),
        item_title: "unlink arc".into(),
        session: "disp".into(),
        holder: "chat:chat-disp".into(),
        globs: vec![],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
        token: None,
        joined: None,
        model: None,
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    let (src, edge) =
        ops::unlink(&s, &a.front.id, "builds-on", &b.front.id, Some("mislink: wrong target".into()))
            .unwrap();
    quarry::coord::clear_dispatch(&s, "it-unlk");
    assert_eq!(edge.rel, "builds-on");
    assert_eq!(edge.to, b.front.id);
    // the edge left the frontmatter — in memory and on disk
    assert!(src.front.edges.iter().all(|e| e.rel != "builds-on"));
    let all = s.load_all().unwrap();
    assert!(
        s.find(&all, &a.front.id).unwrap().front.edges.iter().all(|e| e.rel != "builds-on"),
        "retired edge gone from the reloaded frontmatter"
    );
    // NO version bump on either node (the affirm rationale)
    assert_eq!(s.find(&all, &a.front.id).unwrap().front.v, src_v, "source never bumps");
    assert_eq!(s.find(&all, &b.front.id).unwrap().front.v, dst_v, "target never bumps");
    // the other edge (about) survives
    assert!(s.find(&all, &a.front.id).unwrap().front.edges.iter().any(|e| e.rel == "about"));
    // the log records the act: op, actor, note — and NO badge: a foreign
    // chat's dispatch is not this process's identity (the semantic flip
    // from the single-slot state; per-chat transport is child-tested in
    // log_events_stamp_the_badge_per_chat)
    let log = s.read_log().unwrap();
    let ev = log
        .iter()
        .rev()
        .find(|e| e.get("op").and_then(|v| v.as_str()) == Some("unlink"))
        .expect("unlink logged");
    assert_eq!(ev.get("node").and_then(|v| v.as_str()), Some(a.front.id.as_str()));
    assert_eq!(ev.get("rel").and_then(|v| v.as_str()), Some("builds-on"));
    assert_eq!(ev.get("to").and_then(|v| v.as_str()), Some(b.front.id.as_str()));
    assert_eq!(ev.get("actor").and_then(|v| v.as_str()), Some("test-user"));
    assert!(ev.get("dispatch").is_none(), "unidentified process inherits no badge: {}", ev);
    assert_eq!(ev.get("note").and_then(|v| v.as_str()), Some("mislink: wrong target"));
    // nothing goes behind over housekeeping
    assert!(queries::behind(&s, &all).is_empty(), "retirement leaves no stale refs");
    // retiring what does not exist refuses and teaches what does
    let err = ops::unlink(&s, &a.front.id, "builds-on", &b.front.id, None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("no edge"), "refusal names the miss: {}", err);
    assert!(err.contains("-[about]->"), "refusal lists the edges that exist: {}", err);
}

#[test]
fn fire_time_routing_offers_leave_and_wake() {
    // it-hapc / dc-crea: routing is a PULL, never a send — q dispatch from a
    // non-dispatch session derives dispatch-kind coverage (kind + purview
    // fit + last_seen) and offers; it never denies and never writes state.
    use quarry::coord::{self, FireRouting};
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let other_area = ops::new_node(&s, NewArgs::bare("area", "bodies")).unwrap();
    let mut args = NewArgs::bare("item", "geo pass");
    args.status = Some("ready".into());
    args.about = vec![area.front.id.clone()];
    args.acceptance = vec!["the pass lands".into()];
    let made = ops::new_node(&s, args).unwrap();
    let all = s.load_all().unwrap();
    let it = s.find(&all, &made.front.id).unwrap();

    // no dispatch-kind session registered: fire — today's behavior holds
    coord::save_session(&s, "design", vec![area.front.id.clone()], Some("design".into()), None, false).unwrap();
    assert!(matches!(coord::fire_routing(&s, it, "design"), FireRouting::Fire));

    // a covering dispatch-kind session, never seen: the WAKE offer
    coord::save_session(
        &s, "steward", vec![area.front.id.clone()],
        Some("dispatch".into()), Some("stewards geo".into()), false,
    ).unwrap();
    match coord::fire_routing(&s, it, "design") {
        FireRouting::Wake(c) => {
            assert_eq!(c.len(), 1);
            assert_eq!(c[0].name, "steward");
            assert!(!c[0].live);
            assert!(c[0].age_secs.is_none(), "never seen carries no age");
            assert_eq!(c[0].charter.as_deref(), Some("stewards geo"), "the charter rides the offer");
        }
        _ => panic!("registered but asleep routes to the wake offer"),
    }

    // heartbeat fresh: the LEAVE offer names the live session
    coord::touch_session(&s, "steward");
    match coord::fire_routing(&s, it, "design") {
        FireRouting::Leave(l) => {
            assert_eq!(l.len(), 1);
            assert_eq!(l[0].name, "steward");
            assert!(l[0].live);
        }
        _ => panic!("a live covering dispatcher routes to the leave offer"),
    }
    // …and routing wrote nothing: no held dispatch, item untouched
    assert!(coord::load_dispatches(&s).held.is_empty(), "routing is stateless");
    let all2 = s.load_all().unwrap();
    assert_eq!(s.find(&all2, &made.front.id).unwrap().front.status, "ready", "the item stays honestly ready");

    // stale heartbeat: asleep again — back to the wake offer, age carried
    std::fs::write(
        s.root.join("graph").join(".sessions-live.json"),
        serde_json::json!({"steward": "2020-01-01T00:00:00Z"}).to_string(),
    ).unwrap();
    match coord::fire_routing(&s, it, "design") {
        FireRouting::Wake(c) => {
            assert!(c[0].age_secs.unwrap() > coord::DISPATCH_LIVE_SECS, "stale age surfaces for the user's eyes");
        }
        _ => panic!("a stale dispatcher is asleep"),
    }

    // purview fit: a dispatcher elsewhere never routes this item
    coord::save_session(&s, "steward", vec![other_area.front.id.clone()], Some("dispatch".into()), None, false).unwrap();
    coord::touch_session(&s, "steward");
    assert!(
        matches!(coord::fire_routing(&s, it, "design"), FireRouting::Fire),
        "no purview fit — fire solo without a surface"
    );

    // an area-less item fits any dispatcher vacuously (all-areas charters, dc-wngq)
    let bare = ops::new_node(&s, NewArgs::bare("item", "bare work")).unwrap();
    let all3 = s.load_all().unwrap();
    let bare = s.find(&all3, &bare.front.id).unwrap();
    match coord::fire_routing(&s, bare, "design") {
        FireRouting::Leave(_) => {}
        _ => panic!("an area-less item fits vacuously"),
    }

    // the dispatch-kind session itself never routes — it IS the dispatcher
    coord::save_session(&s, "steward", vec![area.front.id.clone()], Some("dispatch".into()), None, false).unwrap();
    coord::touch_session(&s, "steward");
    assert!(
        matches!(coord::fire_routing(&s, it, "steward"), FireRouting::Fire),
        "a dispatch session fires, never routes to itself"
    );

    // two live dispatchers: BOTH named, unranked — there is never a choice
    // among live dispatchers; the first to claim dispatches it (dc-qyr5)
    coord::save_session(&s, "steward2", vec![area.front.id.clone()], Some("dispatch".into()), None, false).unwrap();
    coord::touch_session(&s, "steward2");
    match coord::fire_routing(&s, it, "design") {
        FireRouting::Leave(l) => assert_eq!(l.len(), 2, "no choosing among live dispatchers"),
        _ => panic!("both live dispatchers named"),
    }

    // a continuation never routes: once the item is live-dispatched, the
    // re-dispatch and steal flows keep their own surfaces
    ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "design", "t").unwrap();
    assert!(
        matches!(coord::fire_routing(&s, it, "design"), FireRouting::Fire),
        "a live dispatch on the item is a continuation, not a routing case"
    );

    // the wake road: inline launch command without a script, the script when present
    let inline = coord::wake_command(&s, "steward");
    assert!(inline.contains("QUARRY_SESSION=steward"), "inline launch fallback: {}", inline);
    std::fs::write(s.root.join("steward-session.cmd"), "@echo off\r\n").unwrap();
    let script = coord::wake_command(&s, "steward");
    assert!(script.ends_with("steward-session.cmd"), "launcher script preferred: {}", script);
}

#[test]
fn find_word_hyphen_is_a_boundary() {
    // it-hjed: find's word predicate — ASCII alphanumerics are the only
    // word chars; everything else bounds, hyphens included. The lexicon
    // join's contains_word adopted this edge rule (it-sc2u); the remaining
    // divergence is the plural fold — find stays exact.
    assert!(!queries::find_word("click the button", "cli"), "cli must never hit click");
    assert!(queries::find_word("the cli area", "cli"));
    assert!(queries::find_word("cli-area rules", "cli"), "hyphen is a boundary for find");
    assert!(queries::find_word("rides atom_line", "atom"), "underscore is a boundary for find");
    assert!(!queries::find_word("the wrapper", "wrap"), "wrap must not hit wrapper");
    assert!(queries::find_word("q wrap runs the lint", "wrap"));
    assert!(queries::find_word("(cli)", "cli"), "punctuation bounds");
    assert!(queries::find_word("cli", "cli"), "text edges bound");
    assert!(!queries::find_word("anything", ""), "the empty query hits nothing");
}

#[test]
fn lexicon_backtick_floor_two_chars() {
    // it-b5tq under dc-qvtz: the deliberate-name floor — backticked spans
    // join from two characters (2-60); one char stays below the floor.
    let spans =
        queries::backticked_spans("the `cli` verbs and the `q` binary ride `intent-delta`");
    assert_eq!(
        spans,
        vec!["cli".to_string(), "intent-delta".to_string()],
        "cli joins at three chars; q stays below the two-char floor"
    );

    // Through relatedness: a body that backticks `cli` reaches a node
    // whose title carries cli as a word — invisible before the ruling.
    let s = temp_store();
    let cli = ops::new_node(&s, NewArgs::bare("item", "cli output conventions")).unwrap();
    let mut d = NewArgs::bare("decision", "the register ruling");
    d.body = "the `cli` register is title-first prose".into();
    let d = ops::new_node(&s, d).unwrap();
    let all = s.load_all().unwrap();
    let dn = s.find(&all, &d.front.id).unwrap();
    let rel = queries::relatedness(&all, dn);
    assert!(
        rel.iter().any(|(n, _)| n.front.id == cli.front.id),
        "a two-plus-char backticked name joins relatedness: {:?}",
        rel.iter().map(|(n, _)| &n.front.title).collect::<Vec<_>>()
    );

    // Bare short prose stays below the bare-token floors: the same words
    // unbackticked join nothing — nothing deliberate happened there.
    let mut p = NewArgs::bare("doc", "meeting minutes");
    p.body = "the cli register is title-first prose".into();
    let p = ops::new_node(&s, p).unwrap();
    let all = s.load_all().unwrap();
    let pn = s.find(&all, &p.front.id).unwrap();
    let rel = queries::relatedness(&all, pn);
    assert!(
        rel.iter().all(|(n, _)| n.front.id != cli.front.id),
        "bare short prose stays below the floor"
    );
}

#[test]
fn lexicon_plural_fold_compare_time() {
    // it-nuw5: s/es folds at compare time, lexicon side only — watches
    // meets watch, leases meets lease.
    let s = temp_store();
    let watch = ops::new_node(&s, NewArgs::bare("item", "watches collapse behind counts")).unwrap();
    let mut d = NewArgs::bare("decision", "the trigger boundary");
    d.body = "every watch names its trigger before filing".into();
    let d = ops::new_node(&s, d).unwrap();
    let all = s.load_all().unwrap();
    let dn = s.find(&all, &d.front.id).unwrap();
    let rel = queries::relatedness(&all, dn);
    assert!(
        rel.iter().any(|(n, _)| n.front.id == watch.front.id),
        "watches meets watch across the inflection: {:?}",
        rel.iter().map(|(n, _)| &n.front.title).collect::<Vec<_>>()
    );

    // The reverse direction of the fold: a singular title token (six-plus
    // chars — the reverse floor stands) meets its plural in an old body.
    let mut old = NewArgs::bare("doc", "sweep diary");
    old.body = "three renderers own every register".into();
    let old = ops::new_node(&s, old).unwrap();
    let renderer = ops::new_node(&s, NewArgs::bare("item", "renderer cadence policy")).unwrap();
    let all = s.load_all().unwrap();
    let rn = s.find(&all, &renderer.front.id).unwrap();
    let rel = queries::relatedness(&all, rn);
    assert!(
        rel.iter().any(|(n, _)| n.front.id == old.front.id),
        "renderer meets renderers in the reverse pass: {:?}",
        rel.iter().map(|(n, _)| &n.front.title).collect::<Vec<_>>()
    );

    // Find's predicate stays exact — the fold is lexicon side only.
    assert!(!queries::find_word("the watch fires", "watches"), "find stays exact");
    assert!(!queries::find_word("all watches fire", "watch"), "find stays exact");
}

#[test]
fn lexicon_derivational_fold_compare_time() {
    // it-rddg: the derivational family (-less, -ful, -er) folds at compare
    // time under the plural-fold pattern — leases meets leaseless in both
    // directions through the shared stem; the family boundary is a stated
    // decision (DERIVATIONAL_SUFFIXES teaching), and find stays exact.
    assert!(
        queries::contains_word("the leaseless observation path", "leases"),
        "leases meets leaseless: plural strip then family derive"
    );
    assert!(
        queries::contains_word("solo leases release at wrap", "leaseless"),
        "leaseless meets leases: family strip then re-pluralize"
    );
    // The other family members, both compositions.
    assert!(queries::contains_word("a watchful reader", "watches"), "-ful folds");
    assert!(queries::contains_word("the dispatcher counts turns", "dispatch"), "-er folds");
    assert!(queries::contains_word("three watchers fire", "watch"), "-er plus plural folds");
    // The boundary holds: outside the family stays unfolded — a decision.
    assert!(
        !queries::contains_word("the leaseness of it", "lease"),
        "-ness stays outside the family"
    );
    assert!(
        !queries::contains_word("an unleased zone", "lease"),
        "prefixes stay outside the family"
    );
    // The three-char stem floor: user never collapses to us.
    assert!(!queries::contains_word("give us the map", "user"), "the stem floor holds");
    // Find's predicate stays exact — the fold is lexicon side only.
    assert!(!queries::find_word("the leaseless zone", "leases"), "find stays exact");
    assert!(!queries::find_word("solo leases release", "leaseless"), "find stays exact");

    // Through the join both directions, like the plural fold before it:
    // a leases-titled node meets a body speaking of leaseless work, and a
    // leaseless-titled node meets an old body speaking of leases.
    let s = temp_store();
    let leases =
        ops::new_node(&s, NewArgs::bare("item", "leases release at the boundary")).unwrap();
    let mut d = NewArgs::bare("decision", "the observation ruling");
    d.body = "leaseless writes are observed, never denied".into();
    let d = ops::new_node(&s, d).unwrap();
    let all = s.load_all().unwrap();
    let dn = s.find(&all, &d.front.id).unwrap();
    let rel = queries::relatedness(&all, dn);
    assert!(
        rel.iter().any(|(n, _)| n.front.id == leases.front.id),
        "leases meets leaseless across the derivation: {:?}",
        rel.iter().map(|(n, _)| &n.front.title).collect::<Vec<_>>()
    );

    let mut old = NewArgs::bare("doc", "lease diary");
    old.body = "solo leases release at wrap with last rites".into();
    let old = ops::new_node(&s, old).unwrap();
    let leaseless =
        ops::new_node(&s, NewArgs::bare("item", "leaseless observation window")).unwrap();
    let all = s.load_all().unwrap();
    let ln = s.find(&all, &leaseless.front.id).unwrap();
    let rel = queries::relatedness(&all, ln);
    assert!(
        rel.iter().any(|(n, _)| n.front.id == old.front.id),
        "leaseless meets leases in the reverse pass: {:?}",
        rel.iter().map(|(n, _)| &n.front.title).collect::<Vec<_>>()
    );
}

#[test]
fn lexicon_floors_measure_the_raw_token_before_folding() {
    // it-wa6e: the six-char floor measures the RAW written form on either
    // side of the fold, never the folded stem — so a five-char folded pair
    // (lease/leases) joins forward and reverse alike through its six-char
    // member, whichever side carries the inflection, while the bare floors
    // of dc-qvtz stand unchanged: an exact five-char pair stays below.

    // The predicate: a five-char word clears the floor only through a
    // floor-length raw form written in the text.
    assert!(
        queries::contains_word_floored("three leases signed early", "lease", 6),
        "the raw form leases (six chars) clears the floor, then folds"
    );
    assert!(
        queries::contains_word_floored("the leaseless zone", "lease", 6),
        "a derivational raw form clears the floor the same way"
    );
    assert!(
        !queries::contains_word_floored("the lease stands alone", "lease", 6),
        "an exact five-char pair stays below the floor — no floor lowered"
    );
    assert!(
        queries::contains_word_floored("the lease stands alone", "leases", 6),
        "a six-char word joins as contains_word, folding down for the compare"
    );

    // Reverse, the filed blind spot: an old body wrote the plural; the
    // concept earns its node under the five-char singular — the raw form
    // in the body passes the reverse floor, then folds for the compare.
    let s = temp_store();
    let mut old = NewArgs::bare("doc", "field diary");
    old.body = "three leases signed early".into();
    let old = ops::new_node(&s, old).unwrap();
    let item = ops::new_node(&s, NewArgs::bare("item", "lease term policy")).unwrap();
    let all = s.load_all().unwrap();
    let it = s.find(&all, &item.front.id).unwrap();
    let rel = queries::relatedness(&all, it);
    assert!(
        rel.iter().any(|(n, _)| n.front.id == old.front.id),
        "a five-char title token meets its plural in an old body: {:?}",
        rel.iter().map(|(n, _)| &n.front.title).collect::<Vec<_>>()
    );

    // Forward, alike: the five-char member on the candidate title side,
    // the inflection written in the new node's text.
    let s = temp_store();
    let item = ops::new_node(&s, NewArgs::bare("item", "lease term policy")).unwrap();
    let mut d = NewArgs::bare("doc", "field diary");
    d.body = "three leases signed early".into();
    let d = ops::new_node(&s, d).unwrap();
    let all = s.load_all().unwrap();
    let dn = s.find(&all, &d.front.id).unwrap();
    let rel = queries::relatedness(&all, dn);
    assert!(
        rel.iter().any(|(n, _)| n.front.id == item.front.id),
        "a lone five-char title hit carries forward through the written plural: {:?}",
        rel.iter().map(|(n, _)| &n.front.title).collect::<Vec<_>>()
    );

    // The floor stands, reverse: both sides written at five chars joins
    // nothing — no raw form clears six anywhere.
    let s = temp_store();
    let mut old = NewArgs::bare("doc", "field ledger");
    old.body = "the lease stands alone".into();
    let old = ops::new_node(&s, old).unwrap();
    let item = ops::new_node(&s, NewArgs::bare("item", "lease term policy")).unwrap();
    let all = s.load_all().unwrap();
    let it = s.find(&all, &item.front.id).unwrap();
    let rel = queries::relatedness(&all, it);
    assert!(
        rel.iter().all(|(n, _)| n.front.id != old.front.id),
        "an exact five-char pair stays below the reverse floor"
    );

    // The floor stands, forward: the same exact pair carries no lone hit.
    let s = temp_store();
    let item = ops::new_node(&s, NewArgs::bare("item", "lease term policy")).unwrap();
    let mut d = NewArgs::bare("doc", "field ledger");
    d.body = "the lease stands alone".into();
    let d = ops::new_node(&s, d).unwrap();
    let all = s.load_all().unwrap();
    let dn = s.find(&all, &d.front.id).unwrap();
    let rel = queries::relatedness(&all, dn);
    assert!(
        rel.iter().all(|(n, _)| n.front.id != item.front.id),
        "an exact five-char lone hit stays below the forward gate"
    );
}

#[test]
fn lexicon_compound_halves_join_and_outrank() {
    // it-sc2u: sig_tokens emits hyphen compounds whole plus halves of
    // five-plus chars; contains_word adopts hyphen-as-boundary; compound
    // hits outrank fragment hits.
    let s = temp_store();
    let comp = ops::new_node(&s, NewArgs::bare("item", "core-sample archive shelf")).unwrap();

    // Spelling variance joins: the spaced mention meets the compound.
    let mut spaced = NewArgs::bare("doc", "field diary");
    spaced.body = "the core sample readings arrived unlabeled".into();
    let spaced = ops::new_node(&s, spaced).unwrap();
    let all = s.load_all().unwrap();
    let sn = s.find(&all, &spaced.front.id).unwrap();
    let rel = queries::relatedness(&all, sn);
    assert!(
        rel.iter().any(|(n, _)| n.front.id == comp.front.id),
        "core sample meets core-sample: {:?}",
        rel.iter().map(|(n, _)| &n.front.title).collect::<Vec<_>>()
    );

    // Compound hits outrank fragment hits, and the why names the whole name.
    let frag = ops::new_node(&s, NewArgs::bare("item", "sample handling bench")).unwrap();
    let mut both = NewArgs::bare("doc", "rig diary");
    both.body = "the core-sample rig hums all night".into();
    let both = ops::new_node(&s, both).unwrap();
    let all = s.load_all().unwrap();
    let bn = s.find(&all, &both.front.id).unwrap();
    let rel = queries::relatedness(&all, bn);
    let pos_comp = rel
        .iter()
        .position(|(n, _)| n.front.id == comp.front.id)
        .expect("the compound-titled node joins");
    let pos_frag = rel
        .iter()
        .position(|(n, _)| n.front.id == frag.front.id)
        .expect("the fragment-titled node joins");
    assert!(pos_comp < pos_frag, "the whole-name hit ranks above the fragment hit");
    let (_, why) = &rel[pos_comp];
    assert!(why.contains("core-sample"), "the why names the whole name: {why}");
}

#[test]
fn find_tiers_word_hits_first_loose_tail_last() {
    // it-hjed: tier one is id substring plus title and body word-boundary
    // hits (body-only keeps the matched-in-body label downstream);
    // substring-only hits are the loose tail — always shown, never hidden.
    // The motivating debris: the query "cli" matched inside "click".
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "cli")).unwrap();
    let mut body_hit = NewArgs::bare("item", "renderer sweep");
    body_hit.body = "the cli owns every register".into();
    let body_hit = ops::new_node(&s, body_hit).unwrap();
    let loose_hit =
        ops::new_node(&s, NewArgs::bare("item", "unpack click-target polish")).unwrap();
    let miss = ops::new_node(&s, NewArgs::bare("item", "unrelated work")).unwrap();
    let all = s.load_all().unwrap();

    let hits = queries::find_hits(&all, "cli");
    assert!(hits.strong.iter().any(|n| n.front.id == area.front.id), "title word hit leads");
    assert!(
        hits.body.iter().any(|n| n.front.id == body_hit.front.id),
        "body word hit is tier one, in the labeled body shelf"
    );
    assert!(
        hits.loose.iter().any(|n| n.front.id == loose_hit.front.id),
        "cli inside click is substring-only: the loose tail, shown but labeled"
    );
    assert!(
        hits.strong.iter().chain(&hits.body).all(|n| n.front.id != loose_hit.front.id),
        "debris never leads"
    );
    let every: Vec<&str> = hits
        .strong
        .iter()
        .chain(&hits.body)
        .chain(&hits.loose)
        .map(|n| n.front.id.as_str())
        .collect();
    assert!(!every.contains(&miss.front.id.as_str()), "a non-match stays out entirely");

    // id substring stays tier one: the suffix of a node's own id finds it
    let frag = &body_hit.front.id[3..];
    let hits = queries::find_hits(&all, frag);
    assert!(
        hits.strong.iter().any(|n| n.front.id == body_hit.front.id),
        "id substring is tier one, never loose"
    );
}

// ── the brief renderer (it-wcwd): tiered, framed, mapped, rendered once ──

#[test]
fn brief_opens_with_floor_line_and_contract_first() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut it = NewArgs::bare("item", "water body graph");
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["bodies persist across reload".into()];
    it.body = "build the graph".into();
    let it = ops::new_node(&s, it).unwrap();
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    let preamble = text
        .find("a floor, not the whole interface")
        .expect("the ratified preamble opens every brief");
    let ret = text.find("RETURN SPEC").unwrap();
    let ws = text.find("WRITE-SET").unwrap();
    let rf = text.find("READ-FIRST").unwrap();
    assert!(preamble < ret, "floor line before the contract");
    assert!(ret < ws && ws < rf, "contract precedes semantics: {} {} {}", ret, ws, rf);
}

#[test]
fn backdrop_tiers_by_distinct_shared_terms() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut d_full = NewArgs::bare("decision", "erosion carves canyon walls");
    d_full.provenance = Some("user".into());
    d_full.about = vec![area.front.id.clone()];
    d_full.body =
        "sediment transport is why erosion carves deepest\nsecond line rides the full render"
            .into();
    ops::new_node(&s, d_full).unwrap();
    let mut d_trunc = NewArgs::bare("decision", "glacier retreat opens moraine");
    d_trunc.provenance = Some("user".into());
    d_trunc.about = vec![area.front.id.clone()];
    d_trunc.body =
        "the canyon line is the one that matters\nunrelated moraine detail hides".into();
    let d_trunc = ops::new_node(&s, d_trunc).unwrap();
    let mut d_none = NewArgs::bare("decision", "aquifer depth ruling");
    d_none.provenance = Some("user".into());
    d_none.about = vec![area.front.id.clone()];
    d_none.body = "basalt columns cool evenly".into();
    ops::new_node(&s, d_none).unwrap();
    let mut it = NewArgs::bare("item", "canyon erosion survey");
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["walls mapped".into()];
    it.body = "map sediment walls of the canyon".into();
    let it = ops::new_node(&s, it).unwrap();
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    // 3+ distinct shared terms: full body
    assert!(
        text.contains("second line rides the full render"),
        "3+ terms earn the full body: {}",
        text
    );
    // 1-2: truncated to matching lines, elision marked, dig-in closing
    assert!(text.contains("the canyon line is the one that matters"), "matching line renders");
    assert!(!text.contains("unrelated moraine detail hides"), "non-matching line elides");
    assert!(
        text.contains(&format!("q open {} if it appears to bear on your task", d_trunc.front.id)),
        "the dig-in command closes the truncation"
    );
    assert!(text.contains("[...]"), "elision marks itself");
    // 0: counted, never shown, reachable
    assert!(!text.contains("basalt columns cool evenly"), "0-match body stays out");
    assert!(text.contains("matched nothing here"), "the remainder is counted: {}", text);
    // weight-then-alphabetical: the full-body entry leads the truncated one
    assert!(
        text.find("erosion carves canyon walls").unwrap()
            < text.find("glacier retreat opens moraine").unwrap(),
        "weight descending order"
    );
}

#[test]
fn shared_capability_name_is_automatic_full_body() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut c = NewArgs::bare("claim", "`core-shelf`: groups species rows");
    c.kind = Some("vein".into());
    c.provenance = Some("user".into());
    c.about = vec![area.front.id.clone()];
    c.body = "first line about grouping\nsecond line about nothing shared".into();
    ops::new_node(&s, c).unwrap();
    let mut it = NewArgs::bare("item", "render pass");
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["lands `core-shelf`: the claim shelf renders".into()];
    it.body = "build it".into();
    let it = ops::new_node(&s, it).unwrap();
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(
        text.contains("second line about nothing shared"),
        "a shared backticked capability name is an automatic full body: {}",
        text
    );
    assert!(text.contains("Veins are the counterforce"), "the ratified vein framing frames the section");
}

#[test]
fn adjacency_multiplies_the_match() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mk_dec = |title: &str| {
        let mut d = NewArgs::bare("decision", title);
        d.provenance = Some("user".into());
        d.about = vec![area.front.id.clone()];
        ops::new_node(&s, d).unwrap()
    };
    let x1 = mk_dec("silt gauging protocol");
    let x2 = mk_dec("flume calibration ruling");
    // the candidate: one lexical match plus two cared edges into the adjacency set
    let mut cand = NewArgs::bare("decision", "delta sediment canyon");
    cand.provenance = Some("user".into());
    cand.about = vec![area.front.id.clone()];
    cand.body = "canyon deltas shift\nhidden line adjacency reveals".into();
    let cand = ops::new_node(&s, cand).unwrap();
    ops::link(&s, &cand.front.id, "builds-on", &x1.front.id, false, None).unwrap();
    ops::link(&s, &cand.front.id, "builds-on", &x2.front.id, false, None).unwrap();
    let mut it = NewArgs::bare("item", "canyon erosion survey");
    it.about = vec![area.front.id.clone()];
    it.body = "map the walls".into();
    it.acceptance = vec!["the report registers".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &x1.front.id, false, None).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &x2.front.id, false, None).unwrap();
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(
        text.contains("hidden line adjacency reveals"),
        "1 lexical match + 2 adjacency edges clears the full-body tier: {}",
        text
    );
}

#[test]
fn claim_shelf_renders_per_species_and_area_open_carries_it() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mk_claim = |title: &str, kind: &str| {
        let mut c = NewArgs::bare("claim", title);
        c.kind = Some(kind.into());
        c.provenance = Some("user".into());
        c.about = vec![area.front.id.clone()];
        ops::new_node(&s, c).unwrap()
    };
    mk_claim("`silt-gauge`: reads the flume", "vein");
    mk_claim("`delta-index`: the receipt of deltas", "feature");
    mk_claim("flume throughput holds at depth", "measured");
    mk_claim("august silt values inform the weir", "reading");
    let text = quarry::render::open(&s, &area.front.id, false).unwrap();
    assert!(text.contains("claims shelf"), "area open carries the species shelf: {}", text);
    assert!(text.contains("VEINS — In a human codebase"), "one-line disposition frames the species");
    assert!(text.contains("RECEIPTS — Receipts are the index"));
    assert!(text.contains("MEASURED — Measured claims are living facts"));
    assert!(text.contains("READINGS — Readings are values taken at a moment"));
    assert!(!text.contains("handrolling anew"), "the old hint line died into the shelf");
    // the brief renders the same species sections, framed in full
    let mut it = NewArgs::bare("item", "weir survey");
    it.about = vec![area.front.id.clone()];
    it.body = "study the flume and the weir".into();
    it.acceptance = vec!["the study registers".into()];
    let it = ops::new_node(&s, it).unwrap();
    let brief = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(brief.contains("VEINS:"), "brief claim shelf groups per species: {}", brief);
    assert!(brief.contains("READINGS:"));
    assert!(brief.contains("sediment, not signal"), "the readings framing ships ratified");
}

#[test]
fn brief_map_renders_file_geography_claims_and_mentions() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::create_dir_all(s.root.join("src")).unwrap();
    std::fs::write(s.root.join("src").join("hydro.rs"), "fn flow() {}\n").unwrap();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut it = NewArgs::bare("item", "weir survey");
    it.about = vec![area.front.id.clone()];
    it.body = "study the weir".into();
    it.acceptance = vec!["the study registers".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "about", "file:src/hydro.rs", false, None).unwrap();
    // a claim over the same file — deliberately zero lexical overlap
    ops::claim(
        &s,
        "the flow mechanism holds through regeneration",
        None,
        None,
        vec!["file:src/hydro.rs".to_string()],
        Some("file:src/hydro.rs".into()),
        None,
        Some("assistant".into()),
        None,
    )
    .unwrap();
    // a body that cites the item
    let mut th = NewArgs::bare("thread", "does the weir shade the flume?");
    th.provenance = Some("user".into());
    th.about = vec![area.front.id.clone()];
    th.body = format!("evidence may come from {}", it.front.id);
    ops::new_node(&s, th).unwrap();
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(text.contains("THE MAP"), "the map renders: {}", text);
    assert!(text.contains("file:src/hydro.rs"), "the item's file edges render with blobs");
    assert!(
        text.contains("the flow mechanism holds"),
        "claims over the item's files ride the map at full fidelity"
    );
    assert!(text.contains("mentioned by"), "the item's mentioned-by backlinks ride the map");
    assert!(text.contains("does the weir shade the flume?"), "the citing body appears");
}

#[test]
fn render_once_a_read_first_body_refs_in_backdrop() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut d = NewArgs::bare("decision", "erosion carves canyon walls");
    d.provenance = Some("user".into());
    d.about = vec![area.front.id.clone()];
    d.body = "distinctive-erosion-substance holds the canyon walls story".into();
    let d = ops::new_node(&s, d).unwrap();
    let mut it = NewArgs::bare("item", "canyon erosion survey");
    it.about = vec![area.front.id.clone()];
    it.body = "map sediment walls of the canyon".into();
    it.acceptance = vec!["the report registers".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &d.front.id, false, None).unwrap();
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    assert_eq!(
        text.matches("distinctive-erosion-substance").count(),
        1,
        "a node renders once at its highest earned fidelity: {}",
        text
    );
    assert!(text.contains("body in READ-FIRST above"), "the backdrop position is a one-line ref");
}

#[test]
fn your_writes_states_expected_acts_by_kind() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut it = NewArgs::bare("item", "weir survey");
    it.kind = Some("slice".into());
    it.about = vec![area.front.id.clone()];
    it.body = "study the weir".into();
    it.acceptance = vec!["the study registers".into()];
    let it = ops::new_node(&s, it).unwrap();
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(text.contains("YOUR-WRITES"), "the your-writes section renders: {}", text);
    assert!(text.contains("a `vein` for each mechanism"), "slice expectation derives from kind");
    assert!(text.contains("fool's gold"), "the ratified close ships");
    assert!(text.contains("--kind <species>"), "the claim shape is stated");
}

// ── sediment and rot (dc-6gn9, it-nmzn): the reading split, the behind
// classifier, archive-on-consumption, species-shaped affirm ─────────────

#[test]
fn claim_kind_plumbs_at_mint() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let c = ops::claim(
        &s,
        "the weir gauge read 4m at survey time",
        None,
        Some("reading".into()),
        vec![area.front.id.clone()],
        None,
        Some("gauge inspection".into()),
        None,
        None,
    )
    .unwrap();
    assert_eq!(c.front.kind.as_deref(), Some("reading"));
}

#[test]
fn behind_tells_sediment_from_rot() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let reading = ops::claim(
        &s,
        "flow was 4m/s on Tuesday",
        None,
        Some("reading".into()),
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    let measured = ops::claim(
        &s,
        "the gauge reports live flow",
        None,
        Some("measured".into()),
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    // Bump the area: both claims drift — the reading as sediment, the
    // living measurement as rot.
    ops::set(&s, &area.front.id, &["title=hydrology".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let behind = queries::behind(&s, &all);
    let r = behind.iter().find(|b| b.src.id == reading.front.id).unwrap();
    assert!(r.sediment, "drift over a reading is sediment");
    let m = behind.iter().find(|b| b.src.id == measured.front.id).unwrap();
    assert!(!m.sediment, "drift on a living measurement is rot");
}

#[test]
fn dead_source_rots_even_under_a_reading() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let evidence = ops::new_node(&s, NewArgs::bare("doc", "counter-survey")).unwrap();
    let upstream = ops::claim(
        &s,
        "the sluice holds at spring tide",
        None,
        None,
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    let reading = ops::claim(
        &s,
        "read 2m against the sluice mark",
        None,
        Some("reading".into()),
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    ops::link(&s, &reading.front.id, "supports", &upstream.front.id, false, None).unwrap();
    ops::refute(&s, &upstream.front.id, &evidence.front.id, None).unwrap();
    let all = s.load_all().unwrap();
    let behind = queries::behind(&s, &all);
    let e = behind
        .iter()
        .find(|b| b.src.id == reading.front.id && b.to == upstream.front.id)
        .unwrap();
    assert_eq!(e.severity, 1, "refuted target is sev 1");
    assert!(!e.sediment, "a refuted source rots anywhere, reading or not");
}

#[test]
fn affirm_teaching_is_species_shaped() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::create_dir_all(s.root.join("tests")).unwrap();
    std::fs::write(s.root.join("tests/basic.rs"), "#[test] fn weir() {}\n").unwrap();
    std::fs::create_dir_all(s.root.join("src")).unwrap();
    std::fs::write(s.root.join("src/gauge.rs"), "fn gauge() {}\n").unwrap();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let instrumented = ops::claim(
        &s,
        "the weir suite holds the gauge honest",
        None,
        Some("measured".into()),
        vec![area.front.id.clone()],
        Some("file:tests/basic.rs".into()),
        Some("integration tests".into()),
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        queries::affirm_teaching(&instrumented),
        Some(quarry::framings::AFFIRM_INSTRUMENT),
        "a file:tests source is the instrument — affirm teaches re-read"
    );
    let manual = ops::claim(
        &s,
        "the outflow probe answers",
        None,
        Some("measured".into()),
        vec![area.front.id.clone()],
        None,
        Some("liveness probe".into()),
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        queries::affirm_teaching(&manual),
        Some(quarry::framings::AFFIRM_MANUAL),
        "a manual method teaches re-run"
    );
    let reading = ops::claim(
        &s,
        "read 4m off the gauge",
        None,
        Some("reading".into()),
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    assert_eq!(
        queries::affirm_teaching(&reading),
        Some(quarry::framings::first_sentence(quarry::framings::READINGS)),
        "a reading teaches its own sediment framing"
    );
    let vein = ops::claim(
        &s,
        "`gauge-loop`: the polling loop owns retry",
        None,
        Some("vein".into()),
        vec![area.front.id.clone()],
        Some("file:src/gauge.rs".into()),
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(queries::affirm_teaching(&vein), None, "other species teach nothing here");
}

/// The incident of it-awhz in shape: a claim sourced on two files, both
/// drifted, affirmed `--to` a node target it has no ref toward. The count is
/// zero for a reason that says nothing about the claim's state — and the old
/// surface printed the unscoped all-clear ("nothing behind — no restamp
/// needed.") over it. The derivation the new surface reads is measured here.
#[test]
fn a_scoped_affirm_zero_is_a_statement_about_its_scope_alone() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::create_dir_all(s.root.join("src")).unwrap();
    std::fs::write(s.root.join("src/teach.rs"), "fn shell_tokens() {}\n").unwrap();
    std::fs::write(s.root.join("src/render.rs"), "fn brief() {}\n").unwrap();
    let area = ops::new_node(&s, NewArgs::bare("area", "cli")).unwrap();
    let mut it = NewArgs::bare("item", "the heredoc landing");
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the body is one opaque token".into()];
    let it = ops::new_node(&s, it).unwrap();
    let c = ops::claim(
        &s,
        "`parse-plausibility`: the parser swallows here-strings whole",
        None,
        Some("vein".into()),
        vec![area.front.id.clone()],
        Some("file:src/teach.rs".into()),
        None,
        Some("assistant".into()),
        None,
    )
    .unwrap();
    ops::link(&s, &c.front.id, "source", "file:src/render.rs", false, None).unwrap();
    // both sources drift under the claim
    std::fs::write(s.root.join("src/teach.rs"), "fn shell_tokens() { /* heredoc */ }\n").unwrap();
    std::fs::write(s.root.join("src/render.rs"), "fn brief() { /* scope */ }\n").unwrap();
    let all = s.load_all().unwrap();
    let cn = s.find(&all, &c.front.id).unwrap();
    assert_eq!(
        queries::behind_node(&s, &all, cn).len(),
        2,
        "the per-node classifier sees both drifted sources"
    );

    // THE INCIDENT: the recipe names a target this claim has no ref toward.
    let n = ops::affirm(&s, &c.front.id, Some(it.front.id.clone())).unwrap();
    assert_eq!(n, 0, "the scope selected nothing to restamp");
    let all = s.load_all().unwrap();
    let cn = s.find(&all, &c.front.id).unwrap();
    let scope = queries::affirm_scope(&s, &all, cn, &it.front.id);
    assert!(!scope.target_known, "the claim carries no ref toward the item — THIS is the zero");
    assert_eq!(scope.toward, 0);
    assert_eq!(scope.elsewhere, 2, "and the zero says nothing about these two");

    // THE FILE-REF AFFORDANCE: review covered one file of a multi-file claim.
    let n = ops::affirm(&s, &c.front.id, Some("file:src/teach.rs".into())).unwrap();
    assert_eq!(n, 1, "--to takes a file ref, spelled as the behind lines print it");
    let all = s.load_all().unwrap();
    let cn = s.find(&all, &c.front.id).unwrap();
    let scope = queries::affirm_scope(&s, &all, cn, "file:src/teach.rs");
    assert!(scope.target_known);
    assert_eq!(scope.toward, 0, "the affirmed file is current");
    assert_eq!(scope.elsewhere, 1, "the claim's other source is still behind");

    // A DANGLING REF RESTAMPS NOTHING, so an unscoped zero is not proof of
    // currency either — the second zero the surface must not collapse.
    std::fs::remove_file(s.root.join("src/render.rs")).unwrap();
    let n = ops::affirm(&s, &c.front.id, None).unwrap();
    assert_eq!(n, 0, "a missing file cannot be restamped");
    let all = s.load_all().unwrap();
    let cn = s.find(&all, &c.front.id).unwrap();
    assert_eq!(
        queries::behind_node(&s, &all, cn).len(),
        1,
        "the node is still behind after an unscoped zero"
    );
}

/// A scoped affirm acts on its scope alone (it-awhz). The path-backed doc's
/// self-blob restamp used to run regardless of `--to`, so a scoped affirm on
/// a drifted doc restamped its own file and returned a count the caller could
/// only read as the named target having moved.
#[test]
fn a_scoped_affirm_restamps_only_within_the_scope_it_was_given() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::create_dir_all(s.root.join("docs/reports")).unwrap();
    std::fs::write(s.root.join("docs/reports/r.md"), "the outcomes hold\n").unwrap();
    let it = ops::new_node(&s, NewArgs::bare("item", "geo pass")).unwrap();
    let mut doc = NewArgs::bare("doc", "dispatch report: geo pass");
    doc.kind = Some("report".into());
    doc.path = Some("docs/reports/r.md".into());
    let doc = ops::new_node(&s, doc).unwrap();
    ops::link(&s, &doc.front.id, "supports", &it.front.id, false, None).unwrap();
    std::fs::write(s.root.join("docs/reports/r.md"), "the outcomes hold, amended\n").unwrap();

    // scoped at the item: the doc's own file is OUTSIDE that scope
    let n = ops::affirm(&s, &doc.front.id, Some(it.front.id.clone())).unwrap();
    assert_eq!(n, 0, "a scoped affirm never restamps outside its scope");
    let all = s.load_all().unwrap();
    let dn = s.find(&all, &doc.front.id).unwrap();
    let scope = queries::affirm_scope(&s, &all, dn, &it.front.id);
    assert!(scope.target_known, "the supports edge is a ref toward the item");
    assert_eq!(scope.toward, 0, "that edge is current");
    assert_eq!(scope.elsewhere, 1, "the doc's own drifted path is what the scope excluded");

    // scoped at the doc's own file — the spelling the homework line advertises
    let n = ops::affirm(&s, &doc.front.id, Some("file:docs/reports/r.md".into())).unwrap();
    assert_eq!(n, 1, "the advertised scoped command still restamps the doc's blob");
    let all = s.load_all().unwrap();
    let dn = s.find(&all, &doc.front.id).unwrap();
    assert!(queries::behind_node(&s, &all, dn).is_empty(), "and clears the drift");
}

/// End to end through the spawned binary: the two zeros no longer share a
/// line, and the unscoped all-clear still means every ref is current.
#[test]
fn the_affirm_surface_says_which_zero_it_is() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        let out = c.output().unwrap();
        assert!(out.status.success(), "q {:?}: {}", args, String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::create_dir_all(s.root.join("src")).unwrap();
    std::fs::write(s.root.join("src/teach.rs"), "fn shell_tokens() {}\n").unwrap();
    std::fs::write(s.root.join("src/render.rs"), "fn brief() {}\n").unwrap();
    let area = ops::new_node(&s, NewArgs::bare("area", "cli")).unwrap();
    let mut it = NewArgs::bare("item", "the heredoc landing");
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the body is one opaque token".into()];
    let it = ops::new_node(&s, it).unwrap();
    let c = ops::claim(
        &s,
        "`parse-plausibility`: the parser swallows here-strings whole",
        None,
        Some("vein".into()),
        vec![area.front.id.clone()],
        Some("file:src/teach.rs".into()),
        None,
        Some("assistant".into()),
        None,
    )
    .unwrap();
    ops::link(&s, &c.front.id, "source", "file:src/render.rs", false, None).unwrap();
    std::fs::write(s.root.join("src/teach.rs"), "fn shell_tokens() { /* heredoc */ }\n").unwrap();
    std::fs::write(s.root.join("src/render.rs"), "fn brief() { /* scope */ }\n").unwrap();

    // the incident call: the all-clear must not appear over real drift
    let out = run(&["affirm", &c.front.id, "--to", &it.front.id]);
    assert!(
        !out.contains("nothing behind — no restamp needed."),
        "the unscoped all-clear never speaks for a scoped zero: {}",
        out
    );
    assert!(out.contains("no ref toward"), "the zero names its own reason: {}", out);
    assert!(out.contains(&it.front.id), "and names the scope it was given: {}", out);
    assert!(
        out.contains("2 other ref(s) on this node are still behind"),
        "the node's reach beyond the scope is stated: {}",
        out
    );
    assert!(
        out.contains(&format!("q affirm {} unscoped", c.front.id)),
        "with the unscoped command in hand: {}",
        out
    );

    // the file-ref affordance, scoped: restamps one, still says what is left
    let out = run(&["affirm", &c.front.id, "--to", "file:src/teach.rs"]);
    assert!(
        out.contains("restamped 1 ref(s) toward file:src/teach.rs"),
        "a scoped restamp names its scope too: {}",
        out
    );
    assert!(
        out.contains("1 other ref(s) on this node are still behind"),
        "and never reads as an all-clear for the node: {}",
        out
    );

    // unscoped: clears the rest, then earns the all-clear
    let out = run(&["affirm", &c.front.id]);
    assert!(out.contains("restamped 1 ref(s)"), "unscoped restamp keeps its line: {}", out);
    assert!(!out.contains("still behind"), "nothing is left behind: {}", out);
    let out = run(&["affirm", &c.front.id]);
    assert!(
        out.contains("nothing behind — no restamp needed."),
        "the unscoped zero keeps its all-clear, and now means it: {}",
        out
    );

    // a scoped zero over a current ref is its own third line
    let out = run(&["affirm", &c.front.id, "--to", "file:src/teach.rs"]);
    assert!(
        out.contains("nothing behind toward file:src/teach.rs"),
        "a scoped all-clear names the scope: {}",
        out
    );
    assert!(
        out.contains("nothing else on this node is behind either"),
        "and states the reach it does not cover: {}",
        out
    );

    // and the unscoped zero is not proof of currency either: a ref no
    // restamp can reach (a missing file, a dangling target) is behind and
    // stays behind, so it is said out loud instead of collapsing.
    std::fs::remove_file(s.root.join("src/render.rs")).unwrap();
    let out = run(&["affirm", &c.front.id]);
    assert!(
        !out.contains("nothing behind — no restamp needed."),
        "an unrestampable ref never collapses into the all-clear: {}",
        out
    );
    assert!(
        out.contains("1 ref(s) on this node are behind but not restampable"),
        "the unscoped zero says what it could not clear: {}",
        out
    );
}

#[test]
fn readings_archive_when_last_consumer_settles() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let reading = ops::claim(
        &s,
        "the weir gauge read 4m",
        None,
        Some("reading".into()),
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    // Consumerless: nothing consumed it, so nothing settles it.
    assert!(ops::consume_readings(&s).unwrap().is_empty());
    // Two consumers lean on it (supports: the reading informed them).
    let i1 = ops::new_node(&s, NewArgs::bare("item", "sluice design")).unwrap();
    let i2 = ops::new_node(&s, NewArgs::bare("item", "weir survey")).unwrap();
    ops::link(&s, &reading.front.id, "supports", &i1.front.id, false, None).unwrap();
    ops::link(&s, &reading.front.id, "supports", &i2.front.id, false, None).unwrap();
    // First consumer settles: the reading stays live.
    ops::set(&s, &i1.front.id, &["status=done".to_string()], None).unwrap();
    assert!(ops::consume_readings(&s).unwrap().is_empty());
    // The LAST live consumer settles: the reading archives itself.
    ops::set(&s, &i2.front.id, &["status=done".to_string()], None).unwrap();
    let swept = ops::consume_readings(&s).unwrap();
    assert_eq!(swept.len(), 1);
    assert_eq!(swept[0].front.id, reading.front.id);
    let all = s.load_all().unwrap();
    assert!(s.find(&all, &reading.front.id).unwrap().front.archived);
    // Idempotent: a second sweep finds nothing.
    assert!(ops::consume_readings(&s).unwrap().is_empty());
}

#[test]
fn reading_supporting_in_force_decision_is_consumed() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let reading = ops::claim(
        &s,
        "spring tide crested at 5m",
        None,
        Some("reading".into()),
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    let mut d = NewArgs::bare("decision", "raise the sluice wall");
    d.provenance = Some("user".into());
    let d = ops::new_node(&s, d).unwrap();
    ops::link(&s, &reading.front.id, "supports", &d.front.id, false, None).unwrap();
    // An in-force ruling has consumed its inputs: the reading is done serving.
    let swept = ops::consume_readings(&s).unwrap();
    assert_eq!(swept.len(), 1);
    assert_eq!(swept[0].front.id, reading.front.id);
    // A measured claim in the same position stays live: its currency is its value.
    let living = ops::claim(
        &s,
        "the tide gauge streams live",
        None,
        Some("measured".into()),
        vec![area.front.id.clone()],
        None,
        Some("gauge poll".into()),
        None,
        None,
    )
    .unwrap();
    ops::link(&s, &living.front.id, "supports", &d.front.id, false, None).unwrap();
    assert!(ops::consume_readings(&s).unwrap().is_empty());
    let all = s.load_all().unwrap();
    assert!(!s.find(&all, &living.front.id).unwrap().front.archived);
}

#[test]
fn readings_archive_by_species_not_ladder() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::create_dir_all(s.root.join("src")).unwrap();
    std::fs::write(s.root.join("src/culvert.rs"), "fn culvert() {}\n").unwrap();
    let area = ops::new_node(&s, NewArgs::bare("area", "water")).unwrap();
    let reading = ops::claim(
        &s,
        "the culvert ran dry in August",
        None,
        Some("reading".into()),
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    // A reading archives at any ladder status: it settled when it landed.
    let n = ops::archive(&s, &reading.front.id, false).unwrap();
    assert!(n.front.archived);
    // Every other claim species still follows the ladder.
    let vein = ops::claim(
        &s,
        "`dry-run`: the culvert check is a pure function",
        None,
        Some("vein".into()),
        vec![area.front.id.clone()],
        Some("file:src/culvert.rs".into()),
        None,
        None,
        None,
    )
    .unwrap();
    let err = ops::archive(&s, &vein.front.id, false).unwrap_err();
    assert!(err.to_string().contains("only settled statuses archive"), "got: {}", err);
}

#[test]
fn landing_ratifies_badge_mints_and_harvest_names_the_assay() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "assay pass");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "geo", "t")
        .unwrap();
    let mk = |title: &str, kind: &str| {
        ops::claim(
            &s,
            title,
            None,
            Some(kind.into()),
            vec![area.front.id.clone()],
            None,
            None,
            Some("user".into()),
            None,
        )
        .unwrap()
    };
    let vein = mk("`geo-pass`: emits layered strata", "vein");
    let feat = mk("`geo-pass-cli`: the pass runs end to end", "feature");
    let reading = mk("pass runtime was 40ms", "reading");
    let unstamped = mk("`geo-cache`: caches strata", "vein");
    // badge-stamped acts (explicit transport, as an agent's env provides)
    for c in [&vein, &feat, &reading] {
        s.log_event(serde_json::json!({
            "ts": Store::now(), "node": c.front.id, "v": 1, "op": "create", "type": "claim",
            "actor": "t", "dispatch": it.front.id
        }))
        .unwrap();
    }
    // the harvest seat names the assay with the ratified line (dc-drr6)
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(
        h.contains("assay: 2 claim(s) minted under this badge ratify with your landing - the diff is the evidence, your judgment is the act. Ratified never means true; refute and blast stand."),
        "harvest teaches the assay verbatim: {}", h
    );
    assert!(
        h.contains(&vein.front.id) && h.contains(&feat.front.id),
        "the assayable mints are listed: {}", h
    );
    // the landing act ratifies — dispatched arc, harvest path
    let v_before = vein.front.v;
    let assay = ops::ratify_landing(&s, &it.front.id, Some("geo")).unwrap().unwrap();
    assert!(!assay.solo, "a dispatched arc ratifies at harvest, never solo");
    assert_eq!(assay.ratified.len(), 2);
    let all = s.load_all().unwrap();
    let v2 = s.find(&all, &vein.front.id).unwrap();
    assert_eq!(v2.front.status, "ratified");
    let r = v2.front.ratified.as_ref().expect("the assayer is stamped");
    assert_eq!(r.by, "test-user");
    assert_eq!(v2.front.v, v_before, "an assay is bookkeeping — no bump, citers never go behind");
    assert_eq!(s.find(&all, &feat.front.id).unwrap().front.status, "ratified");
    assert_eq!(
        s.find(&all, &reading.front.id).unwrap().front.status,
        "asserted",
        "the ladder is species-shaped: readings never ride it"
    );
    assert_eq!(
        s.find(&all, &unstamped.front.id).unwrap().front.status,
        "asserted",
        "mints outside the badge are not this landing's to assay"
    );
    // the act is loud in the log: op ratify, assay harvest, the landing as cause
    let log = s.read_log().unwrap();
    let ev = log
        .iter()
        .rev()
        .find(|e| {
            e.get("op").and_then(|v| v.as_str()) == Some("ratify")
                && e.get("node").and_then(|v| v.as_str()) == Some(vein.front.id.as_str())
        })
        .expect("ratify logged");
    assert_eq!(ev.get("assay").and_then(|v| v.as_str()), Some("harvest"));
    assert_eq!(ev.get("cause").and_then(|v| v.as_str()), Some(it.front.id.as_str()));
    // idempotent: a second landing finds nothing on the asserted rung
    assert!(ops::ratify_landing(&s, &it.front.id, Some("geo")).unwrap().is_none());
}

#[test]
fn solo_landing_self_ratifies_its_own_arc_mints() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "solo shaft");
    it.about = vec![area.front.id.clone()];
    let it = ops::new_node(&s, it).unwrap();
    let mk = |title: &str| {
        ops::claim(
            &s,
            title,
            None,
            Some("vein".into()),
            vec![area.front.id.clone()],
            None,
            None,
            Some("user".into()),
            None,
        )
        .unwrap()
    };
    // a session mint from before the arc: outside the window, never assayed here
    let early = mk("`old-lode`: predates the arc");
    s.log_event(serde_json::json!({
        "ts": "2020-01-01T00:00:00Z", "node": early.front.id, "v": 1, "op": "create",
        "type": "claim", "actor": "t", "session": "solo1"
    }))
    .unwrap();
    // the arc declares: a lease, and the in-flight flip
    quarry::coord::reserve(&s, &it, "solo1", "t", vec!["src/solo/**".into()], false, false, None)
        .unwrap();
    ops::set(&s, &it.front.id, &["status=in-flight".to_string()], None).unwrap();
    let vein = mk("`new-lode`: cut this arc");
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": vein.front.id, "v": 1, "op": "create",
        "type": "claim", "actor": "t", "session": "solo1"
    }))
    .unwrap();
    // another session's mint in the window is never this landing's to assay
    let foreign = mk("`foreign-lode`: another hand");
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": foreign.front.id, "v": 1, "op": "create",
        "type": "claim", "actor": "t", "session": "solo2"
    }))
    .unwrap();
    // no session at the landing: nothing traceable, nothing ratifies
    assert!(ops::ratify_landing(&s, &it.front.id, None).unwrap().is_none());
    let assay = ops::ratify_landing(&s, &it.front.id, Some("solo1")).unwrap().unwrap();
    assert!(assay.solo, "a never-dispatched arc self-ratifies");
    assert_eq!(assay.ratified.len(), 1);
    assert_eq!(assay.ratified[0].front.id, vein.front.id);
    let all = s.load_all().unwrap();
    let v2 = s.find(&all, &vein.front.id).unwrap();
    assert_eq!(v2.front.status, "ratified");
    assert_eq!(v2.front.ratified.as_ref().unwrap().by, "test-user", "stamped to your hand");
    assert_eq!(
        s.find(&all, &early.front.id).unwrap().front.status,
        "asserted",
        "pre-arc mints stay asserted (standing claims stay least-committal)"
    );
    assert_eq!(
        s.find(&all, &foreign.front.id).unwrap().front.status,
        "asserted",
        "another session's mints are never yours to assay"
    );
    let log = s.read_log().unwrap();
    let ev = log
        .iter()
        .rev()
        .find(|e| e.get("op").and_then(|v| v.as_str()) == Some("ratify"))
        .unwrap();
    assert_eq!(ev.get("assay").and_then(|v| v.as_str()), Some("solo"));
}

#[test]
fn weight_held_is_display_and_the_load_query_warns() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut b1 = NewArgs::bare("item", "built thing");
    b1.status = Some("done".into());
    let b1 = ops::new_node(&s, b1).unwrap();
    let b2 = ops::new_node(&s, NewArgs::bare("decision", "a ruling")).unwrap();
    let mut dr = NewArgs::bare("item", "dropped thing");
    dr.status = Some("dropped".into());
    let dr = ops::new_node(&s, dr).unwrap();
    let mk = |title: &str| {
        ops::claim(
            &s,
            title,
            None,
            Some("vein".into()),
            vec![area.front.id.clone()],
            None,
            None,
            Some("user".into()),
            None,
        )
        .unwrap()
    };
    let heavy = mk("`heavy-lode`: two builds stand on it");
    ops::link(&s, &heavy.front.id, "supports", &b1.front.id, false, None).unwrap();
    ops::link(&s, &heavy.front.id, "supports", &b2.front.id, false, None).unwrap();
    ops::link(&s, &heavy.front.id, "supports", &dr.front.id, false, None).unwrap();
    let light = mk("`light-lode`: nothing stands on it");
    let judged = mk("`judged-lode`: assayed already");
    ops::link(&s, &judged.front.id, "supports", &b1.front.id, false, None).unwrap();
    ops::set(&s, &judged.front.id, &["status=ratified".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let h = s.find(&all, &heavy.front.id).unwrap();
    assert_eq!(queries::weight_held(&all, h), 2, "dropped targets never count");
    let atom = quarry::surface::atom(&all, h);
    assert_eq!(atom.weight, Some(2));
    assert!(
        quarry::surface::atom_line(&atom).contains("holds 2"),
        "load is display on the atom line: {}",
        quarry::surface::atom_line(&atom)
    );
    let l = s.find(&all, &light.front.id).unwrap();
    assert!(!quarry::surface::atom_line(&quarry::surface::atom(&all, l)).contains("holds"));
    let load = queries::load_bearing_unassayed(&all);
    assert!(
        load.iter().any(|(n, w)| n.front.id == heavy.front.id && *w == 2),
        "load-bearing and unassayed warns"
    );
    assert!(
        !load.iter().any(|(n, _)| n.front.id == light.front.id),
        "weightless stays off the warning"
    );
    assert!(
        !load.iter().any(|(n, _)| n.front.id == judged.front.id),
        "an assayed claim carries no warning"
    );
    // the ratified verbiage ships verbatim (dc-dsdm: never invented silently)
    assert_eq!(
        quarry::framings::ASSAY_WARNING,
        "load-bearing but never assayed - builds stand on these and no judge has: fool's gold risk rises with weight. Assay on next touch, or refute."
    );
    assert_eq!(
        quarry::framings::ASSAY_CLAIM_HELP,
        "Claims mint asserted. Veins ratify when a landing's judge verifies them against the diff - at harvest, or your own solo landing. Ratified records who assayed, never truth."
    );
    assert_eq!(
        quarry::framings::assay_solo_line(3),
        "assay: 3 claim(s) from this arc self-ratify with your landing - stamped to your hand; one mind, on the record."
    );
    assert_eq!(
        quarry::framings::assay_harvest_line(2),
        "assay: 2 claim(s) minted under this badge ratify with your landing - the diff is the evidence, your judgment is the act. Ratified never means true; refute and blast stand."
    );
    assert!(
        quarry::framings::VEINS.contains("Each vein carries its assay: ratified means a landing's judge verified it against the diff; asserted means one mind wrote it down and no one has stood behind it since"),
        "the veins framing carries the assay sentence"
    );
}

// ── the acceptance gate (dc-p6z4, it-33bb): ready refuses, reserve
// un-readies, the brief trips, the state is derived ─────────────────────

#[test]
fn ready_gate_refuses_acceptance_less_flip_and_mint() {
    let s = temp_store();
    // the flip refuses, facing the shaper with the authoring command
    let it = ops::new_node(&s, NewArgs::bare("item", "unstated work")).unwrap();
    let err = ops::set(&s, &it.front.id, &["status=ready".to_string()], None).unwrap_err();
    assert!(err.to_string().contains("acceptance gate"), "got: {}", err);
    assert!(err.to_string().contains("dc-p6z4"), "the ruling is named: {}", err);
    assert!(
        err.to_string().contains("acceptance+="),
        "the refusal teaches the authoring command: {}",
        err
    );
    let all = s.load_all().unwrap();
    assert_eq!(
        s.find(&all, &it.front.id).unwrap().front.status,
        "sketch",
        "a refused flip mutates nothing"
    );
    // authoring acceptance and flipping ready in ONE act passes, either order
    ops::set(
        &s,
        &it.front.id,
        &["status=ready".to_string(), "acceptance+=the work lands".to_string()],
        None,
    )
    .unwrap();
    let all = s.load_all().unwrap();
    assert_eq!(s.find(&all, &it.front.id).unwrap().front.status, "ready");
    // demotions stay free: shaped accepts an acceptance-less item
    let bare = ops::new_node(&s, NewArgs::bare("item", "quiet sketch")).unwrap();
    ops::set(&s, &bare.front.id, &["status=shaped".to_string()], None).unwrap();
    // the mint path holds the same invariant: no construction reaches ready
    let mut m = NewArgs::bare("item", "minted hot");
    m.status = Some("ready".into());
    let err = ops::new_node(&s, m).unwrap_err();
    assert!(err.to_string().contains("acceptance gate"), "got: {}", err);
    assert!(err.to_string().contains("--acceptance"), "the mint refusal teaches the flag: {}", err);
    // with the contract stated, the mint stands
    let mut ok = NewArgs::bare("item", "minted stated");
    ok.status = Some("ready".into());
    ok.acceptance = vec!["it lands".into()];
    ops::new_node(&s, ok).unwrap();
}

#[test]
fn reserve_backstop_refuses_at_fire_and_unreadies() {
    let s = temp_store();
    // a pre-gate ready item with no acceptance (built shaped, forced by
    // hand to simulate legacy state the gate never saw)
    let it = ops::new_node(&s, NewArgs::bare("item", "legacy ready work")).unwrap();
    {
        let all = s.load_all().unwrap();
        let mut n = s.find(&all, &it.front.id).unwrap().clone();
        n.front.status = "ready".into();
        s.save(&n).unwrap();
    }
    // the dispatch station refuses, un-readies, and teaches return-to-design
    let err =
        ops::dispatch(&s, &it.front.id, vec!["src/**".into()], false, false, None, None, "geo", "t")
            .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("acceptance gate"), "got: {}", msg);
    assert!(msg.contains("UN-READIED"), "the demotion is loud: {}", msg);
    assert!(msg.contains("design"), "the dispatcher is taught return-to-design: {}", msg);
    assert!(
        !msg.contains("acceptance+="),
        "the dispatcher is never taught to author — the pen stays with design: {}",
        msg
    );
    let all = s.load_all().unwrap();
    assert_eq!(
        s.find(&all, &it.front.id).unwrap().front.status,
        "shaped",
        "the refusal un-readies the item so the ready feed stays true"
    );
    // the demotion is logged with the gate named
    let log = s.read_log().unwrap();
    let ev = log
        .iter()
        .rev()
        .find(|e| {
            e.get("op").and_then(|v| v.as_str()) == Some("set")
                && e.get("node").and_then(|v| v.as_str()) == Some(it.front.id.as_str())
        })
        .expect("the demotion is logged");
    assert_eq!(ev.get("from_status").and_then(|v| v.as_str()), Some("ready"));
    assert!(
        ev.get("note").and_then(|v| v.as_str()).unwrap_or("").contains("acceptance gate"),
        "the logged demotion names the gate: {}",
        ev
    );
    // nothing else mutated: no lease, no brief event, no in-flight
    assert!(quarry::coord::load_leases(&s).is_empty(), "no lease on a refused fire");
    assert!(
        !log.iter().any(|e| e.get("op").and_then(|v| v.as_str()) == Some("brief")),
        "no brief event on a refused fire"
    );
    // the solo station refuses too, teaching the authoring command directly
    let all = s.load_all().unwrap();
    let node = s.find(&all, &it.front.id).unwrap().clone();
    let err = ops::acceptance_backstop(&s, &node, true).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("acceptance gate"), "got: {}", msg);
    assert!(msg.contains("acceptance+="), "the solo path is design-capable and taught: {}", msg);
    // a sketch never demotes — there is nothing to un-ready
    let sk = ops::new_node(&s, NewArgs::bare("item", "quiet sketch")).unwrap();
    let err =
        ops::dispatch(&s, &sk.front.id, vec!["src/**".into()], false, false, None, None, "geo", "t")
            .unwrap_err();
    assert!(!err.to_string().contains("UN-READIED"), "got: {}", err);
    let all = s.load_all().unwrap();
    assert_eq!(s.find(&all, &sk.front.id).unwrap().front.status, "sketch");
    // an acceptance-carrying item passes the backstop untouched
    let mut ok = NewArgs::bare("item", "stated work");
    ok.acceptance = vec!["it lands".into()];
    let ok = ops::new_node(&s, ok).unwrap();
    ops::acceptance_backstop(&s, &ok, false).unwrap();
    ops::acceptance_backstop(&s, &ok, true).unwrap();
}

#[test]
fn brief_tripwire_refuses_and_names_the_breach() {
    let s = temp_store();
    let it = ops::new_node(&s, NewArgs::bare("item", "contract-less work")).unwrap();
    let err = quarry::render::brief(&s, &it.front.id).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("acceptance gate breach"), "the breached invariant is named: {}", msg);
    assert!(msg.contains("dc-p6z4"), "the ruling is named: {}", msg);
    assert!(
        !msg.contains("acceptance+="),
        "the agent is never prompted to self-author — the old branch is gone: {}",
        msg
    );
}

#[test]
fn awaiting_acceptance_derives_at_any_status_and_clears_by_authoring() {
    let s = temp_store();
    let sketch = ops::new_node(&s, NewArgs::bare("item", "quiet sketch")).unwrap();
    let shaped = ops::new_node(&s, NewArgs::bare("item", "shaped and unstated")).unwrap();
    ops::set(&s, &shaped.front.id, &["status=shaped".to_string()], None).unwrap();
    // a breach specimen forced by hand: ready with no acceptance
    let breach = ops::new_node(&s, NewArgs::bare("item", "breach specimen")).unwrap();
    {
        let all = s.load_all().unwrap();
        let mut n = s.find(&all, &breach.front.id).unwrap().clone();
        n.front.status = "ready".into();
        s.save(&n).unwrap();
    }
    // settled and stated items stay out; threads are not items
    let mut stated = NewArgs::bare("item", "stated work");
    stated.acceptance = vec!["it lands".into()];
    ops::new_node(&s, stated).unwrap();
    let mut done = NewArgs::bare("item", "landed long ago");
    done.status = Some("done".into());
    ops::new_node(&s, done).unwrap();
    let all = s.load_all().unwrap();
    let aw = queries::awaiting_acceptance(&all);
    let ids: Vec<&str> = aw.iter().map(|n| n.front.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![breach.front.id.as_str(), shaped.front.id.as_str(), sketch.front.id.as_str()],
        "any live status, breach first, nothing stored: {:?}",
        ids
    );
    // authoring acceptance clears it by construction — no flag to unset
    ops::set(&s, &shaped.front.id, &["acceptance+=the shape lands".to_string()], None).unwrap();
    let all = s.load_all().unwrap();
    let aw = queries::awaiting_acceptance(&all);
    assert!(
        !aw.iter().any(|n| n.front.id == shaped.front.id),
        "authoring acceptance clears the derived state"
    );
}

#[test]
fn design_wake_counts_shaped_and_acceptance_less() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            // Isolate the identity pin file (dc-g5x5): without this, a join
            // in one run plants a REAL user-level pin that redirects the
            // next run's identically-keyed identity to a dead temp store.
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    // a shaped-and-acceptance-less item in purview: the gate's real waiter
    let mut it = NewArgs::bare("item", "held at the gate");
    it.status = Some("shaped".into());
    it.about = vec![area.front.id.clone()];
    let it = ops::new_node(&s, it).unwrap();
    // a sketch stays unpressured — early absence is legitimate
    let mut sk = NewArgs::bare("item", "early absence");
    sk.about = vec![area.front.id.clone()];
    ops::new_node(&s, sk).unwrap();
    quarry::coord::save_session(&s, "design", vec![area.front.id.clone()], Some("design".into()), None, false)
        .unwrap();
    let out = run(&[("QUARRY_SESSION", "design")], &["session", "resume"]);
    assert!(out.status.success(), "resume: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("awaiting acceptance: 1 shaped item(s)"),
        "the design wake counts shaped-and-acceptance-less, sketches quiet: {}",
        stdout
    );
    assert!(
        stdout.contains("q query awaiting-acceptance"),
        "the count teaches the derived query: {}",
        stdout
    );
    // the dispatch-kind wake omits the pressure — shaping is not a
    // dispatcher's to settle (the owed-threads omission, dc-wngq)
    quarry::coord::save_session(&s, "disp", vec![area.front.id.clone()], Some("dispatch".into()), None, false)
        .unwrap();
    let out2 = run(&[("QUARRY_SESSION", "disp")], &["session", "resume"]);
    assert!(out2.status.success(), "resume: {}", String::from_utf8_lossy(&out2.stderr));
    assert!(
        !String::from_utf8_lossy(&out2.stdout).contains("awaiting acceptance"),
        "the dispatch shape carries no shaping pressure"
    );
    // authoring acceptance clears the count by construction
    ops::set(&s, &it.front.id, &["acceptance+=it lands".to_string()], None).unwrap();
    let out3 = run(&[("QUARRY_SESSION", "design")], &["session", "resume"]);
    assert!(out3.status.success(), "resume: {}", String::from_utf8_lossy(&out3.stderr));
    assert!(
        !String::from_utf8_lossy(&out3.stdout).contains("awaiting acceptance"),
        "authoring acceptance clears the wake count"
    );
}

#[test]
fn design_wake_counts_load_bearing_unassayed() {
    // The assay's pressure surface (dc-drr6, it-fwn3): the design-kind
    // wake counts load-bearing unassayed claims beside owed threads, the
    // query command in hand. The set is waiter-earned (the dc-p6z4 waiter
    // test): zero-holds unassayed never counts — it stays reachable
    // through q query load alone.
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            // Isolate the identity pin file (dc-g5x5): without this, a join
            // in one run plants a REAL user-level pin that redirects the
            // next run's identically-keyed identity to a dead temp store.
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut b1 = NewArgs::bare("item", "built thing");
    b1.status = Some("done".into());
    let b1 = ops::new_node(&s, b1).unwrap();
    let mk = |title: &str| {
        ops::claim(
            &s,
            title,
            None,
            Some("vein".into()),
            vec![area.front.id.clone()],
            None,
            None,
            Some("user".into()),
            None,
        )
        .unwrap()
    };
    // a load-bearing unassayed claim: a build stands on it — a real waiter
    let heavy = mk("`heavy-lode`: a build stands on it");
    ops::link(&s, &heavy.front.id, "supports", &b1.front.id, false, None).unwrap();
    // zero-holds unassayed: no waiter, no wake line — the boundary
    mk("`light-lode`: nothing stands on it");
    quarry::coord::save_session(&s, "design", vec![area.front.id.clone()], Some("design".into()), None, false)
        .unwrap();
    let out = run(&[("QUARRY_SESSION", "design")], &["session", "resume"]);
    assert!(out.status.success(), "resume: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("load-bearing unassayed: 1 claim(s)"),
        "the design wake counts the waiter-earned set only — zero-holds stays quiet: {}",
        stdout
    );
    assert!(
        stdout.contains("q query load"),
        "the count carries the query command in hand: {}",
        stdout
    );
    // the dispatch-kind wake omits the pressure — judging a claim is a
    // design-session act (the owed-threads omission, dc-wngq)
    quarry::coord::save_session(&s, "disp", vec![area.front.id.clone()], Some("dispatch".into()), None, false)
        .unwrap();
    let out2 = run(&[("QUARRY_SESSION", "disp")], &["session", "resume"]);
    assert!(out2.status.success(), "resume: {}", String::from_utf8_lossy(&out2.stderr));
    assert!(
        !String::from_utf8_lossy(&out2.stdout).contains("load-bearing unassayed"),
        "the dispatch shape carries no assay pressure"
    );
    // the assay clears the count by construction — ratified leaves the set
    ops::set(&s, &heavy.front.id, &["status=ratified".to_string()], None).unwrap();
    let out3 = run(&[("QUARRY_SESSION", "design")], &["session", "resume"]);
    assert!(out3.status.success(), "resume: {}", String::from_utf8_lossy(&out3.stderr));
    assert!(
        !String::from_utf8_lossy(&out3.stdout).contains("load-bearing unassayed"),
        "an assayed claim carries no wake pressure"
    );
}

#[test]
fn promotion_ranking_is_fully_derived() {
    // The dry-feed ranking (dc-dty5): no stored priority anywhere —
    // defects first (the standing ruling reaches into any list), then
    // distance to the feed (shaped with acceptance authored, then shaped,
    // then sketch), then the derived holds weight, age as tiebreak only.
    let s = temp_store();
    let set_created = |id: &str, ts: &str| {
        let all = s.load_all().unwrap();
        let mut n = s.find(&all, id).unwrap().clone();
        n.front.created = ts.into();
        s.save(&n).unwrap();
    };
    // a defect in sketch outranks everything nearer the feed
    let mut bug = NewArgs::bare("item", "bitter defect");
    bug.kind = Some("bug".into());
    let bug = ops::new_node(&s, bug).unwrap();
    // shaped with acceptance and a live thread standing on it: weight 1
    let mut heavy = NewArgs::bare("item", "heavy shaped");
    heavy.status = Some("shaped".into());
    heavy.acceptance = vec!["it lands".into()];
    let heavy = ops::new_node(&s, heavy).unwrap();
    let th = ops::new_node(&s, NewArgs::bare("thread", "who waits")).unwrap();
    ops::link(&s, &th.front.id, "depends-on", &heavy.front.id, false, None).unwrap();
    // shaped with acceptance, nothing standing on it: weight 0
    let mut light = NewArgs::bare("item", "light shaped");
    light.status = Some("shaped".into());
    light.acceptance = vec!["it lands".into()];
    let light = ops::new_node(&s, light).unwrap();
    // shaped without acceptance sits behind the stated pair
    let mut bare = NewArgs::bare("item", "bare shaped");
    bare.status = Some("shaped".into());
    let bare = ops::new_node(&s, bare).unwrap();
    // two sketches tie on every key but age: oldest first
    let old = ops::new_node(&s, NewArgs::bare("item", "weathered sketch")).unwrap();
    set_created(&old.front.id, "2026-08-01T00:00:00Z");
    let fresh = ops::new_node(&s, NewArgs::bare("item", "fresh sketch")).unwrap();
    set_created(&fresh.front.id, "2026-08-05T00:00:00Z");
    // ready, done, and archived never join the pool
    let mut rdy = NewArgs::bare("item", "already fed");
    rdy.status = Some("ready".into());
    rdy.acceptance = vec!["it lands".into()];
    ops::new_node(&s, rdy).unwrap();
    let mut done = NewArgs::bare("item", "landed long ago");
    done.status = Some("done".into());
    ops::new_node(&s, done).unwrap();
    let arch = ops::new_node(&s, NewArgs::bare("item", "cold storage")).unwrap();
    {
        let all = s.load_all().unwrap();
        let mut n = s.find(&all, &arch.front.id).unwrap().clone();
        n.front.archived = true;
        s.save(&n).unwrap();
    }
    let all = s.load_all().unwrap();
    let got: Vec<&str> =
        queries::promotion_candidates(&all).iter().map(|n| n.front.id.as_str()).collect();
    assert_eq!(
        got,
        vec![
            bug.front.id.as_str(),
            heavy.front.id.as_str(),
            light.front.id.as_str(),
            bare.front.id.as_str(),
            old.front.id.as_str(),
            fresh.front.id.as_str(),
        ],
        "defects, feed distance, holds weight, age — in that order: {:?}",
        got
    );
}

#[test]
fn design_wake_counts_defects_in_shaping() {
    // The standing-ruling earner (dc-dty5, dc-ygzz): the design wake
    // counts kind=bug items in shaping whenever nonzero, the query
    // command in hand; ready and in-flight bugs are already fed or under
    // repair and never count; zero renders nothing.
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "smithy")).unwrap();
    // a bug in shaping: the waitered set
    let mut bug = NewArgs::bare("item", "the anvil cracks");
    bug.kind = Some("bug".into());
    bug.status = Some("shaped".into());
    bug.about = vec![area.front.id.clone()];
    let bug = ops::new_node(&s, bug).unwrap();
    // a plain shaped item never counts as a defect
    let mut plain = NewArgs::bare("item", "a new bellows");
    plain.status = Some("shaped".into());
    plain.about = vec![area.front.id.clone()];
    ops::new_node(&s, plain).unwrap();
    // a bug already in ready is fed — out of the counted stratum
    let mut fed = NewArgs::bare("item", "the tongs slip");
    fed.kind = Some("bug".into());
    fed.status = Some("ready".into());
    fed.acceptance = vec!["the grip holds".into()];
    fed.about = vec![area.front.id.clone()];
    ops::new_node(&s, fed).unwrap();
    quarry::coord::save_session(&s, "design", vec![area.front.id.clone()], Some("design".into()), None, false)
        .unwrap();
    let out = run(&[("QUARRY_SESSION", "design")], &["session", "resume"]);
    assert!(out.status.success(), "resume: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("defects in shaping: 1 bug(s)"),
        "the wake counts the shaping stratum only — ready bugs are fed: {}",
        stdout
    );
    assert!(
        stdout.contains("q query defects"),
        "the count carries the query command in hand: {}",
        stdout
    );
    // the dispatch-kind wake omits the pressure (the owed-threads
    // omission, dc-wngq)
    quarry::coord::save_session(&s, "disp", vec![area.front.id.clone()], Some("dispatch".into()), None, false)
        .unwrap();
    let out2 = run(&[("QUARRY_SESSION", "disp")], &["session", "resume"]);
    assert!(out2.status.success(), "resume: {}", String::from_utf8_lossy(&out2.stderr));
    assert!(
        !String::from_utf8_lossy(&out2.stdout).contains("defects in shaping"),
        "the dispatch shape carries no defect pressure"
    );
    // fixing the bug clears the count — zero renders nothing
    ops::set(&s, &bug.front.id, &["status=done".to_string()], None).unwrap();
    let out3 = run(&[("QUARRY_SESSION", "design")], &["session", "resume"]);
    assert!(out3.status.success(), "resume: {}", String::from_utf8_lossy(&out3.stderr));
    assert!(
        !String::from_utf8_lossy(&out3.stdout).contains("defects in shaping"),
        "a fixed defect carries no wake pressure"
    );
}

#[test]
fn design_wake_states_the_dry_feed_and_any_ready_item_silences_it() {
    // The hunger earner (dc-dty5): ready empty while shaping holds work
    // draws the hunger line with the top candidates ranked derived; any
    // item in ready and the line is absent entirely. The predicate is
    // feed-based, never session-based.
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "granary")).unwrap();
    // shaped with acceptance: nearest the feed, first raised
    let mut near = NewArgs::bare("item", "first to the trough");
    near.status = Some("shaped".into());
    near.acceptance = vec!["it lands".into()];
    near.about = vec![area.front.id.clone()];
    let near = ops::new_node(&s, near).unwrap();
    // a sketch trails it
    let mut far = NewArgs::bare("item", "still forming");
    far.about = vec![area.front.id.clone()];
    ops::new_node(&s, far).unwrap();
    quarry::coord::save_session(&s, "design", vec![area.front.id.clone()], Some("design".into()), None, false)
        .unwrap();
    let out = run(&[("QUARRY_SESSION", "design")], &["session", "resume"]);
    assert!(out.status.success(), "resume: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("the feed is dry"),
        "ready empty while shaping holds work states the hunger: {}",
        stdout
    );
    assert!(
        stdout.contains("q query shaping --mine"),
        "the hunger line carries the pool's query in hand: {}",
        stdout
    );
    // ranked within the hunger section: feed distance orders the pair
    let tail = &stdout[stdout.find("the feed is dry").unwrap()..];
    let a = tail.find("first to the trough").expect("the shaped candidate is raised");
    let b = tail.find("still forming").expect("the sketch candidate is raised");
    assert!(a < b, "shaped-with-acceptance outranks the sketch: {}", tail);
    // the dispatch-kind wake omits the pressure (the owed-threads
    // omission, dc-wngq)
    quarry::coord::save_session(&s, "disp", vec![area.front.id.clone()], Some("dispatch".into()), None, false)
        .unwrap();
    let out2 = run(&[("QUARRY_SESSION", "disp")], &["session", "resume"]);
    assert!(out2.status.success(), "resume: {}", String::from_utf8_lossy(&out2.stderr));
    assert!(
        !String::from_utf8_lossy(&out2.stdout).contains("the feed is dry"),
        "the dispatch shape carries no hunger line"
    );
    // one item promoted to ready and the line is absent entirely — the
    // sketch still shapes, but the feed is no longer dry
    ops::set(&s, &near.front.id, &["status=ready".to_string()], None).unwrap();
    let out3 = run(&[("QUARRY_SESSION", "design")], &["session", "resume"]);
    assert!(out3.status.success(), "resume: {}", String::from_utf8_lossy(&out3.stderr));
    assert!(
        !String::from_utf8_lossy(&out3.stdout).contains("the feed is dry"),
        "anything in ready silences the hunger line"
    );
}

// ── ready teaches the leans (dc-ez67, dc-grrb, it-6349): un-edged mentions
// enumerate with the why; refused links name the legal rels ─────────────

#[test]
fn ready_flip_enumerates_unleaned_citations() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut d1 = NewArgs::bare("decision", "already leaned on");
    d1.provenance = Some("user".into());
    let d1 = ops::new_node(&s, d1).unwrap();
    let mut d2 = NewArgs::bare("decision", "cited, never edged");
    d2.provenance = Some("user".into());
    let d2 = ops::new_node(&s, d2).unwrap();
    let mut d3 = NewArgs::bare("decision", "superseded stratum");
    d3.provenance = Some("user".into());
    let d3 = ops::new_node(&s, d3).unwrap();
    let mut d4 = NewArgs::bare("decision", "the successor");
    d4.provenance = Some("user".into());
    let d4 = ops::new_node(&s, d4).unwrap();
    ops::link(&s, &d4.front.id, "supersedes", &d3.front.id, false, None).unwrap();
    let mk_claim = |text: &str| {
        ops::claim(
            &s,
            text, None, None,
            vec![area.front.id.clone()],
            None,
            None,
            Some("user".into()),
            None,
        )
        .unwrap()
    };
    let c1 = mk_claim("`halo-bound`: the halo is bounded");
    let c2 = mk_claim("`ring-diff`: rings difference cleanly");
    let th = ops::new_node(&s, NewArgs::bare("thread", "open question")).unwrap();
    // the item's body cites all of them; only d2 and c1 are un-edged
    let mut it = NewArgs::bare("item", "the work at hand");
    it.acceptance = vec!["it lands".into()];
    it.body = format!(
        "Builds within {} and {}, from {} and {}; {} is history, {} is open.",
        d1.front.id, d2.front.id, c1.front.id, c2.front.id, d3.front.id, th.front.id
    );
    let it = ops::new_node(&s, it).unwrap();
    ops::link(&s, &it.front.id, "depends-on", &d1.front.id, false, None).unwrap();
    ops::link(&s, &c2.front.id, "supports", &it.front.id, false, None).unwrap();
    let all = s.load_all().unwrap();
    let item = s.find(&all, &it.front.id).unwrap();
    let unleaned = queries::unleaned_citations(&all, item);
    let got: Vec<(&str, &str)> =
        unleaned.iter().map(|(n, cmd)| (n.front.id.as_str(), cmd.as_str())).collect();
    let dep = format!("q link {} depends-on {}", it.front.id, d2.front.id);
    let sup = format!("q link {} supports {}", c1.front.id, it.front.id);
    assert_eq!(
        got,
        vec![(d2.front.id.as_str(), dep.as_str()), (c1.front.id.as_str(), sup.as_str())],
        "un-edged citations only, in citation order, each with its ready-made command — \
         edged (either direction), superseded, and non-decision-non-claim citations stay out: {:?}",
        got
    );
}

#[test]
fn lean_prompt_fires_at_ready_teaches_why_and_closes_open() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |args: &[&str]| {
        let out = std::process::Command::new(q)
            .current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "q {:?}: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let mut d = NewArgs::bare("decision", "the governing ruling");
    d.provenance = Some("user".into());
    let d = ops::new_node(&s, d).unwrap();
    let mut it = NewArgs::bare("item", "work citing its ruling");
    it.acceptance = vec!["it lands".into()];
    it.body = format!("Built to the shape {} ruled.", d.front.id);
    let it = ops::new_node(&s, it).unwrap();
    // the flip enumerates, teaches the why, and closes open-ended
    let out = run(&["set", &it.front.id, "status=ready"]);
    assert!(
        out.contains("a mention references, an edge leans"),
        "the prompt teaches the doctrine: {}",
        out
    );
    assert!(
        out.contains("READ-FIRST"),
        "the why names what an edge does — leaned nodes pin the brief: {}",
        out
    );
    let cmd = format!("q link {} depends-on {}", it.front.id, d.front.id);
    assert!(out.contains(&cmd), "each line carries its ready-made link command: {}", out);
    assert!(
        out.contains("what else does the item stand on"),
        "the close is open-ended — reflection past the enumeration: {}",
        out
    );
    // never a gate: the flip stood
    let all = s.load_all().unwrap();
    assert_eq!(s.find(&all, &it.front.id).unwrap().front.status, "ready");
    // once the lean is recorded, the prompt has nothing to say
    run(&["link", &it.front.id, "depends-on", &d.front.id]);
    run(&["set", &it.front.id, "status=shaped"]);
    let out2 = run(&["set", &it.front.id, "status=ready"]);
    assert!(
        !out2.contains("an edge leans"),
        "an edged citation never re-prompts — presence, not nagging: {}",
        out2
    );
    // mint-to-ready is the other construction path: the prompt fires there too
    let mut d2 = NewArgs::bare("decision", "second ruling");
    d2.provenance = Some("user".into());
    let d2 = ops::new_node(&s, d2).unwrap();
    let out3 = run(&[
        "new",
        "item",
        "minted hot with a citation",
        "--status",
        "ready",
        "--acceptance",
        "it lands",
        "--body",
        &format!("Stands on {}.", d2.front.id),
    ]);
    assert!(
        out3.contains("a mention references, an edge leans"),
        "no construction path reaches ready untaught: {}",
        out3
    );
}

#[test]
fn link_refusal_names_legal_rels_for_the_pair() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut d = NewArgs::bare("decision", "a ruling");
    d.provenance = Some("user".into());
    let d = ops::new_node(&s, d).unwrap();
    let it = ops::new_node(&s, NewArgs::bare("item", "some work")).unwrap();
    let th = ops::new_node(&s, NewArgs::bare("thread", "a question")).unwrap();
    let c = ops::claim(
        &s,
        "the halo is bounded", None, None,
        vec![area.front.id.clone()],
        None,
        None,
        Some("user".into()),
        None,
    )
    .unwrap();
    // the near-miss shape (it-33bb): item builds-on decision refused — the
    // refusal now names the legal rel for the exact pair, a one-step redirect
    let err = ops::link(&s, &it.front.id, "builds-on", &d.front.id, false, None).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("legal rels for item → decision: depends-on"),
        "the pair's legal rels are named: {}",
        msg
    );
    assert!(msg.contains("builder → built-upon"), "the attempted rel's shapes still teach: {}", msg);
    assert!(msg.contains("edge matrix"), "the guide pointer stands: {}", msg);
    // no forward rel exists item → claim; the lean runs the other way
    let err = ops::link(&s, &it.front.id, "depends-on", &c.front.id, false, None).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("the lean runs the other way: claim -[supports]-> item"),
        "the reverse redirect is named: {}",
        msg
    );
    // nothing joins thread → claim in either direction: the mention IS the channel
    let err = ops::link(&s, &th.front.id, "supports", &c.front.id, false, None).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("no rel joins thread → claim in either direction"),
        "a true dead end says so: {}",
        msg
    );
    assert!(
        msg.contains("cite the id in the body"),
        "mention-only is taught as correct, not a downgrade: {}",
        msg
    );
}

// ── the vein prompt at land time (it-pgn9, dc-grrb shape): kindless
// backtick-titled mints draw the species question; harvest asks before
// ratification passes them by ───────────────────────────────────────────

#[test]
fn backtick_titled_reads_the_leading_name_under_the_deliberate_floor() {
    use quarry::queries::backtick_titled;
    assert!(backtick_titled("`find-tiers`: word and id hits lead"), "a registered name leads");
    assert!(backtick_titled("`cli`: the two-char floor admits deliberate short names"));
    assert!(!backtick_titled("plain prose title"), "no mark, no prompt");
    assert!(!backtick_titled("word hits lead in `find-tiers` mid-title"), "the name must lead");
    assert!(!backtick_titled("`x`: below the deliberate-name floor"), "the dc-qvtz floor holds");
    assert!(!backtick_titled("`unclosed name runs off"), "an unclosed span is prose");
}

#[test]
fn harvest_asks_the_kindless_backtick_mints_before_ratification_passes_them_by() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "vein prompt pass");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the ask lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "geo", "t")
        .unwrap();
    let mk = |title: &str, kind: Option<&str>| {
        ops::claim(
            &s,
            title,
            None,
            kind.map(String::from),
            vec![area.front.id.clone()],
            None,
            None,
            Some("user".into()),
            None,
        )
        .unwrap()
    };
    let unkinded = mk("`geo-tiers`: word hits lead, loose trails", None);
    let plain = mk("strata scans read bottom-up", None);
    let kinded = mk("`geo-pass`: emits layered strata", Some("vein"));
    let unstamped = mk("`geo-cache`: caches strata", None);
    for c in [&unkinded, &plain, &kinded] {
        s.log_event(serde_json::json!({
            "ts": Store::now(), "node": c.front.id, "v": 1, "op": "create", "type": "claim",
            "actor": "t", "dispatch": it.front.id
        }))
        .unwrap();
    }
    // the ask teaches verbatim (dc-dsdm channel), settle command in hand,
    // before the landing lines — never a gate
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(
        h.contains("1 kindless mint(s) under this badge lead with a registered name - the assay ladder rides species, so landing now passes these by asserted. Settle each, then land:"),
        "the ask teaches verbatim: {}", h
    );
    assert!(
        h.contains(&format!("settle: q set {} kind=vein (or kind=feature)", unkinded.front.id)),
        "the settle command is in hand: {}", h
    );
    assert!(
        !h.contains(&format!("settle: q set {}", plain.front.id))
            && !h.contains(&format!("settle: q set {}", unstamped.front.id))
            && !h.contains(&format!("settle: q set {}", kinded.front.id)),
        "plain-titled, unstamped, and kinded mints draw no ask: {}", h
    );
    assert!(
        h.find("kindless mint(s)").unwrap() < h.find("LANDING").unwrap(),
        "the ask lands before the landing lines: {}", h
    );
    // the ladder alone never reaches a kindless mint — the ask is the channel
    let assay = ops::ratify_landing(&s, &it.front.id, Some("geo")).unwrap().unwrap();
    assert!(assay.ratified.iter().all(|n| n.front.id != unkinded.front.id));
    let all = s.load_all().unwrap();
    assert_eq!(s.find(&all, &unkinded.front.id).unwrap().front.status, "asserted");
    // settle the species as the ask teaches; the re-harvest routes the mint
    // onto the assay ladder and the ask falls silent
    ops::set(&s, &unkinded.front.id, &["kind=vein".to_string()], None).unwrap();
    let h2 = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(!h2.contains("kindless mint(s)"), "settled: the ask is gone: {}", h2);
    assert!(
        h2.contains(&quarry::framings::assay_harvest_line(1)),
        "the settled mint is now the assay's to name: {}", h2
    );
    // the mint prompt's verbiage ships pinned (dc-dsdm: never invented silently)
    assert_eq!(
        quarry::framings::VEIN_PROMPT,
        "this claim leads with a registered name - a mechanism or mandate read off landed code (--kind vein), or the receipt of a landed capability (--kind feature)? Kindless claims never ride the assay ladder."
    );
}

// ── the witness pen (dc-mpg8) ──────────────────────────────────────────────

/// The line check: from a witness seat (a session whose REGISTERED kind is
/// not design) acceptance is transcription — a backticked register name in
/// the line refuses and teaches the plea channel; a plain negation passes
/// and is MARKED at authoring. Design seats and the contested kindless
/// middle keep the free pen (dc-p6z4's design-capable default; the
/// kindless call is queued for ruling).
#[test]
fn witness_pen_refuses_register_names_and_marks_the_rest() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    quarry::coord::save_session(&s, "disp", vec![area.front.id.clone()], Some("dispatch".into()), None, false)
        .unwrap();
    quarry::coord::save_session(&s, "design", vec![area.front.id.clone()], Some("design".into()), None, false)
        .unwrap();
    // a backticked register name refuses from the witness seat, at mint
    let out = run(
        &[("QUARRY_SESSION", "disp")],
        &["new", "item", "seen gap", "--acceptance", "lands `shiny-name`: the gap closes"],
    );
    assert!(!out.status.success(), "register name from a witness seat must refuse");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("witness pen (dc-mpg8)"), "names the pen: {}", err);
    assert!(err.contains("q new thread"), "teaches the plea channel: {}", err);
    // a plain transcription passes and is marked at authoring
    let out = run(
        &[("QUARRY_SESSION", "disp")],
        &["new", "item", "seen gap", "--acceptance", "the witnessed defect no longer reproduces"],
    );
    assert!(out.status.success(), "plain negation passes: {}", String::from_utf8_lossy(&out.stderr));
    let all = s.load_all().unwrap();
    let it = s.find(&all, "seen-gap").unwrap();
    assert_eq!(it.front.witness.len(), 1, "marked at authoring");
    let m = &it.front.witness[0];
    assert_eq!(m.by, "session:disp");
    assert_eq!(m.session.as_deref(), Some("disp"));
    assert_eq!(m.kind.as_deref(), Some("dispatch"));
    assert!(m.ratified.is_none(), "under review until the user's word");
    // the same refusal holds at acceptance+= on an existing item
    let out = run(
        &[("QUARRY_SESSION", "disp")],
        &["set", &it.front.id, "acceptance+=also lands `another-name` here"],
    );
    assert!(!out.status.success(), "register name refuses at set too");
    // a design seat authors freely, register names included, unmarked
    let out = run(
        &[("QUARRY_SESSION", "design")],
        &["new", "item", "designed work", "--acceptance", "lands `real-name`: the capability"],
    );
    assert!(out.status.success(), "design pen is free: {}", String::from_utf8_lossy(&out.stderr));
    let all = s.load_all().unwrap();
    let d = s.find(&all, "designed-work").unwrap();
    assert!(d.front.witness.is_empty(), "design authoring is never witness-marked");
    // the contested kindless middle keeps the design-capable default:
    // an unregistered session authors unmarked (queued for ruling)
    let out = run(
        &[("QUARRY_SESSION", "loose")],
        &["new", "item", "kindless work", "--acceptance", "lands `free`: kindless authoring"],
    );
    assert!(out.status.success(), "kindless keeps the free pen: {}", String::from_utf8_lossy(&out.stderr));
    let all = s.load_all().unwrap();
    assert!(s.find(&all, "kindless-work").unwrap().front.witness.is_empty());
}

/// The sequence check: a witness seat that met the acceptance gate's
/// refusal on an item may not then author that item's contract — the
/// it-hapc self-authorization class refuses by construction, while the
/// design seat's author-after-refusal flow (the gate's own teaching)
/// stays free.
#[test]
fn witness_pen_refuses_the_self_authorization_sequence() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    quarry::coord::save_session(&s, "disp", vec![area.front.id.clone()], Some("dispatch".into()), None, false)
        .unwrap();
    quarry::coord::save_session(&s, "design", vec![area.front.id.clone()], Some("design".into()), None, false)
        .unwrap();
    let mut it = NewArgs::bare("item", "contract-less");
    it.status = Some("shaped".into());
    it.about = vec![area.front.id.clone()];
    let it = ops::new_node(&s, it).unwrap();
    // the dispatcher fires it and meets the gate — the refusal is logged
    let out = run(
        &[("QUARRY_SESSION", "disp")],
        &["dispatch", &it.front.id, "--files", "src/**"],
    );
    assert!(!out.status.success(), "the gate refuses the contract-less fire");
    let log = s.read_log().unwrap();
    assert!(
        log.iter().any(|ev| ev.get("op").and_then(|v| v.as_str()) == Some("gate-refusal")
            && ev.get("node").and_then(|v| v.as_str()) == Some(it.front.id.as_str())),
        "the gate refusal is the pen's memory"
    );
    // the same seat may not now author the contract — refused by sequence
    let out = run(
        &[("QUARRY_SESSION", "disp")],
        &["set", &it.front.id, "acceptance+=the gap closes"],
    );
    assert!(!out.status.success(), "author-after-refusal is the sequence the pen stops");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("witness pen (dc-mpg8)") && err.contains("sequence"), "names the sequence: {}", err);
    assert!(err.contains("thread"), "teaches the plea channel: {}", err);
    // the design seat authors after the same refusal — the taught flow
    let out = run(
        &[("QUARRY_SESSION", "design")],
        &["set", &it.front.id, "acceptance+=the gap closes"],
    );
    assert!(out.status.success(), "design authors after the refusal: {}", String::from_utf8_lossy(&out.stderr));
}

/// The executor check: the authoring badge cannot join or solo-build the
/// item it authored — author is never executor, mechanically. The refused
/// join spends nothing: the single-use token stays live for a fresh agent.
#[test]
fn witness_author_never_executes_by_join_or_solo() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    quarry::coord::save_session(&s, "disp", vec![area.front.id.clone()], Some("dispatch".into()), None, false)
        .unwrap();
    // the area first-touch gate is not under test — record the read
    quarry::coord::record_area_read(&s, "disp", &area.front.id);
    // the witness seat files the item with its transcribed contract
    let out = run(
        &[("QUARRY_SESSION", "disp")],
        &["new", "item", "witnessed defect", "--about", &area.front.id, "--acceptance", "the witnessed defect no longer reproduces", "--status", "ready"],
    );
    assert!(out.status.success(), "filing: {}", String::from_utf8_lossy(&out.stderr));
    let all = s.load_all().unwrap();
    let it = s.find(&all, "witnessed-defect").unwrap().clone();
    // solo-build refuses: brief lands, the reserve refusal is the pen's
    let out = run(&[("QUARRY_SESSION", "disp")], &["brief", &it.front.id]);
    assert!(out.status.success(), "brief: {}", String::from_utf8_lossy(&out.stderr));
    let out = run(
        &[("QUARRY_SESSION", "disp")],
        &["reserve", &it.front.id, "--files", "src/**"],
    );
    assert!(!out.status.success(), "the authoring seat cannot solo-build");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("author is never executor"), "names the ban: {}", err);
    // dispatching it stays legal — author fires, a fresh mind executes
    let out = run(
        &[("QUARRY_SESSION", "disp")],
        &["dispatch", &it.front.id, "--files", "src/**"],
    );
    assert!(out.status.success(), "the author may fire: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let token = stdout
        .lines()
        .find_map(|l| l.split("q join ").nth(1).map(|t| t.split_whitespace().next().unwrap().to_string()))
        .expect("spawn line carries the token");
    // the authoring identity's join refuses WITHOUT spending the token
    let out = run(&[("QUARRY_SESSION", "disp")], &["join", &token]);
    assert!(!out.status.success(), "the authoring badge cannot join");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("author is never executor"),
        "join refusal names the ban: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // a fresh agent joins on the same token — the refusal consumed nothing
    let out = run(
        &[("QUARRY_SESSION", "disp"), ("QUARRY_AGENT", "ag-9")],
        &["join", &token],
    );
    assert!(out.status.success(), "a fresh mind joins: {}", String::from_utf8_lossy(&out.stderr));
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("agent:ag-9"),
        "bound to the fresh identity: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// The review channel: witness marks count on the design wake beside owed
/// threads (the dispatch wake omits them), the q witness surfaces list and
/// show them, and ONLY the user's word clears a flag — ratified marks stay
/// as record. The manual mark road transcribes a pre-pen line's true seat
/// and can only add review, never clear it.
#[test]
fn witness_flags_ride_the_design_wake_until_user_ratified() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    quarry::coord::save_session(&s, "disp", vec![area.front.id.clone()], Some("dispatch".into()), None, false)
        .unwrap();
    quarry::coord::save_session(&s, "design", vec![area.front.id.clone()], Some("design".into()), None, false)
        .unwrap();
    // the area first-touch gate is not under test — record the read
    quarry::coord::record_area_read(&s, "disp", &area.front.id);
    let out = run(
        &[("QUARRY_SESSION", "disp")],
        &["new", "item", "witnessed defect", "--about", &area.front.id, "--acceptance", "the witnessed defect no longer reproduces"],
    );
    assert!(out.status.success(), "filing: {}", String::from_utf8_lossy(&out.stderr));
    let all = s.load_all().unwrap();
    let it = s.find(&all, "witnessed-defect").unwrap().clone();
    // the design wake counts the channel beside owed threads
    let out = run(&[("QUARRY_SESSION", "design")], &["session", "resume"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("witness-authored acceptance under review: 1 line(s)"),
        "the design wake carries the review channel: {}",
        stdout
    );
    assert!(stdout.contains("(q witness)"), "the count teaches the pull surface: {}", stdout);
    // the dispatch wake omits it — the authoring seat is never the review
    // surface (dc-mpg8: the user attends one liaison)
    let out = run(&[("QUARRY_SESSION", "disp")], &["session", "resume"]);
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("witness-authored acceptance under review"),
        "the authoring seat's wake never reviews its own pen"
    );
    // the channel lists the line; ratification demands the user's word
    let out = run(&[], &["witness"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("the witnessed defect no longer reproduces") && stdout.contains(&it.front.id));
    let out = run(&[], &["witness", &it.front.id, "--ratify"]);
    assert!(!out.status.success(), "no --by user, no clear");
    let out = run(&[], &["witness", &it.front.id, "--ratify", "--by", "assistant"]);
    assert!(!out.status.success(), "only the user's word clears the flag");
    let out = run(&[], &["witness", &it.front.id, "--ratify", "--by", "user"]);
    assert!(out.status.success(), "the user's word lands: {}", String::from_utf8_lossy(&out.stderr));
    let all = s.load_all().unwrap();
    let m = &s.find(&all, &it.front.id).unwrap().front.witness[0];
    assert_eq!(m.ratified.as_ref().map(|r| r.by.as_str()), Some("user"), "the mark stays as record, stamped");
    let out = run(&[("QUARRY_SESSION", "design")], &["session", "resume"]);
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("witness-authored acceptance under review"),
        "ratified lines leave the wake"
    );
    let out = run(&[], &["witness"]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("the witness channel is clear"));
    // the manual mark road: a pre-pen line marks with its true seat
    let mut pre = NewArgs::bare("item", "pre-pen filing");
    pre.about = vec![area.front.id.clone()];
    pre.acceptance = vec!["an old transcribed negation".into()];
    let pre = ops::new_node(&s, pre).unwrap();
    let out = run(
        &[],
        &["witness", &pre.front.id, "--mark", "an old transcribed negation", "--author-session", "disp"],
    );
    assert!(out.status.success(), "the seed road marks: {}", String::from_utf8_lossy(&out.stderr));
    let out = run(
        &[],
        &["witness", &pre.front.id, "--mark", "an old transcribed negation", "--author-session", "disp"],
    );
    assert!(!out.status.success(), "double-mark refuses");
    let out = run(
        &[],
        &["witness", &pre.front.id, "--mark", "no such line", "--author-session", "disp"],
    );
    assert!(!out.status.success(), "the mark rides the exact line");
    let all = s.load_all().unwrap();
    let m = &s.find(&all, &pre.front.id).unwrap().front.witness[0];
    assert_eq!(m.by, "session:disp");
    assert_eq!(m.kind.as_deref(), Some("dispatch"), "the seat's kind transcribed from the registry");
    let out = run(&[], &["witness"]);
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("an old transcribed negation"),
        "the seeded line enters the channel"
    );
}

// ── acceptance change verbs (it-ds6b): removal by line, replace in one act ──

/// The removal resolves forgiving and records faithful: the exact line
/// wins outright, any substring matching exactly one line resolves, and
/// zero or multiple matches refuse listing candidates — never a silent
/// no-op. The log stores the FULL resolved line removed, never what was
/// typed, and replace — both fields in one q set — is one act, one log
/// event, one bump: the contract is content (unlike unlink, the item
/// bumps).
#[test]
fn acceptance_removal_resolves_exact_or_unique_substring_and_refuses_ambiguity() {
    let s = temp_store();
    let mut it = NewArgs::bare("item", "layered contract");
    it.acceptance = vec![
        "the pass lands".into(),
        "the clip lands".into(),
        "walls mapped".into(),
    ];
    let it = ops::new_node(&s, it).unwrap();
    // a unique substring resolves; the echo and log carry the full line
    let o = ops::set(&s, &it.front.id, &["acceptance-=walls".to_string()], None).unwrap();
    assert_eq!(o.removed, vec!["walls mapped".to_string()], "the resolved full line, never the typed fragment");
    assert_eq!(o.node.front.acceptance.len(), 2);
    assert!(o.demoted_from.is_none(), "a sketch item has no readiness to lose");
    let ev = s
        .read_log()
        .unwrap()
        .into_iter()
        .filter(|e| {
            e.get("op").and_then(|v| v.as_str()) == Some("set")
                && e.get("node").and_then(|v| v.as_str()) == Some(it.front.id.as_str())
        })
        .next_back()
        .unwrap();
    let fields: Vec<String> = ev["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(
        fields.contains(&"acceptance-=walls mapped".to_string()),
        "the log stores the resolved line, never what was typed: {:?}",
        fields
    );
    assert_eq!(
        ev["removed_acceptance"].as_array().unwrap()[0].as_str().unwrap(),
        "walls mapped"
    );
    // ambiguity refuses, listing every candidate — never a silent no-op
    let err = ops::set(&s, &it.front.id, &["acceptance-=lands".to_string()], None).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("ambiguous"), "got: {}", msg);
    assert!(
        msg.contains("the pass lands") && msg.contains("the clip lands"),
        "candidates listed: {}",
        msg
    );
    // zero matches refuse, listing what stands
    let err = ops::set(&s, &it.front.id, &["acceptance-=zzz".to_string()], None).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("matched no line"), "got: {}", msg);
    assert!(msg.contains("the pass lands"), "what stands is listed: {}", msg);
    let all = s.load_all().unwrap();
    let n = s.find(&all, &it.front.id).unwrap();
    assert_eq!(n.front.acceptance.len(), 2, "a refused removal mutates nothing");
    assert_eq!(n.front.v, 2, "no bump on refusal");
    // replace is both fields in one act — one log event, one bump
    let v_before = n.front.v;
    let o = ops::set(
        &s,
        &it.front.id,
        &[
            "acceptance-=clip".to_string(),
            "acceptance+=the clip re-lands cleanly".to_string(),
        ],
        None,
    )
    .unwrap();
    assert_eq!(o.removed, vec!["the clip lands".to_string()]);
    assert_eq!(o.node.front.v, v_before + 1, "one act, one bump");
    assert!(o.node.front.acceptance.contains(&"the clip re-lands cleanly".to_string()));
    assert!(!o.node.front.acceptance.contains(&"the clip lands".to_string()));
    // the exact line wins outright even when it substrings another
    let mut twin = NewArgs::bare("item", "twin lines");
    twin.acceptance = vec!["the pass".into(), "the pass lands".into()];
    let twin = ops::new_node(&s, twin).unwrap();
    let o = ops::set(&s, &twin.front.id, &["acceptance-=the pass".to_string()], None).unwrap();
    assert_eq!(o.removed, vec!["the pass".to_string()], "exact beats substring ambiguity");
    assert_eq!(o.node.front.acceptance, vec!["the pass lands".to_string()]);
}

/// The it-ds6b rider on dc-p6z4 at mutation time: an edit that strips a
/// readied item's last acceptance line loudly demotes ready to shaped in
/// the verb's own output — the ready feed carries only items whose
/// contract is stated. Replace in one act keeps ready (the contract never
/// empties); in-flight demotes the same way, mirroring the fire-time
/// backstop's arm.
#[test]
fn stripping_a_readied_items_last_line_demotes_to_shaped_in_the_verbs_own_output() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |args: &[&str]| {
        std::process::Command::new(q)
            .current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args)
            .output()
            .unwrap()
    };
    let mut it = NewArgs::bare("item", "readied work");
    it.status = Some("ready".into());
    it.acceptance = vec!["the work lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    let out = run(&["set", &it.front.id, "acceptance-=the work"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("acceptance removed: \"the work lands\""),
        "the echo carries the full resolved line: {}",
        stdout
    );
    assert!(
        stdout.contains("UN-READIED") && stdout.contains("[shaped]"),
        "the demotion is loud in the verb's own output: {}",
        stdout
    );
    let all = s.load_all().unwrap();
    let n = s.find(&all, &it.front.id).unwrap();
    assert_eq!(n.front.status, "shaped", "ready-implies-acceptance held at mutation");
    let ev = s
        .read_log()
        .unwrap()
        .into_iter()
        .filter(|e| {
            e.get("op").and_then(|v| v.as_str()) == Some("set")
                && e.get("node").and_then(|v| v.as_str()) == Some(it.front.id.as_str())
        })
        .next_back()
        .unwrap();
    assert_eq!(
        ev["demoted"]["from"].as_str().unwrap(),
        "ready",
        "the demotion is logged with its cause: {}",
        ev
    );
    // replace in one act keeps ready — the contract never empties
    let mut rep = NewArgs::bare("item", "replaced contract");
    rep.status = Some("ready".into());
    rep.acceptance = vec!["the old outcome".into()];
    let rep = ops::new_node(&s, rep).unwrap();
    let o = ops::set(
        &s,
        &rep.front.id,
        &[
            "acceptance-=old outcome".to_string(),
            "acceptance+=the amended outcome".to_string(),
        ],
        None,
    )
    .unwrap();
    assert_eq!(o.node.front.status, "ready", "replace never demotes");
    assert!(o.demoted_from.is_none());
    // in-flight strips demote the same way (the backstop's arm mirrored)
    let mut fly = NewArgs::bare("item", "flying work");
    fly.status = Some("in-flight".into());
    fly.acceptance = vec!["the flight lands".into()];
    let fly = ops::new_node(&s, fly).unwrap();
    let o = ops::set(&s, &fly.front.id, &["acceptance-=flight".to_string()], None).unwrap();
    assert_eq!(o.demoted_from.as_deref(), Some("in-flight"));
    assert_eq!(o.node.front.status, "shaped");
}

/// The seat rules follow the pen (dc-mpg8, it-ds6b): authoring-by-
/// subtraction is authoring, so a witness seat's removal is marked at the
/// act and rides the design wake's review channel until the user
/// ratifies — flagged though the line is gone by construction. Design
/// seats mutate freely, unmarked; the removed line's own authored mark
/// degrades to record, exactly as the display already states.
#[test]
fn witness_seat_removal_rides_the_review_channel_until_the_users_word() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    quarry::coord::save_session(&s, "disp", vec![area.front.id.clone()], Some("dispatch".into()), None, false)
        .unwrap();
    quarry::coord::save_session(&s, "design", vec![area.front.id.clone()], Some("design".into()), None, false)
        .unwrap();
    let mut it = NewArgs::bare("item", "design authored");
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the gap closes".into(), "the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    // the witness seat removes a line: the act succeeds, marked for review
    let out = run(
        &[("QUARRY_SESSION", "disp")],
        &["set", &it.front.id, "acceptance-=gap"],
    );
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("acceptance removed: \"the gap closes\""),
        "the echo carries the full resolved line: {}",
        stdout
    );
    assert!(
        stdout.contains("witness seat") && stdout.contains("review channel"),
        "the act states its own review ride: {}",
        stdout
    );
    let all = s.load_all().unwrap();
    let n = s.find(&all, &it.front.id).unwrap();
    assert_eq!(n.front.acceptance, vec!["the pass lands".to_string()]);
    assert_eq!(n.front.witness.len(), 1, "the removal is marked");
    let m = &n.front.witness[0];
    assert!(m.removed, "marked as a removal act");
    assert_eq!(m.line, "the gap closes", "the mark stores the resolved line");
    assert_eq!(m.by, "session:disp");
    assert!(m.ratified.is_none());
    // the review channel carries the removal though the line is gone
    let flags = queries::witness_flags(&all);
    assert_eq!(flags.len(), 1, "the removal rides the channel: {:?}", flags.len());
    assert!(flags[0].1.removed);
    // the item view names the act and its state
    let out = run(&[], &["witness", &it.front.id]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("removed by session:disp") && stdout.contains("removal under review"),
        "the display names the subtraction: {}",
        stdout
    );
    // only the user's word clears it
    let out = run(&[], &["witness", &it.front.id, "--ratify", "--by", "user"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let all = s.load_all().unwrap();
    assert!(
        queries::witness_flags(&all).is_empty(),
        "the user's word lets the channel go"
    );
    let m = &s.find(&all, &it.front.id).unwrap().front.witness[0];
    assert!(m.removed && m.ratified.is_some(), "the mark stays as record");
    // a design seat removes freely — no mark, no review
    let out = run(
        &[("QUARRY_SESSION", "design")],
        &["set", &it.front.id, "acceptance-=pass"],
    );
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("witness seat"),
        "the design pen is free: {}",
        stdout
    );
    let all = s.load_all().unwrap();
    let n = s.find(&all, &it.front.id).unwrap();
    assert!(n.front.acceptance.is_empty());
    assert_eq!(n.front.witness.len(), 1, "no mark from the design seat");
    assert!(queries::witness_flags(&all).is_empty());
}

/// The wrap sweep's deep-touch list (dc-hzrm): per-session acts fold per
/// node and rank by touch depth — authoring outranks lifecycle outranks a
/// bare affirm; foreign sessions' acts never enter; the wrap event plants
/// the cursor, so a second wrap in the same boundary derives an empty list
/// and the sweep goes silent — once per boundary by construction. The
/// prompt itself carries the ruled requirements: direct authorship taught,
/// the Goodhart phrase naming volume.
#[test]
fn wrap_sweep_ranks_deep_touches_and_goes_silent_at_the_boundary() {
    let log = vec![
        serde_json::json!({"ts":"2099-01-01T00:00:00Z","op":"create","node":"it-aaaa","session":"geo"}),
        serde_json::json!({"ts":"2099-01-01T00:00:01Z","op":"body","node":"it-aaaa","session":"geo"}),
        serde_json::json!({"ts":"2099-01-01T00:00:02Z","op":"affirm","node":"cl-bbbb","session":"geo"}),
        serde_json::json!({"ts":"2099-01-01T00:00:03Z","op":"set","node":"it-cccc","session":"geo"}),
        serde_json::json!({"ts":"2099-01-01T00:00:04Z","op":"create","node":"it-zzzz","session":"bodies"}),
        serde_json::json!({"ts":"2099-01-01T00:00:05Z","op":"link","node":"it-cccc","session":"geo"}),
    ];
    let deep = queries::deep_touches(&log, Some("geo"));
    let ids: Vec<&str> = deep.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["it-aaaa", "it-cccc", "cl-bbbb"],
        "authoring depth leads, the bare affirm trails: {:?}",
        deep
    );
    assert!(
        deep[0].1 > deep[1].1 && deep[1].1 > deep[2].1,
        "depth strictly ranks the fold: {:?}",
        deep
    );
    assert!(
        !ids.contains(&"it-zzzz"),
        "a foreign session's acts never enter the list"
    );
    // geo's wrap plants the cursor: geo goes silent, bodies is untouched
    let mut log2 = log.clone();
    log2.push(serde_json::json!({"ts":"2099-01-01T00:00:06Z","op":"wrap","node":"session:geo","session":"geo"}));
    assert!(
        queries::deep_touches(&log2, Some("geo")).is_empty(),
        "the boundary replants: a second wrap sweeps nothing"
    );
    assert_eq!(
        queries::deep_touches(&log2, Some("bodies")).len(),
        1,
        "the cursor is per-session"
    );
    // the ruled prompt content: direct authorship taught, volume named
    assert!(
        quarry::framings::SPEND_DOWN.contains("q edit"),
        "the prompt teaches direct authorship"
    );
    assert!(
        quarry::framings::SPEND_DOWN.contains("Goodhart"),
        "the prompt carries the Goodhart phrase"
    );
}

/// The counter-voice (dc-hzrm): N hook-observed turns of graph silence fire
/// one reminder — dense at first encounter, a light phrase after — and any
/// graph act from the acting session resets the counter while a foreign
/// session's act does not. One line per silence stretch: continued silence
/// past the fire stays quiet until an act re-arms it. A filing session
/// never sees it by construction — its own acts keep resetting the count.
#[test]
fn counter_voice_fires_on_silence_resets_on_acts_and_teaches_dense_then_light() {
    let s = temp_store();
    let n = quarry::teach::COUNTER_VOICE_TURNS;
    // silence accumulates turn by turn; the threshold fires exactly once
    for i in 1..n {
        assert!(
            quarry::teach::counter_voice(&s, "geo").is_none(),
            "turn {} is below the threshold",
            i
        );
    }
    let fired = quarry::teach::counter_voice(&s, "geo").expect("the Nth silent turn fires");
    assert!(
        fired.contains("counter-voice") && fired.contains("speaks for the conversation"),
        "first encounter renders the dense teaching: {}",
        fired
    );
    assert!(
        quarry::teach::counter_voice(&s, "geo").is_none(),
        "one line per silence stretch — no nag on the next turn"
    );
    // a graph act from the acting session re-arms the counter
    s.log_event(serde_json::json!({"ts": Store::now(), "node": "it-fake", "v": 1, "op": "create", "actor": "t", "session": "geo"}))
        .unwrap();
    assert!(
        quarry::teach::counter_voice(&s, "geo").is_none(),
        "the act resets: the filing session never sees the line"
    );
    // a foreign session's act does not reset geo's silence
    s.log_event(serde_json::json!({"ts": Store::now(), "node": "it-fake", "v": 2, "op": "set", "actor": "t", "session": "bodies"}))
        .unwrap();
    for i in 1..n {
        assert!(
            quarry::teach::counter_voice(&s, "geo").is_none(),
            "turn {} after the reset stays quiet",
            i
        );
    }
    let again = quarry::teach::counter_voice(&s, "geo").expect("silence re-earns the line");
    assert!(
        again.contains(quarry::framings::COUNTER_VOICE_LIGHT),
        "later encounters render the light phrase: {}",
        again
    );
    assert!(
        !again.contains("speaks for the conversation"),
        "the dense teaching renders once: {}",
        again
    );
}

/// The counter-voice rides the session hook's injected-context channel for
/// the bound acting session, and a subagent-marked hook input (agent_id
/// present) neither counts a turn nor receives the line — the reminder
/// speaks to the conversation, and a subagent's context is not it.
#[test]
fn counter_voice_rides_the_session_hook_and_skips_subagent_contexts() {
    let s = temp_store();
    quarry::coord::write_adopt_request(&s, "cv-geo").unwrap();
    let parent = r#"{"session_id":"chat-cv","tool_name":"Bash","tool_input":{"command":"ls"}}"#;
    let sub = r#"{"session_id":"chat-cv","agent_id":"ag-cv","tool_name":"Bash","tool_input":{"command":"ls"}}"#;
    let context_of = |out: Option<serde_json::Value>| -> String {
        out.and_then(|v| {
            v["hookSpecificOutput"]["additionalContext"]
                .as_str()
                .map(String::from)
        })
        .unwrap_or_default()
    };
    let n = quarry::teach::COUNTER_VOICE_TURNS;
    for i in 1..n {
        let ctx = context_of(quarry::teach::session_hook_output(&s, parent));
        assert!(
            !ctx.contains("counter-voice"),
            "turn {} carries no reminder: {}",
            i,
            ctx
        );
        // a subagent call between turns neither counts nor receives
        let sctx = context_of(quarry::teach::session_hook_output(&s, sub));
        assert!(
            !sctx.contains("counter-voice"),
            "the subagent context never carries the line: {}",
            sctx
        );
    }
    let ctx = context_of(quarry::teach::session_hook_output(&s, parent));
    assert!(
        ctx.contains("counter-voice"),
        "the Nth parent turn draws the reminder into injected context: {}",
        ctx
    );
}

// ── the commit-sweep guard (it-4q6t) ───────────────────────────────────────

#[test]
fn sweep_shapes_match_bulk_staging_and_explicit_paths_pass() {
    use quarry::teach::sweep_shape;
    // Bulk shapes: the whole tree, cwd, graph directories, commit -a.
    let s = sweep_shape("git add -A", "").expect("-A is a sweep");
    assert_eq!(s.covers, vec![(String::new(), false)]);
    assert!(s.captures("graph/nodes/item/it-ab2c-x.md", false));
    let s = sweep_shape("git add .", "").expect(". at the root is a sweep");
    assert!(s.captures("graph/nodes/item/it-ab2c-x.md", false));
    // . in a subdirectory covers only that subtree — no graph capture.
    let s = sweep_shape("git add .", "src").expect(". is still sweep-shaped");
    assert!(!s.captures("graph/nodes/item/it-ab2c-x.md", false));
    let s = sweep_shape("git add graph", "").expect("the graph dir is a sweep");
    assert!(s.captures("graph/nodes/claim/cl-zz2z-y.md", true));
    assert!(sweep_shape("git add graph/nodes/item", "").is_some());
    // graph/log is bulk-shaped but covers only the exempt ledger.
    let s = sweep_shape("git add graph/log", "").expect("a dir is bulk");
    assert!(!s.captures("graph/nodes/item/it-ab2c-x.md", true));
    assert!(s.captures("graph/log/2026-08.jsonl", true));
    // commit -a captures tracked files only.
    let s = sweep_shape("git commit -am \"x\"", "").expect("commit -a is a sweep");
    assert!(s.captures("graph/nodes/item/it-ab2c-x.md", true));
    assert!(!s.captures("graph/nodes/item/it-ab2c-x.md", false));
    // A compound command still matches past the first command.
    assert!(sweep_shape("cargo build && git add -A", "").is_some());
    // Explicit paths and non-staging commands never match.
    assert!(sweep_shape("git add graph/nodes/item/it-ab2c-x.md", "").is_none());
    assert!(sweep_shape("git add graph/sessions.json", "").is_none());
    assert!(sweep_shape("git add src/main.rs; git commit -m msg", "").is_none());
    assert!(sweep_shape("git commit --amend", "").is_none());
    assert!(sweep_shape("git status", "").is_none());
    assert!(sweep_shape("git commit -m \"add graph\"", "").is_none());
}

#[test]
fn commit_set_ownership_derives_last_writer_and_annotates_carried_edits() {
    use quarry::queries::{commit_sets, node_id_of_path, PendingNodeFile};
    let log = vec![
        serde_json::json!({"ts": "2026-08-19T10:00:00Z", "node": "it-aaaa", "op": "create", "session": "quarry"}),
        serde_json::json!({"ts": "2026-08-19T11:00:00Z", "node": "it-aaaa", "op": "set", "session": "dispatcher"}),
        serde_json::json!({"ts": "2026-08-19T09:00:00Z", "node": "it-bbbb", "op": "create", "session": "quarry"}),
        serde_json::json!({"ts": "2026-08-19T12:00:00Z", "node": "it-cccc", "op": "set"}),
    ];
    let files = vec![
        PendingNodeFile {
            path: "graph/nodes/item/it-aaaa-x.md".into(),
            node: "it-aaaa".into(),
            prior_commit: None,
            tracked: false,
        },
        PendingNodeFile {
            path: "graph/nodes/item/it-bbbb-y.md".into(),
            node: "it-bbbb".into(),
            // The prior commit bounds the window: quarry's 09:00 edit is
            // already committed, so the pending delta has no stamped owner.
            prior_commit: Some("2026-08-19T09:30:00Z".into()),
            tracked: true,
        },
        PendingNodeFile {
            path: "graph/nodes/item/it-cccc-z.md".into(),
            node: "it-cccc".into(),
            prior_commit: None,
            tracked: true,
        },
    ];
    let sets = commit_sets(&log, files);
    assert_eq!(sets.len(), 2, "one owned set, one unstamped remainder");
    // Last writer since the prior commit owns the file; the earlier editor
    // is carried, annotated — the file lands in exactly one commit-set.
    assert_eq!(sets[0].owner.as_deref(), Some("dispatcher"));
    assert_eq!(sets[0].files.len(), 1);
    assert_eq!(sets[0].files[0].node, "it-aaaa");
    assert_eq!(sets[0].files[0].carries, vec!["quarry".to_string()]);
    assert_eq!(sets[0].add_command(), "git add graph/nodes/item/it-aaaa-x.md");
    // The unstamped set trails and blocks nothing.
    assert_eq!(sets[1].owner, None);
    let paths: Vec<&str> = sets[1].files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, vec!["graph/nodes/item/it-bbbb-y.md", "graph/nodes/item/it-cccc-z.md"]);
    // The id reads off the filename; non-node graph files stay out.
    assert_eq!(node_id_of_path("graph/nodes/item/it-4q6t-commit-sweep.md").as_deref(), Some("it-4q6t"));
    assert_eq!(node_id_of_path("graph/GRAPH.md"), None);
    assert_eq!(node_id_of_path("graph/nodes/item/README.md"), None);
}

#[test]
fn staging_guard_denies_the_sweep_with_the_askers_line_and_offers_adoption_when_stale() {
    use time::format_description::well_known::Rfc3339;
    let s = temp_store();
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .current_dir(&s.root)
            .args(args)
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {:?}: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    };
    git(&["init", "-q"]);
    git(&["-c", "user.name=t", "-c", "user.email=t@t", "commit", "--allow-empty", "-q", "-m", "root"]);
    // A node whose file is pending and whose last logged writer is quarry.
    let n = ops::new_node(&s, NewArgs::bare("item", "swept work")).unwrap();
    let future = (time::OffsetDateTime::now_utc() + time::Duration::seconds(5))
        .format(&Rfc3339)
        .unwrap();
    s.log_event(serde_json::json!({
        "ts": future, "node": n.front.id, "v": 1, "op": "set", "actor": "t", "session": "quarry"
    }))
    .unwrap();
    let rel = n
        .file
        .strip_prefix(&s.root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    // Backdate the owner's heartbeat: a stale owner flips the foreign line
    // to an explicit-path adoption offer.
    std::fs::write(
        s.root.join("graph").join(".sessions-live.json"),
        "{\"quarry\": \"2020-01-01T00:00:00Z\"}\n",
    )
    .unwrap();
    let cwd = s.root.clone();
    let deny = quarry::teach::staging_guard(&s, "Bash", "git add -A", Some(&cwd), Some("dispatcher"))
        .expect("the sweep denies while another session's node files are pending");
    assert!(deny.contains(&rel), "the foreign file is named: {}", deny);
    assert!(
        deny.contains("nothing of yours is pending"),
        "the asker's own line states its empty commit-set: {}",
        deny
    );
    assert!(
        deny.contains("adopt quarry's uncommitted graph nodes"),
        "a stale owner's set flips to the adoption offer: {}",
        deny
    );
    assert!(
        deny.contains(&format!("git add {}", rel)),
        "the adoption offer stages by explicit paths: {}",
        deny
    );
    assert!(deny.contains("q query commit-set"), "the pull handle is advertised: {}", deny);
    // A fresh owner keeps the leave-for-the-owner line — no adoption offer.
    quarry::coord::touch_session(&s, "quarry");
    let deny =
        quarry::teach::staging_guard(&s, "Bash", "git add graph", Some(&cwd), Some("dispatcher"))
            .expect("still denies");
    assert!(deny.contains("last seen 0m ago"), "the owner's age renders: {}", deny);
    assert!(!deny.contains("adopt quarry"), "a live owner's work is left, not offered: {}", deny);
    // The owner's own sweep captures nothing foreign — silence.
    assert!(
        quarry::teach::staging_guard(&s, "Bash", "git add -A", Some(&cwd), Some("quarry")).is_none()
    );
    // Explicit-path staging always passes, whoever asks.
    assert!(quarry::teach::staging_guard(
        &s,
        "Bash",
        &format!("git add {}", rel),
        Some(&cwd),
        Some("dispatcher")
    )
    .is_none());
    // Non-staging commands never engage the guard.
    assert!(
        quarry::teach::staging_guard(&s, "Bash", "git status", Some(&cwd), Some("dispatcher"))
            .is_none()
    );
    // commit -a stages tracked files only — an untracked (freshly minted)
    // node file is not captured, so the shape passes here.
    assert!(quarry::teach::staging_guard(
        &s,
        "PowerShell",
        "git commit -am \"work\"",
        Some(&cwd),
        Some("dispatcher")
    )
    .is_none());
    // Once the file is tracked and modified again under quarry's hand,
    // commit -a is a capture and denies.
    git(&["add", &rel]);
    git(&["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "land it"]);
    ops::set(&s, &n.front.id, &["title=swept work, renamed".to_string()], None).unwrap();
    let future = (time::OffsetDateTime::now_utc() + time::Duration::seconds(5))
        .format(&Rfc3339)
        .unwrap();
    s.log_event(serde_json::json!({
        "ts": future, "node": n.front.id, "v": 2, "op": "set", "actor": "t", "session": "quarry"
    }))
    .unwrap();
    let deny = quarry::teach::staging_guard(
        &s,
        "PowerShell",
        "git commit -am \"work\"",
        Some(&cwd),
        Some("dispatcher"),
    )
    .expect("commit -a captures the tracked modification");
    assert!(deny.contains(&rel), "the captured file is named: {}", deny);
}



// ── the observed set holds its accounting (it-bj3b) ────────────────────────

#[test]
fn store_relative_survives_windows_case_and_separator_mixing() {
    use quarry::store::store_relative;
    // Windows tool hands: mixed drive case, mixed separators, mixed
    // component case — all resolve to the same store-relative path.
    let root = "B:\\Repos\\Quarry";
    assert_eq!(
        store_relative(root, root, "b:/repos/quarry/SRC/Main.rs").as_deref(),
        Some("src/main.rs")
    );
    assert_eq!(
        store_relative(root, root, "B:\\repos\\QUARRY\\tests/Basic.rs").as_deref(),
        Some("tests/basic.rs")
    );
    // The component boundary holds: a sibling repo sharing the prefix
    // never resolves under this store.
    assert!(store_relative(root, root, "b:/repos/quarry2/src/x.rs").is_none());
    // The WORK root strips first (dc-g5x5): a fork path resolves through
    // the fork root even though the store root shares no prefix with it.
    let fork = "B:\\Repos\\Quarry\\.claude\\worktrees\\fork-1";
    assert_eq!(
        store_relative(fork, root, "b:\\repos\\quarry\\.claude\\worktrees\\FORK-1\\src\\teach.rs")
            .as_deref(),
        Some("src/teach.rs")
    );
    // Outside both roots, or relative-shaped: no resolution — the CALLER
    // records the badged case, never drops it (pinned below).
    assert!(store_relative(root, root, "c:/users/x/appdata/local/temp/scratch.md").is_none());
    assert!(store_relative(root, root, "src/main.rs").is_none());
    // The bare root itself is not a file write.
    assert!(store_relative(root, root, "b:/repos/quarry").is_none());
}

#[test]
fn write_shapes_parses_the_common_write_shapes() {
    use quarry::teach::write_shapes;
    // Redirects: bare, appending, glued, fd-prefixed, quoted target.
    assert_eq!(write_shapes("cat > tests/basic.rs <<'EOF'"), vec!["tests/basic.rs"]);
    assert_eq!(write_shapes("echo hi >> src/lib.rs"), vec!["src/lib.rs"]);
    assert_eq!(write_shapes("cargo build 2>build.log"), vec!["build.log"]);
    assert_eq!(write_shapes("echo x > 'docs/a file.md'"), vec!["docs/a file.md"]);
    // 2>&1 has no file target; /dev/null and variables never accrue.
    assert!(write_shapes("cargo test 2>&1").is_empty());
    assert!(write_shapes("cmd > /dev/null").is_empty());
    assert!(write_shapes("echo x > $OUT").is_empty());
    // tee, appending or not, through a pipe.
    assert_eq!(write_shapes("cargo test | tee -a logs/test.txt"), vec!["logs/test.txt"]);
    // cp/mv: the destination is the write.
    assert_eq!(write_shapes("cp fixtures/a.rs src/a.rs"), vec!["src/a.rs"]);
    assert_eq!(write_shapes("mv old.rs archive/old.rs && cargo check"), vec!["archive/old.rs"]);
    assert_eq!(write_shapes("cp -t src a.rs b.rs"), vec!["src"]);
    // touch names its targets outright.
    assert_eq!(write_shapes("touch src/new.rs tests/new.rs"), vec!["src/new.rs", "tests/new.rs"]);
    // git working-tree writers: checkout -- and restore; a plain branch
    // checkout is not a file write, and git apply is beyond the parse
    // (the sight boundary covers it).
    assert_eq!(write_shapes("git checkout -- src/main.rs"), vec!["src/main.rs"]);
    assert_eq!(write_shapes("git restore --source HEAD~1 src/ops.rs"), vec!["src/ops.rs"]);
    assert!(write_shapes("git checkout feature-branch").is_empty());
    assert!(write_shapes("git apply fix.patch").is_empty());
    // PowerShell content writers: flagged and positional paths, and the
    // here-string shape the incident class rode.
    assert_eq!(
        write_shapes("Set-Content -Path tests\\basic.rs -Value $body"),
        vec!["tests\\basic.rs"]
    );
    assert_eq!(write_shapes("$x | Out-File -FilePath out/report.md"), vec!["out/report.md"]);
    assert_eq!(write_shapes("Add-Content notes.md 'line'"), vec!["notes.md"]);
    assert_eq!(write_shapes("Copy-Item a.rs -Destination src/b.rs"), vec!["src/b.rs"]);
    assert_eq!(write_shapes("Move-Item a.tmp b.tmp"), vec!["b.tmp"]);
    // Compound commands: every simple command scans; duplicates fold.
    assert_eq!(
        write_shapes("touch src/a.rs; echo x > src/a.rs && cat > src/b.rs <<EOF"),
        vec!["src/a.rs", "src/b.rs"]
    );
}

/// The it-ap3x defect: a bare `@` in the observed set, marked shell-parsed.
/// The parse is a guess over text the tokenizer can read wrong, so debris
/// that cannot be a filename must never accrue as a touched path.
#[test]
fn write_shapes_drops_parser_debris_and_sigils() {
    use quarry::teach::write_shapes;
    // THE INCIDENT, verbatim in shape: this repo hands git a multi-line
    // commit message through a PowerShell here-string, and the message is
    // prose. Before the fix the apostrophe in "main.rs's" closed the quote
    // the `@'` opened, the closing `'@` re-opened it, and everything after
    // arrived as ONE word — pushed as a redirect target by the `>` that
    // closes the Co-Authored-By address. A literal "@ 2>&1 | tail -20"
    // landed in the observed set at the judgment seat.
    let incident = "git commit -m @'\n\
        main.rs's Join arm carried its own pinned-store paragraph.\n\
        \n\
        Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>\n\
        '@ 2>&1 | tail -20";
    assert!(
        write_shapes(incident).is_empty(),
        "a here-string commit message names no write target, got {:?}",
        write_shapes(incident)
    );
    // Swallowing the here-string must not blind the parser to the real
    // write beside it — the command's tail still tokenizes.
    assert_eq!(
        write_shapes("git commit -m @'\n\
            prose with don't in it\n\
            '@ && cargo build > build.log"),
        vec!["build.log"]
    );
    // An unterminated here-string ends the parse quietly, never a panic and
    // never a token of the leftover body.
    assert!(write_shapes("git commit -m @'\nbody that never closes").is_empty());
    // The double-quoted here-string form too — and swallowing it whole keeps
    // the writer's own path arg readable, unclosed literal or not.
    assert_eq!(
        write_shapes("Set-Content notes.md -Value @\"\nunclosed"),
        vec!["notes.md"]
    );
    assert_eq!(
        write_shapes("Set-Content notes.md -Value @\"\nline one\n\"@"),
        vec!["notes.md"]
    );
    // Sigil-only tokens are not paths, wherever the shape puts them.
    assert!(write_shapes("echo x > @").is_empty());
    assert!(write_shapes("touch @ -- {}").is_empty());
    assert!(write_shapes("cp a.rs @").is_empty());
    assert!(write_shapes("Out-File -FilePath @").is_empty());
    // Nor is a token carrying a shell metacharacter or a control character:
    // the tokenizer would have split on it in a command, so its presence is
    // the tell that this token is a fragment of something else.
    assert!(write_shapes("echo x > 'out > log'").is_empty());
    assert!(write_shapes("echo x > 'two\nlines'").is_empty());
    assert!(write_shapes("touch 'a | b'").is_empty());
    // The plausible stay plausible: spaces and Windows separators are
    // ordinary in a path and cost nothing to keep.
    assert_eq!(write_shapes("echo x > 'docs/a file.md'"), vec!["docs/a file.md"]);
    assert_eq!(write_shapes("touch src\\teach.rs"), vec!["src\\teach.rs"]);
}

/// The it-dt68 defect: a bash heredoc body tokenized as command text, so a
/// line of PROSE reading like a command minted a perfectly plausible touched
/// path — the it-ap3x false-positive class, one channel over, on the channel
/// this repo's own commit road runs through (`git commit -F - <<'EOF'`).
#[test]
fn write_shapes_never_mints_a_path_from_a_bash_heredoc_body() {
    use quarry::teach::write_shapes;
    // THE INCIDENT SHAPE: the commit road. Every line of the message used to
    // tokenize as words — `touch src/ghost.rs` in the prose minted
    // src/ghost.rs, and the judgment seat cannot tell it from a real write.
    let commit = "git commit -F - <<'EOF'\n\
        the parse: touch src/ghost.rs was never run\n\
        it also never ran cp fixtures/a.rs src/ghost2.rs\n\
        \n\
        Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>\n\
        EOF";
    assert!(
        write_shapes(commit).is_empty(),
        "a heredoc commit message names no write target, got {:?}",
        write_shapes(commit)
    );
    // All three openers: quoted (above), bare, and the tab-stripped form
    // whose terminator is indented.
    assert!(write_shapes("cat <<EOF\ntouch src/ghost.rs\nEOF").is_empty());
    assert!(write_shapes("cat <<\"EOF\"\ntouch src/ghost.rs\nEOF").is_empty());
    assert!(write_shapes("cat <<-EOF\n\ttouch src/ghost.rs\n\tEOF").is_empty());
    // A space between the operator and the delimiter is legal too.
    assert!(write_shapes("cat << EOF\ntouch src/ghost.rs\nEOF").is_empty());
    // A REAL WRITE BESIDE THE HEREDOC STILL PARSES. The body is taken at the
    // newline that ends the opener line, not at the operator, so a redirect
    // standing after the delimiter still belongs to its command.
    assert_eq!(
        write_shapes("cat <<EOF > out/real.txt\ntouch src/ghost.rs\nEOF"),
        vec!["out/real.txt"]
    );
    // And the command after the terminator is its own command, not an
    // argument of the prose that preceded it.
    assert_eq!(
        write_shapes("git commit -F - <<'EOF'\ntouch src/ghost.rs\nEOF\ntouch src/real.rs"),
        vec!["src/real.rs"]
    );
    // Two heredocs on one line take their bodies in order.
    assert_eq!(
        write_shapes("diff <<A <<B > out/diff.txt\ntouch src/g1.rs\nA\ntouch src/g2.rs\nB"),
        vec!["out/diff.txt"]
    );
    // An unterminated body ends the parse quietly — never a panic, never a
    // token of the leftover prose.
    assert!(write_shapes("cat <<EOF\ntouch src/ghost.rs\nnever closes").is_empty());
    // The delimiter must match the WHOLE line: prose that merely contains the
    // word does not end the body (trailing whitespace and a CRLF's `\r` do
    // not make a terminator a body line, though).
    assert!(write_shapes("cat <<EOF\nEOF is the delimiter here\ntouch src/ghost.rs\nEOF").is_empty());
    assert_eq!(
        write_shapes("cat <<EOF\ntouch src/ghost.rs\r\nEOF \ntouch src/real.rs"),
        vec!["src/real.rs"]
    );
    // `<<<` is a here-string, not a heredoc: one word, no body to swallow —
    // the write beside it stays visible.
    assert_eq!(
        write_shapes("cat <<< \"prose that says touch src/ghost.rs\" > out/real.txt"),
        vec!["out/real.txt"]
    );
    // A plain input redirect is untouched.
    assert_eq!(write_shapes("sort < in.txt > out/sorted.txt"), vec!["out/sorted.txt"]);
}

/// The it-dprv defect: a newline was ordinary whitespace, so a multi-line
/// command tokenized as ONE command and only its first word was ever read as
/// a command word. Both error directions followed — a later line's command
/// word minted as a path (the plausibility floor cannot catch it: "touch" is
/// a legal filename), and a real write on a later line went unseen.
#[test]
fn a_newline_ends_the_command_before_it_unless_the_line_continues() {
    use quarry::teach::write_shapes;
    // FALSE POSITIVE, the it-ap3x/it-dt68 class again: pre-fix this parsed to
    // ["src/a.rs", "touch", "src/b.rs"] — line two's command word read as an
    // argument of line one's `touch` and minted as a touched path.
    assert_eq!(
        write_shapes("touch src/a.rs\ntouch src/b.rs"),
        vec!["src/a.rs", "src/b.rs"]
    );
    // FALSE NEGATIVE: pre-fix this parsed to ["build.log"] alone — the `cp` on
    // line two was consumed as an argument of line one and never scanned as a
    // command, so the sight boundary widened by a whole line.
    assert_eq!(
        write_shapes("cargo build > build.log\ncp fixtures/a.rs src/a.rs"),
        vec!["build.log", "src/a.rs"]
    );
    // Every line is its own command, however many there are, and a CRLF ends
    // one exactly as an LF does.
    assert_eq!(
        write_shapes("touch src/a.rs\ntouch src/b.rs\ntouch src/c.rs"),
        vec!["src/a.rs", "src/b.rs", "src/c.rs"]
    );
    assert_eq!(
        write_shapes("touch src/a.rs\r\ntouch src/b.rs"),
        vec!["src/a.rs", "src/b.rs"]
    );
    // A redirect left dangling at the end of a line takes no target from the
    // next line — pre-fix it took "touch".
    assert_eq!(write_shapes("cargo build >\ntouch src/a.rs"), vec!["src/a.rs"]);
    // A LINE CONTINUATION KEEPS ITS TAIL. This tokenizer does no escape
    // handling, so a naive newline boundary would drop the destination of a
    // wrapped command: bash's trailing `\` and PowerShell's trailing backtick
    // both join the lines instead.
    assert_eq!(write_shapes("cp fixtures/a.rs \\\n  src/b.rs"), vec!["src/b.rs"]);
    assert_eq!(write_shapes("cp fixtures/a.rs \\\r\n  src/b.rs"), vec!["src/b.rs"]);
    assert_eq!(
        write_shapes("Copy-Item a.rs `\n  -Destination src/b.rs"),
        vec!["src/b.rs"]
    );
    assert_eq!(write_shapes("cargo build \\\n  > build.log"), vec!["build.log"]);
    // The continuation character must STAND AS ITS OWN WORD — the idiomatic
    // form in both shells, and the narrowing that keeps the exception from
    // minting. A Windows path ending in a separator is not a continuation, so
    // the line after it is still read as its own command: measured, a rule
    // reading ANY trailing `\` parses this to ["B:\\desttouch"] alone — a
    // plausible path nobody wrote, and line two's real write lost with it.
    assert_eq!(
        write_shapes("Copy-Item a.rs B:\\dest\\\ntouch src/b.rs"),
        vec!["B:\\dest\\", "src/b.rs"]
    );
    // The residue, pinned: a continuation GLUED to its word is not read, so a
    // wrapped `cp` loses its destination. The broad rule buys nothing here —
    // bash's own joining makes that destination "a.rssrc/b.rs", which the
    // parse drops either way — so the loss is the word rule's price nowhere.
    assert!(write_shapes("cp a.rs\\\nsrc/b.rs").is_empty());
    // A QUOTED newline is not a boundary: prose inside an argument stays one
    // token, so a commit message line that reads like a command mints nothing
    // (the it-ap3x/it-dt68 floor, reached through this arm).
    assert_eq!(
        write_shapes("git commit -m \"line one\ntouch src/ghost.rs\" && touch src/real.rs"),
        vec!["src/real.rs"]
    );
    // The heredoc body is still taken whole, and the command after its
    // terminator is still its own command (it-dt68's positive control, now
    // reached with newlines carrying boundaries of their own).
    assert_eq!(
        write_shapes("cat <<EOF > out/real.txt\ntouch src/ghost.rs\nEOF\ntouch src/real.rs"),
        vec!["out/real.txt", "src/real.rs"]
    );
}

#[test]
fn observe_shell_accrues_only_resolvable_targets_marked_shell() {
    let s = temp_store();
    let cwd = s.root.clone();
    // A heredoc write inside the repo accrues store-relative, marked shell;
    // an absolute out-of-repo target and a graph/ target accrue nothing —
    // a parse is a guess, only resolvable targets count (it-bj3b #4).
    quarry::teach::observe_shell(
        &s,
        Some("it-shell"),
        Some("sess"),
        "cat > tests/basic.rs <<'EOF' && cp x.rs c:/tmp/x.rs && echo y > graph/.probe",
        Some(&cwd),
    );
    let touches = quarry::coord::touches_for(&s, "item:it-shell");
    assert_eq!(touches.len(), 1, "one resolvable target");
    assert_eq!(touches[0].path, "tests/basic.rs");
    assert_eq!(touches[0].via.as_deref(), Some("shell"));
    assert!(!touches[0].unresolved);
    // Absolute in-repo targets resolve too, whatever their case or
    // separators (the store_relative point carries this channel as well).
    let abs = format!("{}\\SRC\\Lib.rs", s.root.display().to_string().to_uppercase());
    quarry::teach::observe_shell(
        &s,
        Some("it-shell"),
        Some("sess"),
        &format!("echo x > \"{}\"", abs),
        Some(&cwd),
    );
    let touches = quarry::coord::touches_for(&s, "item:it-shell");
    assert_eq!(touches.len(), 2);
    assert_eq!(touches[1].path, "src/lib.rs");
    // Re-parsing the same command accrues nothing new; leaseless shells
    // accrue under the session key like any observed write.
    quarry::teach::observe_shell(
        &s,
        Some("it-shell"),
        Some("sess"),
        "cat > tests/basic.rs",
        Some(&cwd),
    );
    assert_eq!(quarry::coord::touches_for(&s, "item:it-shell").len(), 2, "dedup holds");
    quarry::teach::observe_shell(&s, None, Some("solo-sh"), "touch src/solo.rs", Some(&cwd));
    let solo = quarry::coord::touches_for(&s, "session:solo-sh");
    assert_eq!(solo.len(), 1);
    assert_eq!(solo[0].path, "src/solo.rs");
    // it-ap3x: parser debris resolves store-relative by mere path joining —
    // nothing on disk answers for it — so the drop has to happen at the
    // parse, not at resolution. The observed set stays paths.
    quarry::teach::observe_shell(
        &s,
        Some("it-shell"),
        Some("sess"),
        "git commit -m @'\n\
            main.rs's arm carried its own paragraph\n\
            \n\
            Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>\n\
            '@ 2>&1 | tail -20",
        Some(&cwd),
    );
    let touches = quarry::coord::touches_for(&s, "item:it-shell");
    let paths: Vec<&str> = touches.iter().map(|t| t.path.as_str()).collect();
    assert_eq!(paths, vec!["tests/basic.rs", "src/lib.rs"], "the here-string commit accrued debris");
    // it-dt68, the same reasoning one channel over: a heredoc body line that
    // READS like a command names a path that resolves store-relative by mere
    // joining — src/ghost.rs is a perfectly plausible touched path and no
    // resolution step can tell it from a real write. The drop belongs at the
    // parse; the real write beside the heredoc still accrues.
    quarry::teach::observe_shell(
        &s,
        Some("it-shell"),
        Some("sess"),
        "git commit -F - <<'EOF' > out.log\n\
            the parse: touch src/ghost.rs was never run\n\
            EOF",
        Some(&cwd),
    );
    let touches = quarry::coord::touches_for(&s, "item:it-shell");
    let paths: Vec<&str> = touches.iter().map(|t| t.path.as_str()).collect();
    assert_eq!(
        paths,
        vec!["tests/basic.rs", "src/lib.rs", "out.log"],
        "the heredoc commit accrued its body as touched paths"
    );
    // it-dprv, the same reasoning where a newline was mere whitespace: line
    // two's command word read as line one's argument, and a bare "touch"
    // resolves store-relative by mere joining — an observed path no judgment
    // seat could tell from a real write, and no plausibility floor can refuse
    // ("touch" is a legal filename). The drop belongs at the parse; line two's
    // own write, invisible for the same reason, accrues.
    quarry::teach::observe_shell(
        &s,
        Some("it-shell"),
        Some("sess"),
        "touch src/one.rs\ncp fixtures/a.rs src/two.rs",
        Some(&cwd),
    );
    let touches = quarry::coord::touches_for(&s, "item:it-shell");
    let paths: Vec<&str> = touches.iter().map(|t| t.path.as_str()).collect();
    assert_eq!(
        paths,
        vec!["tests/basic.rs", "src/lib.rs", "out.log", "src/one.rs", "src/two.rs"],
        "the second line's command word accrued, or its write went unseen"
    );
}

#[test]
fn a_badged_write_failing_resolution_is_recorded_and_harvest_states_the_boundary() {
    use quarry::teach::observe_write;
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "sight accounting pass");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the accounting lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::dispatch(&s, &it.front.id, vec!["src/**".into()], false, false, None, None, "geo", "t").unwrap();
    let badge = it.front.id.clone();
    let key = format!("item:{}", badge);
    // A badged tool write whose path never resolved store-relative is
    // RECORDED with its raw path, marked unresolved — never dropped
    // (it-bj3b #2); repeats fold; the resolved reader never shows it.
    observe_write(&s, &[], Some("geo"), Some(&badge), "d:/elsewhere/checkout/src/teach.rs");
    observe_write(&s, &[], Some("geo"), Some(&badge), "d:/elsewhere/checkout/src/teach.rs");
    let touches = quarry::coord::touches_for(&s, &key);
    assert_eq!(touches.len(), 1, "the unresolved write recorded once");
    assert!(touches[0].unresolved);
    assert_eq!(touches[0].path, "d:/elsewhere/checkout/src/teach.rs");
    assert!(quarry::coord::touched_for(&s, &key).is_empty(), "resolved reader stays clean");
    // Leaseless, the same path stays unrecorded — outside the repo is not
    // this graph's arc; only a badge makes it accounting.
    observe_write(&s, &[], Some("loose"), None, "d:/elsewhere/notes.md");
    assert!(quarry::coord::touches_for(&s, "session:loose").is_empty());
    // An ordinary badged write and a shell-parsed one land beside it.
    observe_write(&s, &[], Some("geo"), Some(&badge), "src/main.rs");
    quarry::coord::accrue_touch_ext(&s, &key, "tests/basic.rs", Some("shell"), false);
    // Harvest renders all three channels and states the sight boundary
    // (it-bj3b #5): a partial observed set can never read as complete.
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(h.contains("files touched under the badge (2)"), "resolved set counted: {}", h);
    assert!(
        h.contains("tests/basic.rs  (shell-parsed, best-effort)"),
        "the shell channel is named per file: {}", h
    );
    assert!(
        h.contains("unresolved under the badge (1)")
            && h.contains("d:/elsewhere/checkout/src/teach.rs"),
        "the unresolved write renders raw at the judgment seat: {}", h
    );
    assert!(
        h.contains(quarry::framings::SIGHT_BOUNDARY),
        "the sight boundary is stated: {}", h
    );
    // The boundary line rides the empty set too — and the dispatch trace.
    let bare = {
        let mut b = NewArgs::bare("item", "bare arc");
        b.status = Some("ready".into());
        b.about = vec![area.front.id.clone()];
        b.acceptance = vec!["lands".into()];
        ops::new_node(&s, b).unwrap()
    };
    ops::dispatch(&s, &bare.front.id, vec!["src/**".into()], false, false, None, None, "geo", "t")
        .unwrap();
    let h2 = quarry::render::harvest(&s, &bare.front.id).unwrap();
    assert!(h2.contains("no code writes observed under this badge"));
    assert!(h2.contains(quarry::framings::SIGHT_BOUNDARY), "boundary on the empty set: {}", h2);
    let tr = quarry::render::dispatch_trace(&s, &it.front.id).unwrap();
    assert!(
        tr.contains("(unresolved — raw path, never resolved store-relative)"),
        "the trace marks the unresolved channel: {}", tr
    );
    assert!(tr.contains(quarry::framings::SIGHT_BOUNDARY), "boundary on the trace: {}", tr);
}

// ── silent user-owned calls trip the constructed ends (it-f6c2) ────────────

#[test]
fn return_spec_and_first_echo_carry_the_user_owned_calls_rule() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut it = NewArgs::bare("item", "water body graph");
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["bodies persist across reload".into()];
    it.body = "build the graph".into();
    let it = ops::new_node(&s, it).unwrap();
    // The brief's RETURN spec carries the declaration slot: each call or
    // "none", absence itself a harvest flag.
    let text = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(
        text.contains(quarry::framings::USER_OWNED_SLOT),
        "the RETURN spec carries the user-owned-calls slot: {}",
        text
    );
    // …and states the shape it will be READ in (it-drsu): the requirement
    // was otherwise discoverable only by reading the parser.
    assert!(
        text.contains("The shape it is read in: one entry per call under that head"),
        "the RETURN spec states the read shape: {}",
        text
    );
    // The first-interception echo names thread-filing as the only landing.
    let d = quarry::coord::DispatchState {
        item: "it-bdg".into(),
        item_title: "guard growth".into(),
        session: "geo".into(),
        holder: "chat:chat-disp".into(),
        globs: vec!["src/**".into()],
        acceptance: vec!["a".into()],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: Store::now(),
        token: None,
        joined: None,
        model: None,
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    let out = quarry::teach::observe_write(&s, &[], Some("geo"), Some("it-bdg"), "src/main.rs");
    assert!(
        out.iter().any(|l| l.contains("first write under dispatch")
            && l.contains(quarry::framings::USER_OWNED_ECHO)),
        "the contract echo names the thread landing: {:?}",
        out
    );
}

#[test]
fn declared_user_owned_calls_parses_the_section_shapes() {
    use quarry::queries::declared_user_owned_calls;
    // A markdown heading over list entries.
    assert_eq!(
        declared_user_owned_calls("intro\n\n## User-owned calls\n- one\n- two\n\nREFLECTIONS\n"),
        Some(2)
    );
    // "none" on the head line, markdown dressing included.
    assert_eq!(declared_user_owned_calls("user-owned calls: none"), Some(0));
    assert_eq!(declared_user_owned_calls("**User-owned calls:** none"), Some(0));
    // "none" on its own line under the head.
    assert_eq!(declared_user_owned_calls("USER-OWNED CALLS ENCOUNTERED:\nnone\n\nmore"), Some(0));
    // A single declaration on the head line itself.
    assert_eq!(
        declared_user_owned_calls("user-owned calls: the retire wording is the user's (th-1)"),
        Some(1)
    );
    // Indented continuations never inflate the count.
    assert_eq!(
        declared_user_owned_calls("user-owned calls:\n- call one\n  filed as th-1\n- call two\n"),
        Some(2)
    );
    // No section at all: None — absence is itself the harvest flag.
    assert_eq!(declared_user_owned_calls("outcomes\n\nREFLECTIONS: fine\n"), None);
}

/// it-drsu: a lead-in sentence between the section head and its list is
/// ordinary prose, and reading it as the section's end scored a declared
/// call as zero — the false all-clear the whole accounting exists to
/// prevent. The witnessed shape (do-yeum, the it-p8rp return) leads; the
/// corpus shapes that must keep their old answers follow.
#[test]
fn a_lead_in_before_the_list_never_reads_as_a_declaration_of_none() {
    use quarry::queries::declared_user_owned_calls;
    // THE INCIDENT, verbatim in shape: a bold count announcement, a blank,
    // then the numbered entry it announces.
    let witnessed = "## user-owned calls\n\n**One.**\n\n1. **The ratified `user_owned_await` wording now carries a second meaning** —\n   filed as **th-t4j3** (queued). The line says \"no report file to parse yet\",\n   which was simply true before this fix.\n\n## Reflections\n\n- the fix is four lines\n- second-resolution stamps are a real edge\n";
    assert_eq!(
        declared_user_owned_calls(witnessed),
        Some(1),
        "the lead-in is skipped and the entry below it is counted"
    );
    // The section ends at the next heading, so the reflections list below
    // it never inflates the count — the second half of the acceptance line.
    let two_entries = "## user-owned calls\n\nTwo, both filed.\n\n- one (th-a)\n- two (th-b)\n\n## Reflections\n\n- a\n- b\n- c\n";
    assert_eq!(declared_user_owned_calls(two_entries), Some(2));
    // A loose list — blank lines between entries — keeps its count: a blank
    // closes a paragraph, never the section.
    assert_eq!(
        declared_user_owned_calls("user-owned calls:\n\n- one (th-a)\n\n- two (th-b)\n\n## Reflections\n"),
        Some(2)
    );
    // Prose-only sections. "none" opening the paragraph declares zero
    // however much sentence rides with it (both real corpus shapes);
    // prose that does NOT say none declares one, never zero.
    assert_eq!(
        declared_user_owned_calls("## user-owned calls:\n\nnone declared by the agent (the family was user-agreed per the brief). Two\nin-scope judgment calls flagged: `contains_word` visibility widened.\n\n## Dispatcher's judgment acts at landing\n"),
        Some(0)
    );
    assert_eq!(
        declared_user_owned_calls("## user-owned calls\n\n**none.** One near-miss checked rather than assumed: the `store pinned: …` line\ndeleted here was a composed string.\n\n**Composed register (dc-vzvf):** one string removed.\n\n## Reflections\n"),
        Some(0)
    );
    assert_eq!(
        declared_user_owned_calls("## user-owned calls\n\nThe retire wording is the user's; I took the provisional path.\n\n## Reflections\n"),
        Some(1),
        "a prose declaration is a declaration — never a silent zero"
    );
    // "Nonetheless" is not "none": the word must stand whole.
    assert_eq!(
        declared_user_owned_calls("## user-owned calls\n\nNonetheless the retire wording is the user's.\n\n## Reflections\n"),
        Some(1)
    );
    // An empty section still declares nothing.
    assert_eq!(declared_user_owned_calls("## user-owned calls\n\n## Reflections\n- a\n"), Some(0));
    // An un-headed report cannot swallow its next section: a second prose
    // paragraph past the lead-in ends the walk before that section's list.
    assert_eq!(
        declared_user_owned_calls("user-owned calls:\n\nA lead-in with no entries under it.\n\nReflections:\n\n- a\n- b\n- c\n"),
        Some(1)
    );
}

#[test]
fn harvest_reconciles_declared_user_owned_calls_against_badge_threads() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "geo pass");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "geo", "t")
        .unwrap();
    // No report file to parse yet: the count of badge threads confronts
    // the prose in hand.
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(
        h.contains(&quarry::framings::user_owned_await(0)),
        "pre-registration the reconciliation states the badge-thread count: {}",
        h
    );
    // A thread filed under the badge (the stamped channel an agent's env
    // provides).
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": "th-feed", "v": 1, "op": "create", "type": "thread",
        "actor": "t", "dispatch": it.front.id
    }))
    .unwrap();
    // The registered report declares the call: N = M reconciles.
    std::fs::create_dir_all(s.root.join("docs/reports")).unwrap();
    let rp = s.root.join("docs/reports/r.md");
    std::fs::write(&rp, "outcomes hold\n\nuser-owned calls:\n- the retire wording is the user's; filed as th-feed\n\nREFLECTIONS: fine\n").unwrap();
    let mut doc = NewArgs::bare("doc", "dispatch report: geo pass");
    doc.kind = Some("report".into());
    doc.path = Some("docs/reports/r.md".into());
    let doc = ops::new_node(&s, doc).unwrap();
    ops::link(&s, &doc.front.id, "supports", &it.front.id, false, None).unwrap();
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(
        h.contains(&quarry::framings::user_owned_reconcile(Some(1), 1)),
        "declared 1 against 1 badge thread reconciles: {}",
        h
    );
    // The section gone: absence is itself the flag.
    std::fs::write(&rp, "outcomes hold\n\nREFLECTIONS: fine\n").unwrap();
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(
        h.contains(&quarry::framings::user_owned_reconcile(None, 1)),
        "a report missing the section flags loud: {}",
        h
    );
    assert!(h.contains("absence is itself the flag"), "the flag names itself: {}", h);
    // "none" declared while a badge thread stands: the gap confronts.
    std::fs::write(&rp, "outcomes hold\n\nuser-owned calls: none\n\nREFLECTIONS: fine\n").unwrap();
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(
        h.contains(&quarry::framings::user_owned_reconcile(Some(0), 1)),
        "declared none against a filed thread confronts the gap: {}",
        h
    );
    assert!(h.contains("a gap is a question"), "the gap confronts, never verdicts: {}", h);
}

/// it-p8rp: the reconcile's report join is bounded by the arc's own
/// dispatch. A dispatcher carries a prior arc's report onto an item with a
/// supports edge (the brief shows it as evidence, deliberately) — and
/// unbounded, newest-wins picked that stranger's prose as this arc's
/// return before the arc had registered anything. Both halves measured:
/// the pure pick, and the harvest surface that consumes it.
#[test]
fn the_reconcile_never_parses_a_report_that_predates_the_arcs_dispatch() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "geo pass");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();

    // A PRIOR arc's report, carried onto this item by a dispatcher for its
    // caveat: registered, supports-linked, and older than any dispatch of
    // this item. Its prose declares "none" — the tell if it is ever parsed.
    std::fs::create_dir_all(s.root.join("docs/reports")).unwrap();
    std::fs::write(
        s.root.join("docs/reports/prior.md"),
        "a prior arc's outcomes\n\nuser-owned calls: none\n\nREFLECTIONS: fine\n",
    )
    .unwrap();
    let mut prior = NewArgs::bare("doc", "dispatch report: some other arc");
    prior.kind = Some("report".into());
    prior.path = Some("docs/reports/prior.md".into());
    let prior = ops::new_node(&s, prior).unwrap();
    ops::link(&s, &prior.front.id, "supports", &it.front.id, false, None).unwrap();
    // Backdate on disk, after the link's rewrite: the clock is
    // second-resolution, so a test-authored "prior" is otherwise the same
    // instant as the dispatch it must predate.
    let f = &prior.file;
    let raw = std::fs::read_to_string(f).unwrap();
    let raw = raw.replace(
        &format!("created: {}", prior.front.created),
        "created: 2020-01-01T00:00:00Z",
    );
    std::fs::write(&f, raw).unwrap();
    let all = s.load_all().unwrap();
    let prior_node = s.find(&all, &prior.front.id).unwrap();
    assert_eq!(prior_node.front.created, "2020-01-01T00:00:00Z", "backdating held");

    ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "geo", "t")
        .unwrap();

    // THE PURE PICK: the carried report is visible unbounded (the old
    // behaviour, and what the brief's evidence section still shows) and
    // invisible once the arc's dispatch bounds it.
    let all = s.load_all().unwrap();
    let log = s.read_log().unwrap();
    let since = queries::arc_dispatched_at(&log, &it.front.id).expect("the dispatch stamped the log");
    assert!(
        queries::latest_report_doc(&all, &it.front.id, None)
            .map(|d| d.front.id.clone())
            .as_deref()
            == Some(prior.front.id.as_str()),
        "unbounded, the carried report is the newest supports-linked report"
    );
    assert!(
        queries::latest_report_doc(&all, &it.front.id, Some(&since)).is_none(),
        "bounded by the arc's dispatch, a report that predates it is no return"
    );

    // THE SURFACE: the await arm speaks — the reconcile never parsed the
    // stranger's "none".
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(
        h.contains(&quarry::framings::user_owned_await(0)),
        "before this arc registers, the await arm speaks: {}",
        h
    );
    assert!(
        !h.contains(&quarry::framings::user_owned_reconcile(Some(0), 0)),
        "the carried report's declaration never stands in as the arc's return: {}",
        h
    );

    // This arc's own report registers: the reconcile is mechanical again,
    // and it parses THIS report, not the carried one.
    std::fs::write(
        s.root.join("docs/reports/mine.md"),
        "this arc's outcomes\n\nuser-owned calls:\n- the wording is the user's; filed as th-feed\n\nREFLECTIONS: fine\n",
    )
    .unwrap();
    let mut mine = NewArgs::bare("doc", "dispatch report: geo pass");
    mine.kind = Some("report".into());
    mine.path = Some("docs/reports/mine.md".into());
    let mine = ops::new_node(&s, mine).unwrap();
    ops::link(&s, &mine.front.id, "supports", &it.front.id, false, None).unwrap();
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(
        h.contains(&quarry::framings::user_owned_reconcile(Some(1), 0)),
        "the arc's own report is what the reconcile parses: {}",
        h
    );

    // A RE-DISPATCH moves the bound forward: arc 1's report is a
    // predecessor's from arc 2's seat. Measured on the pure pick, where the
    // stamp is explicit and the second-resolution clock cannot blur it.
    let all = s.load_all().unwrap();
    let later = "2999-01-01T00:00:00Z";
    assert!(
        queries::latest_report_doc(&all, &it.front.id, Some(later)).is_none(),
        "a later arc reconciles against its own report, never its predecessor's"
    );
    assert_eq!(
        queries::latest_report_doc(&all, &it.front.id, Some("2020-01-01T00:00:00Z"))
            .map(|d| d.front.id.clone()),
        Some(mine.front.id.clone()),
        "within the bound, newest-created still wins"
    );
    // Never dispatched: nothing to bound by, and the filter degrades to
    // unfiltered rather than to empty.
    assert!(queries::arc_dispatched_at(&log, "it-never").is_none());
}

#[test]
fn a_comma_joined_files_value_is_refused_where_the_globs_enter() {
    // it-x4bb: --files is repeatable and carries no value delimiter, so one
    // comma-joined value mints ONE glob. `globs_overlap` compares STATIC
    // PREFIXES by the prefix-of relation, so that glob matches the first path
    // in the value and nothing after it: the brief prints the rest as leased
    // and the agent's own badge then denies the writes, mid-arc, in a seat
    // that cannot extend a lease. The shape is refused where the globs enter.
    use quarry::coord::{check_glob_shapes, globs_overlap};
    let s = temp_store();
    let joined = "src/ops.rs,src/render.rs,tests/**";

    // THE DEFECT, measured on the matcher itself — the arc that lost its
    // src/render.rs and tests/** halves (it-rmqy) lost them to exactly this.
    assert!(globs_overlap(joined, "src/ops.rs"), "the first path matches — the arc starts");
    assert!(!globs_overlap(joined, "src/render.rs"), "the second is leased in name only");
    assert!(!globs_overlap(joined, "tests/basic.rs"), "and so is the third");

    // THE SOLO STATION (q reserve's road): coord::reserve refuses, teaching
    // the repeatable flag with the corrected command derived from the value
    // in hand, and no lease is written.
    let mut a = NewArgs::bare("item", "comma at reserve");
    a.acceptance = vec!["the work lands".into()];
    let a = ops::new_node(&s, a).unwrap();
    let err = quarry::coord::reserve(&s, &a, "geo", "t", vec![joined.into()], false, false, None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("REPEATABLE"), "the flag's shape is taught: {}", err);
    assert!(
        err.contains("--files \"src/ops.rs\" --files \"src/render.rs\" --files \"tests/**\""),
        "the corrected command is derived from the offending value: {}",
        err
    );
    assert!(quarry::coord::load_leases(&s).is_empty(), "nothing is leased whole");

    // THE FIRING STATION: ops::dispatch refuses ahead of every mutation —
    // no brief event, no lease, no badge, no in-flight flip, nothing to undo.
    let mut b = NewArgs::bare("item", "comma at dispatch");
    b.status = Some("ready".into());
    b.acceptance = vec!["the work lands".into()];
    let b = ops::new_node(&s, b).unwrap();
    let err = ops::dispatch(&s, &b.front.id, vec![joined.into()], false, false, None, None, "geo", "t")
        .unwrap_err()
        .to_string();
    assert!(err.contains("REPEATABLE"), "the same teaching at both stations: {}", err);
    assert!(quarry::coord::load_leases(&s).is_empty(), "no lease on a refused fire");
    assert!(quarry::coord::dispatch_for_item(&s, &b.front.id).is_none(), "no badge minted");
    assert!(
        !quarry::coord::briefed_this_session(&s, &b.front.id, "geo"),
        "no brief event — C8 stays unsatisfied, so the refusal cannot be walked past"
    );
    let all = s.load_all().unwrap();
    assert_eq!(
        s.find(&all, &b.front.id).unwrap().front.status,
        "ready",
        "a mistyped fire leaves the item exactly as it found it"
    );

    // THE FALLBACK ROAD: an item's recorded write-set reaches the same lease,
    // so it meets the same floor at reserve — a --files-less dispatch cannot
    // inherit the shape from the item either.
    ops::set(&s, &b.front.id, &[format!("write-set+={}", joined)], None).unwrap();
    let err = ops::dispatch(&s, &b.front.id, vec![], false, false, None, None, "geo", "t")
        .unwrap_err()
        .to_string();
    assert!(err.contains("REPEATABLE"), "the fallback inherits the floor: {}", err);
    assert!(quarry::coord::load_leases(&s).is_empty(), "still nothing leased whole");

    // THE POSITIVE CONTROL: the repeatable form leases every glob, and every
    // path the brief would print as leased passes the overlap test for real.
    let mut c = NewArgs::bare("item", "the repeatable form");
    c.status = Some("ready".into());
    c.acceptance = vec!["the work lands".into()];
    let c = ops::new_node(&s, c).unwrap();
    let out = ops::dispatch(
        &s,
        &c.front.id,
        vec!["src/ops.rs".into(), "src/render.rs".into(), "tests/**".into()],
        false,
        false,
        None,
        None,
        "geo",
        "t",
    )
    .unwrap();
    assert_eq!(out.globs.len(), 4, "three flags, three globs — plus the arc's own report path (it-3prx)");
    for p in ["src/ops.rs", "src/render.rs", "tests/basic.rs"] {
        assert!(
            out.globs.iter().any(|g| globs_overlap(g, p)),
            "{} is leased for real, not in name only",
            p
        );
    }
    // The predicate itself: a comma anywhere in the list is the tell, and
    // ordinary globs pass untouched.
    assert!(check_glob_shapes(&["src/**".into(), "tests/**".into()]).is_ok());
    assert!(check_glob_shapes(&[]).is_ok());
    assert!(check_glob_shapes(&["src/**".into(), "docs/a.md,docs/b.md".into()]).is_err());

    // END TO END through the real binary: the value arrives at the station
    // unsplit (nothing in the arg parse divides it), the station refuses with
    // the teaching, and the invariant holds whatever the arm — no lease may
    // exist that leases the value whole.
    let q = env!("CARGO_BIN_EXE_q");
    let run = |args: &[&str]| {
        let mut cmd = std::process::Command::new(q);
        cmd.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .env("QUARRY_SESSION", "geo")
            .args(args);
        cmd.output().unwrap()
    };
    let mut e2e = NewArgs::bare("item", "fired through the binary");
    e2e.status = Some("ready".into());
    e2e.acceptance = vec!["the work lands".into()];
    let e2e = ops::new_node(&s, e2e).unwrap();
    let out = run(&["dispatch", &e2e.front.id, "--solo", "--files", joined]);
    assert!(!out.status.success(), "the comma-joined fire refuses at the station");
    let msg = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(msg.contains("REPEATABLE"), "the repeatable flag is taught to the dispatcher: {}", msg);
    let leases = quarry::coord::load_leases(&s);
    assert!(
        !leases.iter().any(|l| l.globs.iter().any(|g| g.contains(','))),
        "never leased whole: {:?}",
        leases
    );
    // …and the corrected command fires, leasing each glob on its own.
    let ok = run(&[
        "dispatch", &e2e.front.id, "--solo",
        "--files", "src/ops.rs", "--files", "src/render.rs", "--files", "tests/**",
    ]);
    assert!(
        ok.status.success(),
        "the repeatable form fires: {}{}",
        String::from_utf8_lossy(&ok.stdout),
        String::from_utf8_lossy(&ok.stderr)
    );
    let held = quarry::coord::dispatch_for_item(&s, &e2e.front.id).unwrap();
    assert_eq!(held.globs.len(), 4, "the badge carries all three, plus the arc report: {:?}", held.globs);
    for p in ["src/ops.rs", "src/render.rs", "tests/basic.rs"] {
        assert!(
            held.globs.iter().any(|g| globs_overlap(g, p)),
            "{} is leased on the badge the agent's brief renders from",
            p
        );
    }
}

/// it-3prx: the brief demanded a report the lease forbade. A lease is derived
/// from `--files` or an item's recorded write-set, and a write-set names the
/// CODE the work touches — so the one artifact the RETURN spec mandates fell
/// outside it and the guard denied it, mid-arc, in a seat that cannot extend a
/// lease. The firing station leases the arc's own report path beside the
/// write-set: one concrete path per arc, never the zone.
#[test]
fn the_arcs_report_path_is_leased_at_the_fire_and_registers_from_the_agents_seat() {
    use quarry::coord::{arc_report_in, globs_overlap, is_arc_report, set_arc_report};
    use quarry::teach::{lease_check, LeaseCheck};
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "the geo pass");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    let disp = vec![it.front.id.clone()];

    // THE DEFECT, measured on the lease shape the station used to hand out:
    // code globs alone, and the report lands outside them — denied under the
    // badge with the extend-the-lease line, in the one seat that cannot.
    let code_only = vec![quarry::coord::Lease {
        item: it.front.id.clone(),
        item_title: "the geo pass".into(),
        session: "geo".into(),
        actor: "t".into(),
        globs: vec!["src/geo/**".into()],
        shared: false,
        since: "now".into(),
    }];
    assert!(
        matches!(
            lease_check(&code_only, None, Some(it.front.id.as_str()), &disp, "docs/reports/r.md"),
            LeaseCheck::Deny(_)
        ),
        "the pre-fix shape: the brief's own artifact is outside the brief's own write-set"
    );

    // THE FIRE: the dispatch leases a concrete report path beside the
    // write-set the dispatcher named.
    let out = ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "geo", "t").unwrap();
    let path = arc_report_in(&out.globs).expect("the arc's report path is leased").to_string();
    assert!(path.starts_with("docs/reports/") && path.ends_with(".md"), "{}", path);
    assert!(path.contains(&it.front.id), "the item id keeps concurrent arcs apart: {}", path);
    assert!(out.globs.contains(&"src/geo/**".to_string()), "the write-set is untouched: {:?}", out.globs);
    let leases = quarry::coord::load_leases(&s);
    assert!(
        matches!(lease_check(&leases, None, Some(it.front.id.as_str()), &disp, &path), LeaseCheck::Allow),
        "the guard now admits the artifact the contract demands"
    );
    assert!(
        matches!(
            lease_check(&leases, None, Some(it.front.id.as_str()), &disp, "docs/reports/someone-elses.md"),
            LeaseCheck::Deny(_)
        ),
        "one path, never the zone — a neighbour's return stays out of reach"
    );

    // NO CO-WRITE ZONE: a second arc fires from another session, and the two
    // returns never overlap. Leasing `docs/reports/**` instead would have
    // refused this fire outright (C7) and handed each agent write access to
    // the other's return.
    let mut other = NewArgs::bare("item", "the hydro pass");
    other.status = Some("ready".into());
    other.about = vec![area.front.id.clone()];
    other.acceptance = vec!["the pass lands".into()];
    let other = ops::new_node(&s, other).unwrap();
    let o = ops::dispatch(&s, &other.front.id, vec!["src/hydro/**".into()], false, false, None, None, "ops", "t").unwrap();
    let opath = arc_report_in(&o.globs).expect("the second arc leases its own").to_string();
    assert_ne!(opath, path);
    assert!(!globs_overlap(&path, &opath), "two arcs' returns never overlap: {} vs {}", path, opath);
    // …and a write-set that COVERS the reports zone still fires beside a live
    // arc. A return is never contended ground: counted at the C7 test, every
    // `docs/**` dispatch would refuse while any other arc flies.
    let mut docs = NewArgs::bare("item", "the docs pass");
    docs.status = Some("ready".into());
    docs.about = vec![area.front.id.clone()];
    docs.acceptance = vec!["the docs land".into()];
    let docs = ops::new_node(&s, docs).unwrap();
    let d = ops::dispatch(&s, &docs.front.id, vec!["docs/**".into()], false, false, None, None, "scribe", "t")
        .expect("a docs write-set fires beside a live arc's leased return");
    assert!(d.globs.contains(&"docs/**".to_string()), "{:?}", d.globs);

    // THE BRIEF names the path and hands over the registration command,
    // filled in — the agent memorizes nothing.
    let b = quarry::render::brief(&s, &it.front.id).unwrap();
    assert!(b.contains(&path), "the RETURN spec names the leased path: {}", b);
    assert!(
        b.contains(&format!(
            "--kind report --path {} --about {} --supports {}",
            path, area.front.id, it.front.id
        )),
        "the registration command is delivered whole: {}",
        b
    );

    // HARVEST, before the arc returns: the exact path rides the homework, and
    // an unwritten report is never "dead weight in the lease".
    let h0 = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(h0.contains(&format!("--kind report --path {}", path)), "{}", h0);
    assert!(!h0.contains("leased but untouched: docs/reports"), "{}", h0);
    assert!(h0.contains(&quarry::framings::user_owned_await(0)), "no return yet: {}", h0);

    // THE AGENT'S OWN SEAT: it writes the report at the leased path and
    // registers the doc itself.
    std::fs::create_dir_all(s.root.join("docs/reports")).unwrap();
    std::fs::write(
        s.root.join(&path),
        "outcomes hold\n\nuser-owned calls: none\n\nREFLECTIONS: fine\n",
    )
    .unwrap();
    quarry::coord::accrue_touch(&s, &format!("item:{}", it.front.id), &path);
    let mut doc = NewArgs::bare("doc", "dispatch report: the geo pass");
    doc.kind = Some("report".into());
    doc.path = Some(path.clone());
    let doc = ops::new_node(&s, doc).unwrap();
    ops::link(&s, &doc.front.id, "supports", &it.front.id, false, None).unwrap();

    // …and harvest reconciles what the agent declared, with no dispatcher's
    // hand in the registration — and asks for nothing it already holds.
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(
        h.contains(&quarry::framings::user_owned_reconcile(Some(0), 0)),
        "the declaration is mechanical now, not the dispatcher's good faith: {}",
        h
    );
    assert!(!h.contains(&quarry::framings::user_owned_await(0)), "the await arm is gone: {}", h);
    assert!(h.contains("registered from the agent's own seat"), "{}", h);
    assert!(h.contains(&doc.front.id), "the registered report is named at the seat that judges it: {}", h);
    assert!(!h.contains("q new doc"), "no second doc node is invited for one report: {}", h);
    assert!(!h.contains("⚠ outside the lease"), "the report reads as leased work: {}", h);

    // A RE-DISPATCH is a new arc with its own return: the lease re-points, so
    // arc two can never overwrite the file arc one registered.
    let again = ops::dispatch(&s, &it.front.id, vec![], false, false, None, None, "geo", "t").unwrap();
    let p2 = arc_report_in(&again.globs).expect("arc two leases its own return").to_string();
    assert_ne!(p2, path, "a re-dispatch never inherits its predecessor's path");
    assert!(p2.contains("-arc2"), "{}", p2);
    let leases = quarry::coord::load_leases(&s);
    let live = leases.iter().find(|l| l.item == it.front.id).unwrap();
    assert!(live.globs.contains(&"src/geo/**".to_string()), "the code write-set survives the re-point: {:?}", live.globs);
    assert!(
        matches!(lease_check(&leases, None, Some(it.front.id.as_str()), &disp, &path), LeaseCheck::Deny(_)),
        "arc one's registered report is out of arc two's reach"
    );
    assert!(matches!(lease_check(&leases, None, Some(it.front.id.as_str()), &disp, &p2), LeaseCheck::Allow));

    // THE PURE MOVES: a prior arc's path drops when the new one lands, one
    // return per arc, and a dispatcher's own broader glob is neither mistaken
    // for an arc's return nor thrown away.
    let mut g = vec!["src/**".to_string(), "docs/reports/**".to_string()];
    assert!(set_arc_report(&mut g, "docs/reports/a.md"));
    assert_eq!(arc_report_in(&g), Some("docs/reports/a.md"), "a pattern never stands in for a path: {:?}", g);
    assert!(g.contains(&"docs/reports/**".to_string()), "the dispatcher's own glob survives: {:?}", g);
    assert!(!set_arc_report(&mut g, "docs/reports/a.md"), "idempotent: the same path changes nothing");
    assert!(set_arc_report(&mut g, "docs/reports/b.md"));
    assert_eq!(g.iter().filter(|x| is_arc_report(x)).count(), 1, "one return per arc: {:?}", g);
    assert!(g.contains(&"src/**".to_string()), "the code write-set is never touched: {:?}", g);
}

/// it-2eqk: the arc's own return can never satisfy the land-time landmark
/// check. With the report path leased (cl-ue2e) the agent's report is an
/// in-repo write under the badge, so it accrues into the observed set like any
/// other touch — and `vein_check` hands the observed set to `files_cited`,
/// which counts a DOC WHOSE OWN PATH falls inside the checked globs as a
/// citation. The registered report is exactly such a doc, so every dispatched
/// landing whose agent registered its return satisfied the check trivially and
/// the prompt of dc-grrb went silent on the one class of landing it was
/// written for.
#[test]
fn the_arcs_own_return_never_satisfies_the_land_time_landmark_check() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    let area = ops::new_node(&s, NewArgs::bare("area", "geology")).unwrap();
    let mut it = NewArgs::bare("item", "the geo pass");
    it.status = Some("ready".into());
    it.about = vec![area.front.id.clone()];
    it.acceptance = vec!["the pass lands".into()];
    let it = ops::new_node(&s, it).unwrap();
    let out =
        ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, None, "design", "t")
            .unwrap();
    let path = quarry::coord::arc_report_in(&out.globs)
        .expect("the arc's report path is leased")
        .to_string();

    // What the guard actually sees a dispatched arc touch: the code it
    // repaired, and the return it was leased to write.
    let observed = vec!["src/geo/pass.rs".to_string(), path.clone()];
    let all = s.load_all().unwrap();
    assert!(
        !queries::files_cited(&all, &observed),
        "before the return registers, nothing in the graph cites either path"
    );

    // THE AGENT'S SEAT: it writes its report at the leased path and registers
    // the doc the RETURN spec demands — and changes NOTHING else.
    std::fs::create_dir_all(s.root.join("docs/reports")).unwrap();
    std::fs::write(s.root.join(&path), "outcomes hold\n\nuser-owned calls: none\n").unwrap();
    let mut doc = NewArgs::bare("doc", "dispatch report: the geo pass");
    doc.kind = Some("report".into());
    doc.path = Some(path.clone());
    let doc = ops::new_node(&s, doc).unwrap();
    ops::link(&s, &doc.front.id, "supports", &it.front.id, false, None).unwrap();

    // THE DEFECT, measured: that one registration — no claim, no file edge,
    // nothing said about the code — flips the unfiltered check to satisfied.
    let all = s.load_all().unwrap();
    assert!(
        queries::files_cited(&all, &observed),
        "the pre-fix shape: the registered report is a doc sitting inside the observed set, so the arc cites itself"
    );

    // THE FIX at the consumer: the return drops before the check, and the code
    // the arc actually landed is still uncited — the prompt has its say.
    let judged = queries::landmark_globs(&observed);
    assert_eq!(
        judged,
        vec!["src/geo/pass.rs".to_string()],
        "the return is accounting, never a landed capability: {:?}",
        judged
    );
    assert!(!queries::files_cited(&all, &judged), "the code the arc touched is uncited");

    // THE REPORT PATH ALONE never satisfies the check: judged, the set is
    // empty, so the check has nothing to ask about — never a citation.
    assert!(
        queries::landmark_globs(&[path.clone()]).is_empty(),
        "an arc that touched only its own return landed no code to cite"
    );
    // A dispatcher's own broader glob is not an arc's return (coord::is_arc_report)
    // and survives: it was authored as write-set, so a doc under it really cites.
    assert_eq!(
        queries::landmark_globs(&["docs/reports/**".to_string()]),
        vec!["docs/reports/**".to_string()],
        "the dispatcher's own glob is the work, not the paperwork"
    );

    // END TO END at the landing seat: the dispatcher lands the item, the
    // observed set carries both touches, and the prompt still speaks — naming
    // the code that landed and never the paperwork about it.
    let key = format!("item:{}", it.front.id);
    quarry::coord::accrue_touch(&s, &key, "src/geo/pass.rs");
    quarry::coord::accrue_touch(&s, &key, &path);
    let o = run(&[("QUARRY_SESSION", "design")], &["set", &it.front.id, "status=done"]);
    let text = String::from_utf8_lossy(&o.stdout).to_string();
    let uncited = text
        .lines()
        .find(|l| l.contains("landed uncited"))
        .unwrap_or_else(|| panic!("a dispatched landing still draws the prompt: {}", text));
    assert!(uncited.contains("src/geo/pass.rs"), "the code wants a vein: {}", uncited);
    assert!(
        !uncited.contains(&path),
        "the arc's return never appears among the files asked for a vein: {}",
        uncited
    );

    // THE SECOND CONSUMER, the wrap backstop, takes the same exclusion. It is
    // reachable: harvest clears the BADGE, never the lease, so a dispatcher who
    // lands without releasing reaches the boundary with the arc's report path
    // still in the held globs — and there the backstop reads the lease, not the
    // observed set.
    let h = run(&[("QUARRY_SESSION", "design")], &["harvest", &it.front.id]);
    assert!(h.status.success(), "harvest: {}", String::from_utf8_lossy(&h.stderr));
    let live = quarry::coord::load_leases(&s);
    let held = live.iter().find(|l| l.item == it.front.id).expect("the lease outlives the harvest");
    assert!(held.globs.contains(&path), "the held globs still carry the arc's return: {:?}", held.globs);
    let w = run(&[("QUARRY_SESSION", "design")], &["wrap"]);
    let wtext = String::from_utf8_lossy(&w.stdout).to_string();
    let backstop = wtext
        .lines()
        .find(|l| l.contains("landed uncited") && l.contains("held"))
        .unwrap_or_else(|| panic!("the boundary backstop still speaks: {}", wtext));
    assert!(backstop.contains("src/geo/**"), "the code wants a vein: {}", backstop);
    assert!(
        !backstop.contains(&path),
        "the arc's return is not dead weight to answer for at the boundary: {}",
        backstop
    );
}

#[test]
fn a_dispatched_arc_files_under_the_model_it_was_spawned_on() {
    // it-xcvb. The chat-actor map is keyed by CHAT and written at
    // SessionStart; a subagent is not a chat and never fires SessionStart, so
    // a dispatched agent's shells inherited the DISPATCHER's model and every
    // node the agent minted filed under a model that did not write it. The
    // harness offers no cure: measured 2026-08-21, a PreToolUse firing inside
    // a subagent carries session_id, transcript_path, cwd, prompt_id,
    // permission_mode, agent_id, agent_type, effort and the tool fields —
    // and no model at all. The dispatcher is the one party that knows, so the
    // badge carries the stamp.
    let s = temp_store();
    quarry::coord::record_chat_actor(&s, "chat-disp", "claude-fable-5");
    let area = ops::new_node(&s, NewArgs::bare("area", "attribution")).unwrap();
    let ready = |title: &str| {
        let mut a = NewArgs::bare("item", title);
        a.status = Some("ready".into());
        a.about = vec![area.front.id.clone()];
        a.acceptance = vec!["the arc lands".into()];
        ops::new_node(&s, a).unwrap()
    };

    // THE DEFECT IN SHAPE: a fire that names no model stamps none, so the
    // badge overrides nothing and the hook road below the stamp answers. The
    // fire says which of the two the dispatcher got, at the one station that
    // can still fix it — amended at it-xwpw: the unstamped answer is no
    // longer the dispatching chat's actor, so the fire names no model there
    // at all (a_bare_fire_names_no_model_and_the_join_names_what_it_read).
    let plain = ready("inherited arc");
    let bare = ops::dispatch(
        &s, &plain.front.id, vec!["src/a/**".into()], false, false, None, None, "disp", "claude-fable-5",
    )
    .unwrap();
    assert!(bare.model.is_none(), "nothing stamped when nothing was named");
    assert_eq!(bare.arc_actor, None, "an unstamped fire has no model to name");
    let jb = ops::join(&s, &bare.token, Some("agent:ag-inherit".into())).unwrap();
    assert_ne!(jb.actor_source, ops::ArcActorSource::Stamp, "an unstamped badge overrides nothing");
    assert_eq!(
        quarry::coord::badge_model(&quarry::coord::load_dispatches(&s), "ag-inherit"),
        None,
        "no stamp, no override — the hook falls through to the chat-actor map, the defect's own road"
    );

    // THE CURE: the fire names the model it is spawning on, the badge carries
    // it, and the joined AGENT identity resolves it.
    let it = ready("opus arc");
    let out = ops::dispatch(
        &s,
        &it.front.id,
        vec!["src/b/**".into()],
        false,
        false,
        None,
        Some("claude-opus-5"),
        "disp",
        "claude-fable-5",
    )
    .unwrap();
    assert_eq!(out.model.as_deref(), Some("claude-opus-5"));
    assert_eq!(
        out.arc_actor.as_deref(),
        Some("claude-opus-5"),
        "the fire states what the arc will file under"
    );
    let d = quarry::coord::dispatch_for_item(&s, &it.front.id).unwrap();
    assert_eq!(d.model.as_deref(), Some("claude-opus-5"), "the stamp rides the badge, not the log alone");

    let j = ops::join(&s, &out.token, Some("agent:ag-opus".into())).unwrap();
    assert_eq!(j.actor_source, ops::ArcActorSource::Stamp);
    assert_eq!(j.arc_actor, "claude-opus-5", "the join states it to the one mind that knows its own model");
    // The join event is the whole of the pre-bind window: the hook fired
    // before the bind existed, so that shell's QUARRY_ACTOR is still the
    // dispatcher's. Stamped here from the badge directly, the arc's FIRST
    // badged act already files right.
    let log = s.read_log().unwrap();
    let join_ev = log
        .iter()
        .rev()
        .find(|e| e.get("op").and_then(|v| v.as_str()) == Some("join"))
        .expect("the bind logs a join");
    assert_eq!(
        join_ev.get("actor").and_then(|v| v.as_str()),
        Some("claude-opus-5"),
        "the arc's first act files under the model that made it, not the one that sent it: {}",
        join_ev
    );
    let dispatch_ev = log
        .iter()
        .rev()
        .find(|e| {
            e.get("op").and_then(|v| v.as_str()) == Some("dispatch")
                && e.get("node").and_then(|v| v.as_str()) == Some(it.front.id.as_str())
        })
        .expect("the fire logs");
    assert_eq!(
        dispatch_ev.get("model").and_then(|v| v.as_str()),
        Some("claude-opus-5"),
        "the log carries the stamp too — the badge dies at harvest, the log is the durable seat: {}",
        dispatch_ev
    );

    // KEYED ON THE AGENT ALONE. Every other identity is a real chat, which
    // fired SessionStart and already has its own model recorded; reading a
    // chat-keyed acting row would buy nothing and would spread the badge's
    // model onto the dispatching chat's own shells wherever parent and
    // subagent are indistinguishable.
    let m = quarry::coord::load_dispatches(&s);
    assert_eq!(quarry::coord::badge_model(&m, "ag-opus").as_deref(), Some("claude-opus-5"));
    assert_eq!(quarry::coord::badge_model(&m, "ag-stranger"), None, "a foreign agent resolves nothing");
    let chatted = ready("chat-joined arc");
    let co = ops::dispatch(
        &s, &chatted.front.id, vec!["src/c/**".into()], false, false, None, Some("claude-opus-5"), "disp",
        "claude-fable-5",
    )
    .unwrap();
    let cj = ops::join(&s, &co.token, Some("chat:chat-elsewhere".into())).unwrap();
    assert_eq!(
        cj.actor_source,
        ops::ArcActorSource::Injected,
        "a chat-keyed joiner keeps its own recorded model"
    );
    assert_eq!(
        quarry::coord::badge_model(&quarry::coord::load_dispatches(&s), "chat-elsewhere"),
        None,
        "the resolver never reads a chat row, however it is spelled"
    );

    // RESIDUE IS NEVER A BINDING. live_acting_badge already refuses a row
    // pointing at a cleared dispatch, so a harvested arc stops overriding by
    // construction — the next thing that agent does files under its chat
    // again, not under a dead badge's model.
    quarry::coord::clear_dispatch(&s, &it.front.id);
    assert_eq!(
        quarry::coord::badge_model(&quarry::coord::load_dispatches(&s), "ag-opus"),
        None,
        "harvest frees the actor with the badge"
    );

    // THE PROVENANCE GUARD STILL RUNS. Derivation keys on "claude" in the
    // actor string, so a display-name model would otherwise read as USER
    // provenance — the stamp goes through safe_actor like every other
    // injected actor.
    let named = ready("display-name arc");
    let dn = ops::dispatch(
        &s, &named.front.id, vec!["src/d/**".into()], false, false, None, Some("Opus 5"), "disp",
        "claude-fable-5",
    )
    .unwrap();
    assert_eq!(
        dn.arc_actor.as_deref(),
        Some("claude:Opus 5"),
        "a display name cannot mint user provenance"
    );
    let dj = ops::join(&s, &dn.token, Some("agent:ag-display".into())).unwrap();
    assert_eq!(dj.arc_actor, "claude:Opus 5");
    assert_eq!(
        quarry::coord::badge_actor(&s, Some("ag-display")).as_deref(),
        Some("claude:Opus 5"),
        "the store-reading half is the same value the hook injects"
    );
    assert_eq!(quarry::coord::badge_actor(&s, None), None, "no agent id, no override");
}

#[test]
fn the_session_hook_injects_the_badge_model_over_the_inherited_chat_actor() {
    // it-xcvb, end to end through the spawned binary — the only seat where
    // QUARRY_ACTOR can honestly be absent from the environment, which is the
    // state a real hook process runs in.
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    quarry::coord::record_chat_actor(&s, "chat-disp", "claude-fable-5");
    let area = ops::new_node(&s, NewArgs::bare("area", "attribution")).unwrap();
    let mut a = NewArgs::bare("item", "the spawned arc");
    a.status = Some("ready".into());
    a.about = vec![area.front.id.clone()];
    a.acceptance = vec!["the arc lands".into()];
    let it = ops::new_node(&s, a).unwrap();

    let hook = |input: &str| {
        use std::io::Write as _;
        let mut c = std::process::Command::new(q)
            .current_dir(&s.root)
            .args(["hook", "session"])
            .env_remove("QUARRY_ACTOR")
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        c.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
        let o = c.wait_with_output().unwrap();
        let text = String::from_utf8_lossy(&o.stdout).to_string();
        let v: serde_json::Value = serde_json::from_str(text.trim()).unwrap_or_else(|e| {
            panic!(
                "hook output not JSON ({}): {} / stderr {}",
                e,
                text,
                String::from_utf8_lossy(&o.stderr)
            )
        });
        v["hookSpecificOutput"]["updatedInput"]["command"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };
    let sub = r#"{"session_id":"chat-disp","agent_id":"ag-opus","tool_name":"Bash","tool_input":{"command":"q open it-x"}}"#;
    let parent = r#"{"session_id":"chat-disp","tool_name":"Bash","tool_input":{"command":"q open it-x"}}"#;

    // THE DEFECT, MEASURED BEFORE THE STAMP EXISTS: the subagent's shell and
    // its dispatcher's shell are handed the SAME actor, because the only map
    // the hook could read is keyed by the chat they share.
    let before = hook(sub);
    assert!(before.contains("QUARRY_ACTOR='claude-fable-5'"), "unstamped: the subagent inherits — {}", before);
    assert!(
        hook(parent).contains("QUARRY_ACTOR='claude-fable-5'"),
        "the dispatcher's own shell, for contrast"
    );

    // THE CURE: fire naming the model, join as the agent, and that agent's
    // shells file under the model that spawned them.
    let out = ops::dispatch(
        &s,
        &it.front.id,
        vec!["src/**".into()],
        false,
        false,
        None,
        Some("claude-opus-5"),
        "disp",
        "claude-fable-5",
    )
    .unwrap();
    ops::join(&s, &out.token, Some("agent:ag-opus".into())).unwrap();
    let after = hook(sub);
    assert!(
        after.contains("QUARRY_ACTOR='claude-opus-5'"),
        "the joined agent files under the model that wrote it: {}",
        after
    );
    assert!(!after.contains("claude-fable-5"), "and never under the one that sent it: {}", after);
    assert!(
        after.contains("QUARRY_AGENT='ag-opus'") && after.contains("QUARRY_CHAT='chat-disp'"),
        "the identity injections are undisturbed beside it: {}",
        after
    );
    // The dispatching chat's OWN shells are untouched — the override is keyed
    // on the agent id, so the badge's model can never leak back up the wire
    // to the seat that fired it.
    let par = hook(parent);
    assert!(
        par.contains("QUARRY_ACTOR='claude-fable-5'"),
        "the dispatcher keeps its own model while its agent flies: {}",
        par
    );

    // Harvest frees it: the acting row survives as residue and resolves
    // nothing, so the same agent's next shell inherits again.
    quarry::coord::clear_dispatch(&s, &it.front.id);
    let cleared = hook(sub);
    assert!(
        cleared.contains("QUARRY_ACTOR='claude-fable-5'"),
        "a cleared badge stops overriding: {}",
        cleared
    );
}

#[test]
fn the_chat_actor_row_refreshes_from_the_transcript_at_every_fire() {
    // it-j4tx. The row is written ONCE, at SessionStart, and used to be read
    // back unquestioned: a `/model` switch or a resume onto a different model
    // left the whole rest of the session filing under a model that had
    // stopped writing it. Measured on the dispatching chat itself, 2026-08-21
    // — the row said claude-fable-5 while all 400 assistant entries in that
    // chat's own transcript said claude-opus-5.
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");

    // THE PURE CORE, on the incident shape: a session that started on one
    // model and switched. The LAST assistant entry wins, whatever came first.
    let switched = concat!(
        r#"{"type":"assistant","message":{"model":"claude-fable-5"},"isSidechain":false}"#,
        "\n",
        r#"{"type":"user","message":{"role":"user"}}"#,
        "\n",
        r#"{"type":"assistant","message":{"model":"claude-opus-5"},"isSidechain":false}"#,
        "\n",
        r#"{"type":"system","subtype":"stop_hook_summary"}"#,
        "\n"
    );
    assert_eq!(
        quarry::coord::last_assistant_model(switched.as_bytes(), false).as_deref(),
        Some("claude-opus-5"),
        "the last assistant entry names the model producing turns now"
    );

    // THE FILTERS, each a measured shape rather than a guess.
    // `<synthetic>` is what the harness writes for assistant entries it
    // composed itself — measured in 4 of the 23 real transcripts on this
    // machine — and it is not a model anyone can be attributed to.
    let synthetic = concat!(
        r#"{"type":"assistant","message":{"model":"claude-opus-5"}}"#,
        "\n",
        r#"{"type":"assistant","message":{"model":"<synthetic>"}}"#,
        "\n"
    );
    assert_eq!(
        quarry::coord::last_assistant_model(synthetic.as_bytes(), false).as_deref(),
        Some("claude-opus-5"),
        "a synthetic entry is skipped, never adopted as a model name"
    );
    // A subagent's turns carry isSidechain:true (measured: all 78 assistant
    // entries of this arc's own subagent transcript). They are never the
    // chat's model, in whatever file they land in.
    let sidechain = concat!(
        r#"{"type":"assistant","message":{"model":"claude-fable-5"},"isSidechain":false}"#,
        "\n",
        r#"{"type":"assistant","message":{"model":"claude-opus-5"},"isSidechain":true}"#,
        "\n"
    );
    assert_eq!(
        quarry::coord::last_assistant_model(sidechain.as_bytes(), false).as_deref(),
        Some("claude-fable-5"),
        "a subagent's turn never becomes the chat's model"
    );
    // The tail begins mid-file, so its first line is a fragment. Dropped —
    // and dropping it is the whole of what `truncated` means. The fragment
    // here is deliberately one that PARSES: a half-line usually parses to
    // nothing and would be skipped anyway, so a shape that survives the
    // parser is what actually pins the rule.
    let fragment = concat!(
        r#"{"type":"assistant","message":{"model":"claude-fragment-5"}}"#,
        "\n",
        r#"{"type":"user","message":{"role":"user"}}"#,
        "\n"
    );
    assert_eq!(
        quarry::coord::last_assistant_model(fragment.as_bytes(), true),
        None,
        "the first line of a truncated tail is never read as an entry"
    );
    assert_eq!(
        quarry::coord::last_assistant_model(fragment.as_bytes(), false).as_deref(),
        Some("claude-fragment-5"),
        "…and is read as one when the tail is the whole file"
    );
    assert_eq!(
        quarry::coord::last_assistant_model(b"", false),
        None,
        "an empty tail answers nothing"
    );
    assert_eq!(
        quarry::coord::last_assistant_model(b"not json at all\n{\"a\":1}\n", false),
        None,
        "unreadable lines are skipped, never guessed at"
    );

    // THE DEFECT, MEASURED ON THE DERIVATION. The recorded row is the model
    // SessionStart saw; the transcript is the model writing now.
    let tdir = s.root.join("transcripts");
    std::fs::create_dir_all(&tdir).unwrap();
    let tpath = tdir.join("chat-live.jsonl");
    std::fs::write(&tpath, switched).unwrap();
    quarry::coord::record_chat_actor(&s, "chat-live", "claude-fable-5");
    assert_eq!(
        quarry::coord::chat_actor(&s, "chat-live").as_deref(),
        Some("claude-fable-5"),
        "the stale row, exactly as SessionStart left it"
    );
    let tp = tpath.display().to_string();
    assert_eq!(
        quarry::coord::refreshed_chat_actor(&s, "chat-live", Some(&tp)).as_deref(),
        Some("claude-opus-5"),
        "the fire reads the model actually producing turns"
    );
    // And the row itself is corrected, so the record on disk stops lying.
    assert_eq!(
        quarry::coord::chat_actor(&s, "chat-live").as_deref(),
        Some("claude-opus-5"),
        "the refresh writes back — the row is a record, not a cache of one moment"
    );

    // NO TRANSCRIPT, NO REGRESSION: the recorded row stands and attribution
    // keeps its old SessionStart grain. A hook must never fail a shell over
    // an undocumented file format, and it must never fall to "claude" when it
    // already holds a better answer.
    quarry::coord::record_chat_actor(&s, "chat-quiet", "claude-fable-5");
    assert_eq!(
        quarry::coord::refreshed_chat_actor(&s, "chat-quiet", None).as_deref(),
        Some("claude-fable-5"),
        "no transcript path: the recorded row stands"
    );
    assert_eq!(
        quarry::coord::refreshed_chat_actor(&s, "chat-quiet", Some("B:/nope/missing.jsonl"))
            .as_deref(),
        Some("claude-fable-5"),
        "a transcript that cannot be read: the recorded row stands"
    );
    let empty = tdir.join("empty.jsonl");
    std::fs::write(&empty, "").unwrap();
    assert_eq!(
        quarry::coord::refreshed_chat_actor(&s, "chat-quiet", Some(&empty.display().to_string()))
            .as_deref(),
        Some("claude-fable-5"),
        "a transcript with no assistant entry: the recorded row stands"
    );

    // THE TAIL IS READ FROM THE END, so the cost does not grow with the
    // session. A transcript padded past the window still answers, and the
    // answer comes from its end rather than its beginning.
    let big = tdir.join("big.jsonl");
    let pad = format!(
        "{}\n",
        serde_json::json!({"type": "user", "message": {"role": "user", "content": "x".repeat(4096)}})
    );
    let mut body = String::new();
    body.push_str(r#"{"type":"assistant","message":{"model":"claude-ancient-1"}}"#);
    body.push('\n');
    while body.len() < (quarry::coord::TRANSCRIPT_TAIL_BYTES as usize) * 2 {
        body.push_str(&pad);
    }
    body.push_str(r#"{"type":"assistant","message":{"model":"claude-opus-5"}}"#);
    body.push('\n');
    std::fs::write(&big, &body).unwrap();
    assert_eq!(
        quarry::coord::transcript_model(&big).as_deref(),
        Some("claude-opus-5"),
        "a transcript larger than the window is read from its end"
    );

    // END TO END THROUGH THE SPAWNED BINARY — the only seat where
    // QUARRY_ACTOR is honestly absent from the environment, which is the
    // state a real hook process runs in.
    let hook = |input: &str| {
        use std::io::Write as _;
        let mut c = std::process::Command::new(q)
            .current_dir(&s.root)
            .args(["hook", "session"])
            .env_remove("QUARRY_ACTOR")
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        c.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
        let o = c.wait_with_output().unwrap();
        let text = String::from_utf8_lossy(&o.stdout).to_string();
        let v: serde_json::Value = serde_json::from_str(text.trim()).unwrap_or_else(|e| {
            panic!(
                "hook output not JSON ({}): {} / stderr {}",
                e,
                text,
                String::from_utf8_lossy(&o.stderr)
            )
        });
        v["hookSpecificOutput"]["updatedInput"]["command"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };
    let esc = tp.replace('\\', "\\\\");
    quarry::coord::record_chat_actor(&s, "chat-switch", "claude-fable-5");
    let stale =
        r#"{"session_id":"chat-switch","tool_name":"Bash","tool_input":{"command":"q open it-x"}}"#;
    assert!(
        hook(stale).contains("QUARRY_ACTOR='claude-fable-5'"),
        "the defect's own shape end to end: no transcript in the payload, the stale row rides"
    );
    let live = format!(
        r#"{{"session_id":"chat-switch","transcript_path":"{}","tool_name":"Bash","tool_input":{{"command":"q open it-x"}}}}"#,
        esc
    );
    let out = hook(&live);
    assert!(
        out.contains("QUARRY_ACTOR='claude-opus-5'"),
        "the switched-to model reaches the shell: {}",
        out
    );
    assert!(
        !out.contains("claude-fable-5"),
        "and the model that stopped writing this session does not: {}",
        out
    );

    // THE BADGE STILL OUTRANKS IT (cl-dqt4). A joined subagent's model comes
    // from the stamp its dispatcher made, and the transcript the hook is
    // handed inside a subagent is the PARENT CHAT's — measured 2026-08-21 —
    // so reading it there would file the agent's work under the chat that
    // spawned it. The order is: badge, then the refreshed chat row.
    let area = ops::new_node(&s, NewArgs::bare("area", "attribution-live")).unwrap();
    let mut a = NewArgs::bare("item", "the arc under a switched chat");
    a.status = Some("ready".into());
    a.about = vec![area.front.id.clone()];
    a.acceptance = vec!["the arc lands".into()];
    let it = ops::new_node(&s, a).unwrap();
    let d = ops::dispatch(
        &s,
        &it.front.id,
        vec!["src/**".into()],
        false,
        false,
        None,
        Some("claude-haiku-5"),
        "disp",
        "claude-opus-5",
    )
    .unwrap();
    ops::join(&s, &d.token, Some("agent:ag-live".into())).unwrap();
    let sub = format!(
        r#"{{"session_id":"chat-switch","agent_id":"ag-live","transcript_path":"{}","tool_name":"Bash","tool_input":{{"command":"q open it-x"}}}}"#,
        esc
    );
    let subout = hook(&sub);
    assert!(
        subout.contains("QUARRY_ACTOR='claude-haiku-5'"),
        "the badge stamp outranks the chat's live model: {}",
        subout
    );
    // …and the chat's own shells still get the chat's live model beside it.
    assert!(
        hook(&live).contains("QUARRY_ACTOR='claude-opus-5'"),
        "the dispatcher's own shell keeps its own live model"
    );

    // THE RESIDUE IS STATED WHERE THE ACTOR IS EXPLAINED. The refresh road is
    // the one taken, so the row normally names the model writing now — but
    // when the transcript cannot answer, the recorded row stands and it is
    // SessionStart-grained. The guide's ENVIRONMENT section is the one place
    // that explains the injected actor to an agent, so it is where the grain
    // has to be readable rather than inferable.
    let g = quarry::teach::GUIDE;
    assert!(
        g.contains("re-derived at EVERY fire"),
        "the guide states that the actor refreshes per fire"
    );
    assert!(
        g.contains("SessionStart-grained"),
        "the guide names the grain the fallback keeps"
    );
}

#[test]
fn a_subagents_model_resolves_from_the_harness_record_keyed_by_its_agent_id() {
    // it-6ekf. cl-dqt4 put ATTRIBUTION on the discipline road it had just
    // taken IDENTITY off: q dispatch --model is a flag the dispatcher must
    // remember on every fire, and omitting it while spawning on another model
    // reproduces the original defect silently — which is what dc-zbxj's
    // "identity is structural, never discipline" rules against and what
    // th-e5ez asks the user to settle. The road cl-dqt4 could not find is on
    // disk: the harness writes each subagent's own transcript AND a spawn
    // sidecar beside the chat's transcript, keyed by the very agent id the
    // session hook already injects (cl-z6gc). Nothing has to arrive in a hook
    // payload; nothing has to be remembered.
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");

    // THE PURE CORE, on the shape that made the reuse non-trivial. cl-cv92's
    // sidechain filter is LOAD-BEARING over a chat's file and FATAL over a
    // subagent's: measured across the 268 subagent transcripts on this
    // machine, every one of the 261 holding a readable assistant entry
    // carries isSidechain:true on ALL of them, with no non-sidechain entry
    // anywhere among them. Same bytes, two answers, by whose file it is.
    let agent_tail = concat!(
        r#"{"type":"user","message":{"role":"user"},"isSidechain":true}"#,
        "\n",
        r#"{"type":"assistant","message":{"model":"claude-opus-5"},"isSidechain":true}"#,
        "\n"
    );
    assert_eq!(
        quarry::coord::last_agent_model(agent_tail.as_bytes(), false).as_deref(),
        Some("claude-opus-5"),
        "in a subagent's own file the sidechain mark is native, not foreign"
    );
    assert_eq!(
        quarry::coord::last_assistant_model(agent_tail.as_bytes(), false),
        None,
        "the chat scan is unchanged: a subagent's turn is never the chat's model"
    );
    // Every other filter still runs on the relaxed road — the format is
    // undocumented either way, so relaxing ONE judgment relaxes only it.
    let synthetic = concat!(
        r#"{"type":"assistant","message":{"model":"claude-opus-5"},"isSidechain":true}"#,
        "\n",
        r#"{"type":"assistant","message":{"model":"<synthetic>"},"isSidechain":true}"#,
        "\n"
    );
    assert_eq!(
        quarry::coord::last_agent_model(synthetic.as_bytes(), false).as_deref(),
        Some("claude-opus-5"),
        "a synthetic entry is skipped on the agent road too"
    );
    // …and a truncated tail still drops its fragment first line. The fragment
    // here is deliberately one that PARSES, so the rule is what drops it.
    let fragment = concat!(
        r#"{"type":"assistant","message":{"model":"claude-fragment-5"},"isSidechain":true}"#,
        "\n",
        r#"{"type":"user","message":{"role":"user"},"isSidechain":true}"#,
        "\n"
    );
    assert_eq!(quarry::coord::last_agent_model(fragment.as_bytes(), true), None);
    assert_eq!(
        quarry::coord::last_agent_model(fragment.as_bytes(), false).as_deref(),
        Some("claude-fragment-5"),
        "…and is read as an entry when the tail is the whole file"
    );

    // THE LAYOUT, derived from the one path a subagent's hook is handed: the
    // PARENT CHAT's transcript (cl-cv92, measured). The chat dir sits beside
    // the chat file, named for it without the suffix.
    let projects = s.root.join("projects");
    let chat_file = projects.join("chat-chatty.jsonl");
    let subdir = projects.join("chat-chatty").join("subagents");
    std::fs::create_dir_all(&subdir).unwrap();
    let (jsonl, meta) = quarry::coord::subagent_records(&chat_file, "ag-sub").unwrap();
    assert_eq!(jsonl, subdir.join("agent-ag-sub.jsonl"));
    assert_eq!(meta, subdir.join("agent-ag-sub.meta.json"));
    // ONE KNOWN SUFFIX, stripped by name. with_extension("") would agree on
    // every path that really ends .jsonl and diverge the moment one does not:
    // handed anything else, it would amputate whatever follows the last dot
    // and point the lookup at a directory nobody named. Strip what is known
    // to be there; leave every other name alone.
    let dotted = projects.join("chat.v2.jsonl");
    let (dj, _) = quarry::coord::subagent_records(&dotted, "ag-sub").unwrap();
    assert!(
        dj.starts_with(projects.join("chat.v2")),
        "a dotted chat id keeps its whole stem: {}",
        dj.display()
    );
    let suffixless = projects.join("chat.v2");
    let (sj, _) = quarry::coord::subagent_records(&suffixless, "ag-sub").unwrap();
    assert!(
        sj.starts_with(projects.join("chat.v2")),
        "a path that is not a .jsonl is not truncated at its last dot: {}",
        sj.display()
    );

    // THE SIDECAR reads the spawn's own record — the ALIAS the dispatcher
    // typed, never a resolved id. Measured across the 268 sidecars here,
    // `opus` resolved to claude-opus-5 in 128 arcs and claude-opus-4-8 in 43,
    // so no table maps one to the other and this can only ever be a fallback.
    std::fs::write(
        &meta,
        r#"{"agentType":"general-purpose","description":"d","toolUseId":"t","spawnDepth":1,"model":"opus"}"#,
    )
    .unwrap();
    assert_eq!(quarry::coord::sidecar_model(&meta).as_deref(), Some("opus"));
    // 88 of the 268 carry no model key at all. That is "the spawn named none",
    // never "runs the parent's model".
    let bare = subdir.join("agent-ag-bare.meta.json");
    std::fs::write(&bare, r#"{"agentType":"Explore","spawnDepth":2}"#).unwrap();
    assert_eq!(quarry::coord::sidecar_model(&bare), None, "no model key, no answer");
    assert_eq!(
        quarry::coord::sidecar_model(&subdir.join("agent-nobody.meta.json")),
        None,
        "a missing sidecar answers nothing rather than failing"
    );

    // THE PRECEDENCE INSIDE THE RECORD: the agent's own transcript leads,
    // because it names the RESOLVED model — one spelling across the arc — and
    // because it sees models the spawn call never named (an agent type's own
    // default, a nested spawn inheriting its parent AGENT). Measured on the 88
    // model-less sidecars: 9 ran on a model their parent chat was not running.
    let cp = chat_file.display().to_string();
    assert_eq!(
        quarry::coord::agent_model(Some(&cp), "ag-sub").as_deref(),
        Some("opus"),
        "before the first turn lands, the sidecar alias is the only answer"
    );
    std::fs::write(&jsonl, agent_tail).unwrap();
    assert_eq!(
        quarry::coord::agent_model(Some(&cp), "ag-sub").as_deref(),
        Some("claude-opus-5"),
        "once the agent's own turns are on disk, the resolved name wins"
    );
    // Every road out is a degradation to None, never an error: a hook must
    // not fail a shell over an undocumented layout.
    assert_eq!(quarry::coord::agent_model(None, "ag-sub"), None, "no transcript path, no directory to find");
    assert_eq!(quarry::coord::agent_model(Some(&cp), "ag-ghost"), None, "an agent with no record answers nothing");
    assert_eq!(
        quarry::coord::agent_model(Some(&jsonl.display().to_string()), "ag-sub"),
        None,
        "handed a subagent's own path instead of the chat's, the lookup misses and degrades"
    );

    // THE DEFECT, MEASURED END TO END THROUGH THE SPAWNED BINARY — the only
    // seat where QUARRY_ACTOR is honestly absent, which is the state a real
    // hook process runs in.
    let hook = |input: &str| {
        use std::io::Write as _;
        let mut c = std::process::Command::new(q)
            .current_dir(&s.root)
            .args(["hook", "session"])
            .env_remove("QUARRY_ACTOR")
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env("QUARRY_HOME", &s.root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        c.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
        let o = c.wait_with_output().unwrap();
        let text = String::from_utf8_lossy(&o.stdout).to_string();
        let v: serde_json::Value = serde_json::from_str(text.trim()).unwrap_or_else(|e| {
            panic!("hook output not JSON ({}): {} / stderr {}", e, text, String::from_utf8_lossy(&o.stderr))
        });
        v["hookSpecificOutput"]["updatedInput"]["command"].as_str().unwrap_or_default().to_string()
    };
    // The chat runs fable and its transcript says so; the subagent runs opus
    // and its own record says so. This is exactly the shape that mis-filed
    // every node of the it-xcvb arc.
    std::fs::write(
        &chat_file,
        concat!(r#"{"type":"assistant","message":{"model":"claude-fable-5"}}"#, "\n"),
    )
    .unwrap();
    let esc = cp.replace('\\', "\\\\");
    let sub = format!(
        r#"{{"session_id":"chat-chatty","agent_id":"ag-sub","transcript_path":"{}","tool_name":"Bash","tool_input":{{"command":"q open it-x"}}}}"#,
        esc
    );
    let parent = format!(
        r#"{{"session_id":"chat-chatty","transcript_path":"{}","tool_name":"Bash","tool_input":{{"command":"q open it-x"}}}}"#,
        esc
    );
    let out = hook(&sub);
    assert!(
        out.contains("QUARRY_ACTOR='claude-opus-5'"),
        "an UNSTAMPED subagent files under its own model, nothing remembered: {}",
        out
    );
    assert!(
        !out.contains("claude-fable-5"),
        "and never under the chat that spawned it: {}",
        out
    );
    // THE CHAT'S OWN SHELLS ARE UNTOUCHED. The lookup is keyed on the agent
    // id, which the harness gives only to a subagent, so the record can never
    // leak back up the wire to the seat that fired.
    assert!(
        hook(&parent).contains("QUARRY_ACTOR='claude-fable-5'"),
        "the dispatcher keeps its own live model beside its agent"
    );

    // --model SURVIVES AS AN OVERRIDE, not as the mechanism. The badge stamp
    // still outranks the harness record: the dispatcher's explicit word beats
    // a derived one, and it is the road that still works where the layout
    // does not exist at all.
    let area = ops::new_node(&s, NewArgs::bare("area", "attribution-record")).unwrap();
    let mut a = NewArgs::bare("item", "the unstamped arc");
    a.status = Some("ready".into());
    a.about = vec![area.front.id.clone()];
    a.acceptance = vec!["the arc lands".into()];
    let it = ops::new_node(&s, a).unwrap();
    let d = ops::dispatch(
        &s, &it.front.id, vec!["src/**".into()], false, false, None, Some("claude-haiku-5"),
        "disp", "claude-fable-5",
    )
    .unwrap();
    ops::join(&s, &d.token, Some("agent:ag-sub".into())).unwrap();
    let stamped = hook(&sub);
    assert!(
        stamped.contains("QUARRY_ACTOR='claude-haiku-5'"),
        "the stamp overrides the harness record: {}",
        stamped
    );
    // …and when the badge is freed, the structural road answers again — where
    // before it-6ekf the same shell fell all the way back to the chat's model.
    quarry::coord::clear_dispatch(&s, &it.front.id);
    let freed = hook(&sub);
    assert!(
        freed.contains("QUARRY_ACTOR='claude-opus-5'"),
        "a cleared badge falls to the agent's own model, not the dispatcher's: {}",
        freed
    );
    // A SUBAGENT THE RECORD CANNOT SPEAK FOR still inherits, which is the
    // right answer for a spawn that genuinely inherits.
    let unknown = format!(
        r#"{{"session_id":"chat-chatty","agent_id":"ag-ghost","transcript_path":"{}","tool_name":"Bash","tool_input":{{"command":"q open it-x"}}}}"#,
        esc
    );
    assert!(
        hook(&unknown).contains("QUARRY_ACTOR='claude-fable-5'"),
        "no record, no invention: the chat road answers"
    );

    // THE UNDOCUMENTED DEPENDENCY IS STATED WHERE IT IS READ. The guide's
    // ENVIRONMENT section is the one place the injected actor is explained to
    // an agent, so the second harness-internal dependency has to be readable
    // there rather than inferable from behaviour.
    let g = quarry::teach::GUIDE;
    assert!(g.contains("subagents/agent-<id>.jsonl"), "the guide names the layout it reads");
    assert!(
        g.contains("UNDOCUMENTED AND HARNESS-INTERNAL"),
        "the guide states the exposure plainly"
    );
    assert!(g.contains("OVERRIDE"), "the guide says --model is now an override");
}

#[test]
fn a_bare_fire_names_no_model_and_the_join_names_what_it_read() {
    // it-xwpw. it-6ekf made a subagent's model resolve structurally from the
    // harness's own record keyed by the injected agent id (cl-jp4q), so an
    // UNSTAMPED dispatch stopped filing under the dispatching chat's model.
    // Two statements written for the old world went on saying it does, in the
    // present tense, at the two seats an operator and an agent actually read:
    // the fire's unstamped attribution line and the join's. Both were false
    // in every clause, and the join's was worse than wrong — it told the ONE
    // mind that could check the answer a reason that had stopped being true.
    //
    // The two seats are not symmetrical, and the cure differs by seat. The
    // FIRE genuinely cannot know: the agent does not exist yet, so there is
    // no agent id and no record, and the honest line names the ROAD rather
    // than a model. The JOIN runs INSIDE the agent with its id in hand, so it
    // can read the record itself and NAME what it resolved.
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");

    // ── THE LOCATOR, the piece the join needed and did not have.
    // `agent_model` takes the CHAT transcript path, which a hook is handed in
    // its payload and a verb is not: nothing injects it. What a verb does
    // have is the chat id (QUARRY_CHAT) and the harness's naming convention —
    // each chat's transcript sits one project directory down, named for the
    // chat — so the file is found by walking the project directories for
    // `<chat>.jsonl`.
    let harness = s.root.join("harness");
    let projects = harness.join("projects");
    std::fs::create_dir_all(projects.join("proj-decoy")).unwrap();
    let projb = projects.join("proj-b");
    std::fs::create_dir_all(&projb).unwrap();
    let chat_file = projb.join("chat-chatty.jsonl");
    std::fs::write(&chat_file, "").unwrap();
    assert_eq!(
        quarry::coord::find_chat_transcript_in(&projects, "chat-chatty").as_deref(),
        Some(chat_file.as_path()),
        "the walk passes a project directory that does not hold the chat and finds the one that does"
    );
    assert_eq!(
        quarry::coord::find_chat_transcript_in(&projects, "chat-stranger"),
        None,
        "an unknown chat answers nothing rather than guessing at a neighbour"
    );
    // The chat id arrives from the ENVIRONMENT, and joining it as a path
    // component is the one way this walk could reach outside the tree it was
    // pointed at — measured on a file that a traversal really would find:
    // <project>/../escape.jsonl resolves back up into the projects root, and
    // without the guard the walk returns it. Refused at the top, before any
    // directory is read.
    std::fs::write(projects.join("escape.jsonl"), "").unwrap();
    assert_eq!(
        quarry::coord::find_chat_transcript_in(&projects, "../escape"),
        None,
        "a separator-bearing id never becomes a path component"
    );
    assert_eq!(
        quarry::coord::find_chat_transcript_in(&projects, "   "),
        None,
        "an empty id names nothing"
    );
    assert_eq!(
        quarry::coord::find_chat_transcript_in(&s.root.join("no-such-root"), "chat-chatty"),
        None,
        "a missing root degrades to None — a verb must not fail over an undocumented layout"
    );

    // The harness's record of three agents. ag-sub opens with its SIDECAR
    // alone — the pre-first-turn window cl-jp4q names, where the spawn alias
    // is the only answer — and its own transcript lands later, mid-test.
    let subdir = projb.join("chat-chatty").join("subagents");
    std::fs::create_dir_all(&subdir).unwrap();
    let agent_turns = concat!(
        r#"{"type":"assistant","message":{"model":"claude-opus-5"},"isSidechain":true}"#,
        "\n"
    );
    std::fs::write(
        subdir.join("agent-ag-sub.meta.json"),
        r#"{"agentType":"general-purpose","spawnDepth":1,"model":"opus"}"#,
    )
    .unwrap();
    std::fs::write(subdir.join("agent-ag-two.jsonl"), agent_turns).unwrap();
    std::fs::write(subdir.join("agent-ag-three.jsonl"), agent_turns).unwrap();

    // ── THE FIRE HAS NOTHING TO NAME when the dispatcher stamps nothing, and
    // the outcome type now says so: the field that used to fall back to the
    // dispatching chat's actor is an Option, so the false statement is no
    // longer representable.
    let area = ops::new_node(&s, NewArgs::bare("area", "attribution-statements")).unwrap();
    let ready = |title: &str| {
        let mut a = NewArgs::bare("item", title);
        a.status = Some("ready".into());
        a.about = vec![area.front.id.clone()];
        a.acceptance = vec!["the arc lands".into()];
        ops::new_node(&s, a).unwrap()
    };
    let pure = ready("the pure fire");
    let pd = ops::dispatch(
        &s, &pure.front.id, vec!["src/p/**".into()], false, false, None, None, "disp",
        "claude-fable-5",
    )
    .unwrap();
    assert_eq!(pd.arc_actor, None, "an unstamped fire has no model to name");
    let ps = ops::dispatch(
        &s,
        &ready("the stamped fire").front.id,
        vec!["src/q/**".into()],
        false,
        false,
        None,
        Some("claude-haiku-5"),
        "disp",
        "claude-fable-5",
    )
    .unwrap();
    assert_eq!(
        ps.arc_actor.as_deref(),
        Some("claude-haiku-5"),
        "a stamped fire is certain of it"
    );
    // With no record reachable in-process (temp_store scrubs QUARRY_CHAT), the
    // join degrades to the injected actor and SAYS it is a fallback — it never
    // claims a road it did not take.
    let pj = ops::join(&s, &pd.token, Some("agent:ag-nowhere".into())).unwrap();
    assert_eq!(pj.actor_source, ops::ArcActorSource::Injected);

    // ── END TO END THROUGH THE SPAWNED BINARY, the only seat where the two
    // printed statements exist at all. The child's environment carries the
    // dispatching chat's model as QUARRY_ACTOR — which is exactly what the
    // old lines named — and a fabricated harness home under CLAUDE_CONFIG_DIR.
    let sroot = s.root.display().to_string();
    let hroot = harness.display().to_string();
    let empty = s.root.join("empty-harness");
    std::fs::create_dir_all(&empty).unwrap();
    let eroot = empty.display().to_string();
    let qrun = |envs: &[(&str, &str)], args: &[&str]| -> String {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
            .env_remove("QUARRY_STORE")
            .env_remove("CLAUDE_CONFIG_DIR")
            .env("QUARRY_HOME", &s.root)
            .env("QUARRY_ACTOR", "claude-fable-5")
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        let out = c.output().unwrap();
        assert!(
            out.status.success(),
            "q {:?} failed: {}{}",
            args,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    let fire = |item: &str, model: Option<&str>| -> (String, String) {
        let mut args = vec!["dispatch", item, "--files", "src/**", "--solo"];
        if let Some(m) = model {
            args.push("--model");
            args.push(m);
        }
        let out = qrun(&[("QUARRY_SESSION", "disp")], &args);
        let line = out
            .lines()
            .find(|l| l.contains("attribution:"))
            .unwrap_or_else(|| panic!("the fire states attribution every time: {}", out))
            .to_string();
        let marker = "run: q join ";
        let pos = out.find(marker).expect("spawn line present");
        let token = out[pos + marker.len()..].split_whitespace().next().unwrap().to_string();
        (line, token)
    };

    // THE FIRE'S UNSTAMPED LINE. Every clause of the old one was false: it
    // named the dispatching chat's model as what the arc would file under,
    // called it inherited, and warned that on any other model the agent's
    // mints would file under a model that did not write them.
    let bare_item = ready("the unstamped arc");
    let (bare_line, bare_token) = fire(&bare_item.front.id, None);
    assert!(
        !bare_line.contains("claude-fable-5"),
        "the fire never names the dispatching chat's model as the arc's: {}",
        bare_line
    );
    assert!(
        !bare_line.contains("inherit"),
        "and never calls the unstamped answer inherited: {}",
        bare_line
    );
    assert!(
        bare_line.contains("AGENT's own model") && bare_line.contains("first fire"),
        "it names the road instead — the record read at the agent's own first fire: {}",
        bare_line
    );
    assert!(
        bare_line.contains("--model <model>"),
        "the stamp stays in hand as an override at the one station that can add it: {}",
        bare_line
    );
    // THE FIRE'S STAMPED LINE still names the model, and now says what the
    // stamp DOES: it outranks the harness's own record of the agent.
    let stamped_item = ready("the stamped arc");
    let (stamped_line, stamped_token) = fire(&stamped_item.front.id, Some("claude-haiku-5"));
    assert!(
        stamped_line.contains("claude-haiku-5") && stamped_line.contains("OVERRIDES"),
        "a stamped fire names the model and what the stamp does to the record: {}",
        stamped_line
    );

    let join = |token: &str, agent: &str, cfg: &str| -> String {
        let out = qrun(
            &[
                ("QUARRY_CHAT", "chat-chatty"),
                ("QUARRY_AGENT", agent),
                ("CLAUDE_CONFIG_DIR", cfg),
            ],
            &["join", token, "--store", &sroot],
        );
        out.lines()
            .find(|l| l.contains("file under"))
            .unwrap_or_else(|| panic!("the join states attribution every time: {}", out))
            .to_string()
    };

    // ── THE JOIN NAMES WHAT IT READ. The old line said the acts file under
    // the dispatching chat's model, "because the harness tells a subagent's
    // hooks nothing about its own model" — both halves false after it-6ekf:
    // the harness does tell, one directory over, and the acts do not file
    // under the chat.
    let bare_join = join(&bare_token, "ag-sub", &hroot);
    assert!(
        !bare_join.contains("claude-fable-5"),
        "the join never hands the agent the dispatching chat's model: {}",
        bare_join
    );
    assert!(
        !bare_join.contains("the harness tells a subagent's hooks nothing"),
        "nor the reason that stopped being true: {}",
        bare_join
    );
    // THE PRE-FIRST-TURN WINDOW, cl-jp4q's one named grain: before this
    // agent's own turns reach disk the sidecar's coarse spawn alias is the
    // only answer, and a family-correct alias beats the dispatcher's model,
    // which is wrong outright. safe_actor still guards it.
    assert!(
        bare_join.contains("claude:opus"),
        "the sidecar answers the window the transcript cannot: {}",
        bare_join
    );
    assert!(
        bare_join.contains("harness's own record of THIS agent"),
        "and the line names the road it took: {}",
        bare_join
    );
    // …AND THE GRAIN CLOSES ONE SHELL LATER. The join is composed after the
    // hook that injected this shell's actor, so a turn that had not reached
    // disk at the fire has by now: the same identity's re-join — an
    // idempotent read — reads the RESOLVED model where the alias stood.
    std::fs::write(subdir.join("agent-ag-sub.jsonl"), agent_turns).unwrap();
    let rejoin = join(&bare_token, "ag-sub", &hroot);
    assert!(
        rejoin.contains("claude-opus-5") && !rejoin.contains("claude:opus"),
        "once the agent's own turns are on disk the resolved name answers: {}",
        rejoin
    );

    // THE STAMP STILL OUTRANKS THE RECORD, which is what makes --model an
    // override rather than dead weight: ag-two's own transcript says
    // claude-opus-5 and the badge says claude-haiku-5, and the arc files
    // under the badge — stated as a stamp, so a mismatch is reportable.
    let stamped_join = join(&stamped_token, "ag-two", &hroot);
    assert!(
        stamped_join.contains("claude-haiku-5") && stamped_join.contains("stamped into the badge"),
        "the dispatcher's explicit word wins, and the line says so: {}",
        stamped_join
    );

    // ── THE ARC'S FIRST ACT FILES UNDER THE MODEL THAT MADE IT, with no
    // stamp anywhere. The join event used to take whatever actor the shell
    // was injected with; it now takes what the record answered here, so an
    // unstamped arc's first badged act is right even when the hook that
    // injected the shell could not resolve it.
    let third = ready("the recorded arc");
    let (_, third_token) = fire(&third.front.id, None);
    let third_join = join(&third_token, "ag-three", &hroot);
    assert!(third_join.contains("claude-opus-5"), "the record answers: {}", third_join);
    let log = s.read_log().unwrap();
    let join_ev = log
        .iter()
        .rev()
        .find(|e| {
            e.get("op").and_then(|v| v.as_str()) == Some("join")
                && e.get("node").and_then(|v| v.as_str()) == Some(third.front.id.as_str())
        })
        .expect("the bind logs a join");
    assert_eq!(
        join_ev.get("actor").and_then(|v| v.as_str()),
        Some("claude-opus-5"),
        "unstamped, and the first act still files under the agent's own model: {}",
        join_ev
    );

    // ── EVERY MISS DEGRADES, NEVER ERRORS, AND NEVER INVENTS. Pointed at a
    // harness home holding no record of this agent, the join keeps the actor
    // its shell was injected with — and says that is what it is, rather than
    // dressing a fallback as a reading.
    let ghost = ready("the unrecorded arc");
    let (_, ghost_token) = fire(&ghost.front.id, None);
    let ghost_join = join(&ghost_token, "ag-ghost", &eroot);
    assert!(
        ghost_join.contains("claude-fable-5") && ghost_join.contains("could not answer"),
        "no record, no invention — the fallback is named as one: {}",
        ghost_join
    );
    assert!(
        !ghost_join.contains("the harness tells a subagent's hooks nothing"),
        "and still never asserts the reason it-6ekf retired: {}",
        ghost_join
    );
}
