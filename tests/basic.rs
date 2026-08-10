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
        "the halo is bounded",
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
    assert_eq!(behind[0].src_id, c.front.id);
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
        "halo is 4-11 cells",
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
        "halo bounded",
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
    let err = ops::claim(&s, "vibes", vec![area.front.id.clone()], None, None, Some("assistant".into()), None)
        .unwrap_err();
    assert!(err.to_string().contains("C2"), "got: {}", err);
    // method alone grounds it
    ops::claim(&s, "measured thing", vec![area.front.id.clone()], None,
        Some("log inspection".into()), None, None).unwrap();
    // a file: source grounds it (matrix widened), blob-stamped
    let c = ops::claim(&s, "read off the code", vec![area.front.id.clone()],
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
        "materials resolve per-voxel through layered override stacks",
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
    assert!(text.contains("depends on decision"));
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
        "`geo-pass`: emits layered strata",
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
fn actor_recording_and_safety_prefix() {
    let s = temp_store();
    assert!(quarry::coord::chat_actor(&s, "chat-1").is_none());
    quarry::coord::record_chat_actor(&s, "chat-1", "Fable 5");
    assert_eq!(quarry::coord::chat_actor(&s, "chat-1").as_deref(), Some("Fable 5"));
    assert_eq!(quarry::coord::safe_actor("claude-fable-5"), "claude-fable-5");
    assert_eq!(quarry::coord::safe_actor("Fable 5"), "claude:Fable 5");
}
