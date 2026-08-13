# Dispatch report: q serve — the view rendered per request (it-kxdu)

Agent report, registered verbatim by the dispatcher at harvest.

## What changed and the mechanism

- **src/view.rs** — added `pub const DEFAULT_PORT: u16 = 7171`, `pub fn serve(store: &Store, listener: TcpListener) -> Result<()>`, and private `fn respond(...)`. Std-only (`std::net::TcpListener`), zero new dependencies, no cache, no watcher. The accept loop reads each request head minimally (until `\r\n\r\n`, a 16KB cap, or a 5s stall), then calls the existing `view::render(store)` — which re-reads nodes and log from disk every call (verified `load_all`/`read_log` have no cache) — and answers `HTTP/1.1 200 OK` with `Content-Type: text/html; charset=utf-8`, `Content-Length`, `Connection: close`. Render error → 500 with the error text as text/plain. Any path serves the page (hash routing). Failed accepts continue; read errors break out silently (liveness probes don't spam the console); write errors log per-connection and never kill the loop. `view::write` untouched.
- **src/main.rs** — `Serve { port: Option<u16> }` variant (`--port`, default 7171, after_help citing dc-f79h) and its match arm: bind 127.0.0.1:port, print the confirmation, enter the loop. Also the sanctioned `q view --open` upgrade: a 200ms `TcpStream::connect_timeout` probe of the default port; if something listens, `--open` opens the served URL and says so; otherwise it opens the baked file exactly as before. The baked write runs unconditionally either way (dc-5pb3 preserved).

## Build and tests

`cargo build --release`: clean, finished in 16.41s, no warnings.

`cargo test` raw result lines (including filter selecting `test result:` lines, nothing excluded):

```
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (lib unittests)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (main unittests)
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.32s  (tests/basic.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (tests/surface_lint.rs)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s   (doc-tests)
```

No test homework.

## Acceptance outcome

"A refresh of a long-lived tab always shows the current graph, with no regeneration act anywhere": met by construction (render runs inside the request handler; no state between requests to go stale) and evidenced live. Started `./target/release/q.exe serve`; startup printed `✔ serving the view at http://127.0.0.1:7171 — every request renders the live graph; Ctrl-C stops`. Request 1 arrived with embedded `"generated":"2026-08-12T11:57:24Z"`; request 2 (different path) carried `"generated":"2026-08-12T11:57:25Z"` — a different timestamp, proving fresh render, not replay, and loop survival (four requests total including a HEAD). Headers confirmed. `q view` run while the loop was up still baked the file normally. Server killed after verification; port confirmed closed.

## Reflections (agent's own words)

- The `view --open` probe is a bare TCP connect, so anything squatting on 7171 reads as "a serve loop is up" and the browser opens a stranger. A stricter check (GET + sniff the title) is a five-line hardening if it ever bites. Related gap: serve on a non-default port is invisible to `--open`, which only probes the default. Both feel acceptable for a localhost single-user tool.
- The loop is single-threaded serial: a slow client holds the next request out. Read/write timeouts (5s/30s) bound the damage; for one long-lived tab on loopback that's the right amount of machinery — threads would be ceremony — but it's a real ceiling if two browsers plus a crawler ever point at it.
- Surprise: probes (connections sending nothing) initially would have logged failure noise on every `view --open`; empty-head connections now silently drop. That felt like the loop minding its own business rather than swallowing errors — write failures mid-response still log.
- Minor friction: the worktree carried unharvested it-6gj9/it-5j34 changes in src/view.rs — these edits sit downstream and both ride each served request, as the brief predicted.
