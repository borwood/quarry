//! The atom lint (dc-nnf5), CI half: `front.title` outside src/surface.rs
//! fails the build's test run. The invariant is total — render, matching,
//! and mutation all ride the surface module's renderers and raw accessors —
//! so the check needs no judgment about which reference is "formatting".
//! `q wrap` runs the same scan (surface::lint_sources) inside quarry's repo.

#[test]
fn front_title_lives_only_in_the_surface_module() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let offenders = quarry::surface::lint_sources(&src);
    assert!(
        offenders.is_empty(),
        "front.title formatted outside src/surface.rs — every register rides the atom (dc-nnf5): {:?}",
        offenders
    );
}
