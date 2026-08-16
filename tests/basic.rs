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
        "the halo is bounded", None,
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
        "halo is 4-11 cells", None,
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
        "halo bounded", None,
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
    ops::new_node(&s, i1).unwrap();
    let mut i2 = NewArgs::bare("item", "gait clip");
    i2.about = vec![bod.front.id.clone()];
    i2.status = Some("ready".into());
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
    let err = ops::claim(&s, "vibes", None, vec![area.front.id.clone()], None, None, Some("assistant".into()), None)
        .unwrap_err();
    assert!(err.to_string().contains("C2"), "got: {}", err);
    // method alone grounds it
    ops::claim(&s, "measured thing", None, vec![area.front.id.clone()], None,
        Some("log inspection".into()), None, None).unwrap();
    // a file: source grounds it (matrix widened), blob-stamped
    let c = ops::claim(&s, "read off the code", None, vec![area.front.id.clone()],
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
    ops::dispatch(&s, &f.front.id, vec!["src/geo/**".into()], false, false, None, "geo", "t").unwrap();
    let mut th = NewArgs::bare("thread", "which datum wins?");
    th.status = Some("queued".into());
    th.about = vec![aid.clone()];
    th.provenance = Some("user".into());
    let th = ops::new_node(&s, th).unwrap();
    // homework residue: a claim citing the area at v1, then the area bumps
    let c = ops::claim(
        &s,
        "strata are layered", None,
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

    // a purview with nothing owed says so once, and only about ready
    let quiet = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let out = quarry::render::dispatch_wake(&s, &all, &[quiet.front.id.as_str()]);
    assert_eq!(out.len(), 1, "quiet purview, one line: {:?}", out);
    assert!(out[0].starts_with("ready to dispatch: none in purview"));
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
        "materials resolve per-voxel through layered override stacks", None,
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
    let out = ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, "geo", "t").unwrap();
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
    assert_eq!(d.globs, vec!["src/geo/**".to_string()]);
    assert_eq!(d.acceptance.len(), 1, "the contract rides the state file");
    assert_eq!(d.token.as_deref(), Some(out.token.as_str()), "the join token rides the held entry");
    assert!(d.joined.is_none(), "unconsumed until an agent joins");
    assert!(quarry::coord::briefed_this_session(&s, &it.front.id, "geo"), "dispatch briefs (C8)");
    // re-dispatch keeps the lease but is a NEW hand-off: fresh token
    let again = ops::dispatch(&s, &it.front.id, vec![], false, false, None, "geo", "t").unwrap();
    assert!(again.reused_lease);
    assert_ne!(again.token, out.token, "a re-dispatch mints a fresh token");
    // FLIP (dc-qyr5, was: "a second dispatch refuses from the chat already
    // holding one"): MULTI-HELD — the same chat dispatches a second item
    // freely; fire-them-all-off from one chat is literal
    let other = ops::new_node(&s, NewArgs::bare("item", "other work")).unwrap();
    let par = ops::dispatch(&s, &other.front.id, vec!["docs/**".into()], false, false, None, "geo", "t").unwrap();
    assert!(!par.reused_lease);
    let held = quarry::coord::held_dispatches(&s, "session:geo");
    assert_eq!(held.len(), 2, "one chat, two live dispatches (dc-qyr5)");
    // …what refuses now is PER-ITEM ownership: another chat dispatching a
    // live-dispatched item is turned away naming the holding chat and
    // session, with the loud road advertised
    let err = ops::dispatch(&s, &other.front.id, vec![], false, false, None, "geo2", "t").unwrap_err();
    assert!(err.to_string().contains("already dispatched"), "got: {}", err);
    assert!(err.to_string().contains("session:geo"), "holding chat named: {}", err);
    assert!(err.to_string().contains("session geo"), "holding session named: {}", err);
    assert!(err.to_string().contains("--steal"), "the loud road advertised: {}", err);
    // an unidentified process amid TWO live badges inherits neither — the
    // machine-global fallback is gone; env transport is the agent's stamp
    let c = ops::claim(
        &s,
        "`geo-pass`: emits layered strata", None,
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
    let err = ops::dispatch(&s, &done.front.id, vec!["src/**".into()], false, false, None, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("[done]"), "got: {}", err);
    // a foreign SOLO lease (no live dispatch) still blocks dispatch with the
    // holder named — the dispatch steal takes dispatches, not solo leases
    let it = ops::new_node(&s, NewArgs::bare("item", "contested work")).unwrap();
    quarry::coord::reserve(&s, &it, "bodies", "t", vec!["src/x/**".into()], false, false, None).unwrap();
    let err = ops::dispatch(&s, &it.front.id, vec!["src/x/**".into()], false, false, None, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("bodies"), "got: {}", err);
    // no globs anywhere refuses with the teaching line
    let bare = ops::new_node(&s, NewArgs::bare("item", "bare work")).unwrap();
    let err = ops::dispatch(&s, &bare.front.id, vec![], false, false, None, "geo", "t").unwrap_err();
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
    let out = ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, "geo", "t").unwrap();
    // an agent joined the original dispatch and works under it
    ops::join(&s, &out.token, Some("agent:ag-old".into())).unwrap();
    assert_eq!(quarry::coord::badge_for(&s, Some("ag-old"), None, None).as_deref(), Some(it.front.id.as_str()));
    // --steal without --reason refuses: the reason is required (dc-qyr5),
    // and nothing moved
    let err = ops::dispatch(&s, &it.front.id, vec![], false, true, None, "ops", "t").unwrap_err();
    assert!(err.to_string().contains("--reason"), "got: {}", err);
    assert_eq!(quarry::coord::dispatch_for_item(&s, &it.front.id).unwrap().holder, "session:geo");
    // steal with the reason takes the dispatch WHOLE
    let st = ops::dispatch(&s, &it.front.id, vec![], false, true, Some("holder went dark"), "ops", "t").unwrap();
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
    assert_eq!(leases[0].globs, vec!["src/geo/**".to_string()], "globs survive the take-over");
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
    let out = ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, "geo", "t").unwrap();
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
    let again = ops::dispatch(&s, &it.front.id, vec![], false, false, None, "geo", "t").unwrap();
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
    let out = ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, "geo", "t").unwrap();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |envs: &[(&str, &str)], args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .env_remove("QUARRY_CHAT")
            .env_remove("QUARRY_AGENT")
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
    let it2 = ops::new_node(&s, it2).unwrap();
    let out4 = ops::dispatch(&s, &it2.front.id, vec!["docs/**".into()], false, false, None, "geo2", "t").unwrap();
    let out5 = run(&[("QUARRY_CHAT", "chat-f")], &["join", &out4.token]);
    assert!(out5.status.success(), "{}", String::from_utf8_lossy(&out5.stderr));
    assert_eq!(
        quarry::coord::load_dispatches(&s).acting.get("chat:chat-f").map(|b| b.as_str()),
        Some(it2.front.id.as_str())
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
        "`body-graph`: bodies keep identity", None,
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
        "`geo-pass`: emits layered strata", None,
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
        "`body-graph`: bodies keep identity across regen", None,
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
        "`halo-check`: bounded halo", None,
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
        "`chunk-cache`: regen hits a warm cache", None,
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
        "`halo-check`: halo bounded, measured late", None,
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
        "halo bounded", None,
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
            .args(args);
        for (k, v) in envs {
            c.env(k, v);
        }
        c.output().unwrap()
    };
    // env badge: wrap and session resume/retire all refuse, teaching
    for args in [&["wrap"][..], &["session", "resume"][..], &["session", "retire", "x"][..]] {
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
    ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, false, None, "design", "t").unwrap();
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
