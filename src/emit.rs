//! Non-fatal output (it-8tcy): a closed pipe is a normal fate for CLI
//! output — `head` takes its line and leaves, a terminal dies mid-print —
//! and must end output quietly, never panic. Worse than the panic itself
//! was its sequencing blast: real state work (readings sweeps, area
//! watermarks, ratification, log events, view regeneration) is sequenced
//! AFTER prints in the verb paths, so a mid-print panic skipped mutations,
//! not just text. These macros are the one fix for both: every print in
//! the binary rides them, they swallow io errors, and so no mutation can
//! be skipped by an output failure, whatever order the code runs in.
//! What a dead pipe loses is only the delivery — the printed surfaces are
//! DERIVED from graph state, and the per-node homework re-derives on
//! demand at `q query homework <node>`.

/// `println!` that swallows io errors — a closed pipe ends output quietly,
/// never panics, and never skips the state work sequenced after it.
#[macro_export]
macro_rules! outln {
    ($($arg:tt)*) => {{
        use ::std::io::Write as _;
        let _ = ::std::writeln!(::std::io::stdout(), $($arg)*);
    }};
}

/// `print!` that swallows io errors — the no-newline sibling of `outln!`.
#[macro_export]
macro_rules! out {
    ($($arg:tt)*) => {{
        use ::std::io::Write as _;
        let _ = ::std::write!(::std::io::stdout(), $($arg)*);
    }};
}

/// `eprintln!` that swallows io errors — stderr dies the same deaths
/// stdout does, and a refusal path must still reach its `exit(2)`.
#[macro_export]
macro_rules! errln {
    ($($arg:tt)*) => {{
        use ::std::io::Write as _;
        let _ = ::std::writeln!(::std::io::stderr(), $($arg)*);
    }};
}
