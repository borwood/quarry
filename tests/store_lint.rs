//! The locale lint (dc-g5x5), CI half: outside src/store.rs no source line
//! may resolve a store around the one resolver — the fenced tokens are the
//! discovery walk-up and the pin reads (`var("QUARRY_STORE"` /
//! `var("QUARRY_HOME"`). Every verb rides Store::resolve; hooks ride
//! resolve_store with their input's identity; nothing else decides where
//! acts land. `q wrap` runs the same scan (store::lint_locale_sources)
//! inside quarry's repo.

#[test]
fn store_resolution_lives_only_in_the_store_module() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let offenders = quarry::store::lint_locale_sources(&src);
    assert!(
        offenders.is_empty(),
        "store resolution outside src/store.rs — one resolver owns the graph locale (dc-g5x5): {:?}",
        offenders
    );
}
