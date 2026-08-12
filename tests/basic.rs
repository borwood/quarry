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

    quarry::coord::save_session(&s, "geo", vec![geo.front.id.clone()], None, false).unwrap();
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
fn session_injection_binds_and_rewrites() {
    let s = temp_store();
    quarry::coord::write_adopt_request(&s, "geo").unwrap();
    let input = r#"{"session_id":"chat-abc","tool_name":"Bash","tool_input":{"command":"q wrap","description":"lint"}}"#;
    let out = quarry::teach::session_hook_output(&s, input).expect("injects after adopt");
    let cmd = out["hookSpecificOutput"]["updatedInput"]["command"].as_str().unwrap();
    assert_eq!(cmd, "export QUARRY_SESSION='geo'; q wrap");
    assert_eq!(out["hookSpecificOutput"]["updatedInput"]["description"].as_str().unwrap(), "lint");
    // binding persisted: no pending request, still injects; PowerShell prefix
    let input2 = r#"{"session_id":"chat-abc","tool_name":"PowerShell","tool_input":{"command":"q view"}}"#;
    let out2 = quarry::teach::session_hook_output(&s, input2).expect("bound");
    let cmd2 = out2["hookSpecificOutput"]["updatedInput"]["command"].as_str().unwrap();
    assert_eq!(cmd2, "$env:QUARRY_SESSION='geo'; q view");
    // unbound chat: no-op
    let input3 = r#"{"session_id":"chat-other","tool_name":"Bash","tool_input":{"command":"ls"}}"#;
    assert!(quarry::teach::session_hook_output(&s, input3).is_none());
    // non-shell tools: no-op even when bound
    let input4 = r#"{"session_id":"chat-abc","tool_name":"Write","tool_input":{"file_path":"x"}}"#;
    assert!(quarry::teach::session_hook_output(&s, input4).is_none());
}

#[test]
fn purview_overlap_detection() {
    let s = temp_store();
    let a = ops::new_node(&s, NewArgs::bare("area", "worldgen")).unwrap();
    let b = ops::new_node(&s, NewArgs::bare("area", "materials sdk")).unwrap();
    let c = ops::new_node(&s, NewArgs::bare("area", "bodies")).unwrap();
    quarry::coord::save_session(&s, "geo", vec![a.front.id.clone(), b.front.id.clone()], None, false).unwrap();
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
    quarry::coord::save_session(&s, "geo", vec![area.front.id.clone()], None, false).unwrap();
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
    quarry::coord::save_session(&s, "audit", vec![a.front.id.clone()], None, true).unwrap();
    quarry::coord::save_session(&s, "geo2", vec![a.front.id.clone()], None, false).unwrap();
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
    quarry::coord::save_session(&s, "audit", vec![a.front.id.clone()], None, true).unwrap();
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
fn spine_presence_check() {
    let s = temp_store();
    std::process::Command::new("git").arg("init").arg("-q").current_dir(&s.root).status().unwrap();
    std::fs::write(s.root.join("resolve.rs"), "fn spine() {}\n").unwrap();
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
    assert!(queries::files_cited(&all, &["resolve.rs".to_string()]), "the spine claim cites it");
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
    assert!(matches!(lease_check(&leases, Some("bodies"), None, "src/geo/pass.rs"), LeaseCheck::Deny(_)), "foreign exclusive zone denies");
    assert!(matches!(lease_check(&leases, None, None, "src/geo/pass.rs"), LeaseCheck::Deny(_)), "unbound writes into leased zones deny");
    assert!(matches!(lease_check(&leases, Some("geo"), None, "src/geo/pass.rs"), LeaseCheck::Allow));
    assert!(matches!(lease_check(&leases, Some("geo"), None, "src/other.rs"), LeaseCheck::Warn(_)), "scope creep warns the holder");
    assert!(matches!(lease_check(&leases, Some("geo"), Some(it.front.id.as_str()), "src/other.rs"), LeaseCheck::Deny(_)), "dispatch outside write-set denies");
    assert!(matches!(lease_check(&leases, Some("bodies"), None, "graph/sessions.json"), LeaseCheck::Allow));
    assert!(matches!(lease_check(&[], None, None, "src/x.rs"), LeaseCheck::Allow));
    // Out-of-repo paths are never judged: no scope creep, no deny — for
    // holders, foreigners, the unbound, and even a badge.
    assert!(matches!(
        lease_check(&leases, Some("geo"), None, "c:/users/x/appdata/local/temp/scratchpad/notes.md"),
        LeaseCheck::Allow
    ));
    assert!(matches!(lease_check(&leases, Some("bodies"), None, "c:/tmp/elsewhere/src/geo/pass.rs"), LeaseCheck::Allow));
    assert!(matches!(lease_check(&leases, None, None, "/tmp/notes.md"), LeaseCheck::Allow));
    assert!(matches!(
        lease_check(&leases, Some("geo"), Some(it.front.id.as_str()), "/tmp/outside.rs"),
        LeaseCheck::Allow
    ));
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
        globs: vec!["src/**".into()],
        acceptance: vec!["it lands".into()],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    assert_eq!(quarry::coord::load_dispatch(&s).unwrap().item, "it-test");
    // badge resolution falls back to the state file (env unset in tests)
    assert_eq!(quarry::coord::current_dispatch_badge(&s).as_deref(), Some("it-test"));
    // clearing a different item leaves the badge; the matching item clears it
    quarry::coord::clear_dispatch(&s, "it-other");
    assert!(quarry::coord::load_dispatch(&s).is_some());
    quarry::coord::clear_dispatch(&s, "it-test");
    assert!(quarry::coord::load_dispatch(&s).is_none());
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
fn log_events_stamp_the_badge_from_state() {
    let s = temp_store();
    let d = quarry::coord::DispatchState {
        item: "it-bdg".into(),
        item_title: "badged".into(),
        session: "geo".into(),
        globs: vec![],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": "cl-zzzz", "v": 1, "op": "create", "type": "claim", "actor": "t"
    }))
    .unwrap();
    let log = s.read_log().unwrap();
    let ev = log.last().unwrap();
    assert_eq!(ev.get("dispatch").and_then(|v| v.as_str()), Some("it-bdg"));
    // badge cleared: later events are unstamped
    quarry::coord::clear_dispatch(&s, "it-bdg");
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": "cl-yyyy", "v": 1, "op": "create", "actor": "t"
    }))
    .unwrap();
    let log = s.read_log().unwrap();
    assert!(log.last().unwrap().get("dispatch").is_none());
}

#[test]
fn observe_write_contract_echo_and_drift() {
    use quarry::teach::observe_write;
    let s = temp_store();
    let d = quarry::coord::DispatchState {
        item: "it-bdg".into(),
        item_title: "guard growth".into(),
        session: "geo".into(),
        globs: vec!["src/**".into()],
        acceptance: vec!["a".into(), "b".into()],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: Store::now(),
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
    let mut d2 = quarry::coord::load_dispatch(&s).unwrap();
    d2.checked = "2000-01-01T00:00:00Z".into();
    quarry::coord::save_dispatch(&s, &d2).unwrap();
    let out3 = observe_write(&s, &[], Some("geo"), Some("it-bdg"), "src/other.rs");
    assert!(out3.iter().any(|l| l.contains("dispatch drift")), "got {:?}", out3);
    // cursor advanced by the delivery: stale throttle again, no re-delivery
    let mut d3 = quarry::coord::load_dispatch(&s).unwrap();
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
    let out = ops::dispatch(&s, &it.front.id, vec!["src/geo/**".into()], false, "geo", "t").unwrap();
    assert!(out.payload.contains(&format!("QUARRY_DISPATCH={}", it.front.id)), "payload carries the badge");
    assert!(out.payload.contains("DISPATCH BRIEF"), "payload carries the brief whole");
    assert!(!out.reused_lease);
    let leases = quarry::coord::load_leases(&s);
    assert_eq!(leases.len(), 1);
    assert_eq!(leases[0].item, it.front.id);
    assert_eq!(leases[0].session, "geo");
    let all = s.load_all().unwrap();
    assert_eq!(s.find(&all, &it.front.id).unwrap().front.status, "in-flight");
    let d = quarry::coord::load_dispatch(&s).unwrap();
    assert_eq!(d.item, it.front.id);
    assert_eq!(d.globs, vec!["src/geo/**".to_string()]);
    assert_eq!(d.acceptance.len(), 1, "the contract rides the state file");
    assert!(quarry::coord::briefed_this_session(&s, &it.front.id, "geo"), "dispatch briefs (C8)");
    // re-dispatch keeps the lease
    let again = ops::dispatch(&s, &it.front.id, vec![], false, "geo", "t").unwrap();
    assert!(again.reused_lease);
    // a second concurrent dispatch refuses — one badge at a time
    let other = ops::new_node(&s, NewArgs::bare("item", "other work")).unwrap();
    let err = ops::dispatch(&s, &other.front.id, vec!["docs/**".into()], false, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("already active"), "got: {}", err);
    // acts under the badge are stamped (state-file transport)
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
    assert_eq!(ev.get("dispatch").and_then(|v| v.as_str()), Some(it.front.id.as_str()));
    // observed writes accrue under the item key; harvest renders the seat
    quarry::coord::accrue_touch(&s, &format!("item:{}", it.front.id), "src/geo/pass.rs");
    let h = quarry::render::harvest(&s, &it.front.id).unwrap();
    assert!(h.contains("src/geo/pass.rs"), "observed file listed");
    assert!(h.contains("stop signal"), "harness done never transitions");
    assert!(h.contains(&format!("q query dispatch {}", it.front.id)), "trace advertised");
    assert!(h.contains("--supports"), "one-command report registration");
    assert!(h.contains("1 claim(s) minted"), "badge-stamped acts counted: {}", h);
    assert!(h.contains("the pass lands"), "RETURN spec re-listed for judging");
    let t = quarry::render::dispatch_trace(&s, &it.front.id).unwrap();
    assert!(t.contains("src/geo/pass.rs"));
    assert!(t.contains(&c.front.id), "stamped claim in the trace");
    // wrap sees the unharvested dispatch; a harvest event clears it
    let all = s.load_all().unwrap();
    let un = queries::unharvested_dispatches(&all, &log);
    assert!(un.iter().any(|n| n.front.id == it.front.id));
    s.log_event(serde_json::json!({
        "ts": Store::now(), "node": it.front.id, "v": 1, "op": "harvest", "actor": "t"
    }))
    .unwrap();
    let log2 = s.read_log().unwrap();
    assert!(queries::unharvested_dispatches(&all, &log2).is_empty());
}

#[test]
fn dispatch_refuses_settled_and_foreign_lease() {
    let s = temp_store();
    let mut done = NewArgs::bare("item", "landed work");
    done.status = Some("done".into());
    let done = ops::new_node(&s, done).unwrap();
    let err = ops::dispatch(&s, &done.front.id, vec!["src/**".into()], false, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("[done]"), "got: {}", err);
    // a foreign holder blocks dispatch with the holder named
    let it = ops::new_node(&s, NewArgs::bare("item", "contested work")).unwrap();
    quarry::coord::reserve(&s, &it, "bodies", "t", vec!["src/x/**".into()], false, false, None).unwrap();
    let err = ops::dispatch(&s, &it.front.id, vec!["src/x/**".into()], false, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("bodies"), "got: {}", err);
    // no globs anywhere refuses with the teaching line
    let bare = ops::new_node(&s, NewArgs::bare("item", "bare work")).unwrap();
    let err = ops::dispatch(&s, &bare.front.id, vec![], false, "geo", "t").unwrap_err();
    assert!(err.to_string().contains("--files"), "got: {}", err);
}

#[test]
fn brief_carries_dispatch_citizenship_sections() {
    let s = temp_store();
    let area = ops::new_node(&s, NewArgs::bare("area", "hydrology")).unwrap();
    let mut spine = NewArgs::bare("claim", "`body-graph`: bodies keep identity");
    spine.about = vec![area.front.id.clone()];
    spine.body = "water bodies keep identity across chunk regeneration by graph persistence".into();
    spine.provenance = Some("user".into());
    ops::new_node(&s, spine).unwrap();
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
        "spine shelf renders bodies, not titles: {}",
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
    let mut d1 = NewArgs::bare("decision", "spine landmarks");
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
    // a refuted spine enumerates its builders — and flips no builder status
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
    // a spine in the shared area lands one intent
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
    // an emergent spine in the item's area that no intent named
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
    assert!(un.contains(&"halo-check"), "the foreign-area spine is unintended there: {:?}", un);
    assert!(!un.contains(&"body-graph"), "intended and landed is quiet: {:?}", un);
    // area scoping excludes the foreign claim
    let scoped = queries::intent_delta(&all, Some(hydro.front.id.as_str()));
    assert!(scoped.unintended.iter().all(|(_, c)| c.front.id != foreign.front.id));
    assert_eq!(scoped.unlanded.len(), 1);
    // a refuted spine no longer lands its name
    let ev = ops::new_node(&s, NewArgs::bare("doc", "remeasurement")).unwrap();
    ops::refute(&s, &bg.front.id, &ev.front.id, None).unwrap();
    let all = s.load_all().unwrap();
    let d = queries::intent_delta(&all, None);
    assert!(
        d.unlanded.iter().any(|(n, _)| n == "body-graph"),
        "a refuted spine no longer lands the intent"
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
    // body_html carries the anchor to the node (JSON-escaped in the data blob)
    assert!(
        html.contains(&format!("<a href=\\\"#/n/{id}\\\">{id}<\\/a>", id = d.front.id)),
        "view unpack hyperlinks to the node anchor"
    );
    assert!(
        html.contains("[decision in-force: <code>bodies persist<\\/code>]"),
        "view unpack expands to id [type status: title]"
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
fn boundary_verbs_refuse_under_active_badge() {
    let s = temp_store();
    // no badge: no refusal
    assert!(quarry::coord::boundary_refusal(&s, "q wrap").is_none());
    // machine-local dispatch state alone suffices (env unset in tests)
    let d = quarry::coord::DispatchState {
        item: "it-b0nd".into(),
        item_title: "badged work".into(),
        session: "disp".into(),
        globs: vec!["src/**".into()],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    let msg = quarry::coord::boundary_refusal(&s, "q wrap").expect("state file refuses");
    assert!(msg.contains("boundary-verb capture"), "names the incident class: {}", msg);
    assert!(msg.contains("q harvest it-b0nd"), "teaches the exit: {}", msg);
    assert!(msg.contains("q wrap"), "names the refused verb: {}", msg);
    // the same guard serves every boundary verb
    let msg = quarry::coord::boundary_refusal(&s, "q session retire").unwrap();
    assert!(msg.contains("q session retire") && msg.contains("q harvest it-b0nd"));
    // harvest/land clears the badge and the boundary reopens
    quarry::coord::clear_dispatch(&s, "it-b0nd");
    assert!(quarry::coord::boundary_refusal(&s, "q wrap").is_none());
}

#[test]
fn wrap_refuses_badged_then_regenerates_view_when_clear() {
    let s = temp_store();
    let q = env!("CARGO_BIN_EXE_q");
    let run = |badge: Option<&str>, args: &[&str]| {
        let mut c = std::process::Command::new(q);
        c.current_dir(&s.root)
            .env_remove("QUARRY_SESSION")
            .env_remove("QUARRY_DISPATCH")
            .args(args);
        if let Some(b) = badge {
            c.env("QUARRY_DISPATCH", b);
        }
        c.output().unwrap()
    };
    // env badge: wrap and session resume/retire all refuse, teaching
    for args in [&["wrap"][..], &["session", "resume"][..], &["session", "retire", "x"][..]] {
        let out = run(Some("it-t3st"), args);
        assert!(!out.status.success(), "{:?} must refuse under a badge", args);
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("boundary-verb capture"), "{:?} names the incident class: {}", args, err);
        assert!(err.contains("q harvest it-t3st"), "{:?} teaches the exit: {}", args, err);
    }
    assert!(
        !s.root.join("graph").join("view").join("index.html").exists(),
        "a refused wrap regenerates nothing"
    );
    // machine-local state alone (no env) refuses the same way
    let d = quarry::coord::DispatchState {
        item: "it-loc4".into(),
        item_title: "state-file badge".into(),
        session: "disp".into(),
        globs: vec![],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
    };
    quarry::coord::save_dispatch(&s, &d).unwrap();
    let out = run(None, &["wrap"]);
    assert!(!out.status.success(), "state-file badge refuses without env");
    assert!(String::from_utf8_lossy(&out.stderr).contains("q harvest it-loc4"));
    quarry::coord::clear_dispatch(&s, "it-loc4");
    // badge clear: wrap runs and the boundary regenerates the derived view
    let out = run(None, &["wrap"]);
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
    // a badge on the machine stamps the retirement like any act
    let d = quarry::coord::DispatchState {
        item: "it-unlk".into(),
        item_title: "unlink arc".into(),
        session: "disp".into(),
        globs: vec![],
        acceptance: vec![],
        since: "2026-01-01T00:00:00Z".into(),
        cursor: 0,
        checked: "2026-01-01T00:00:00Z".into(),
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
    // the log records the act: op, actor, badge, note
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
    assert_eq!(ev.get("dispatch").and_then(|v| v.as_str()), Some("it-unlk"));
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
