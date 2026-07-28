use super::*;

/// The workspace root — `demo-server`'s own parent directory. Used both by tests that read the real
/// `demo-app/src/lib.rs` and by tests that need a `root` argument for [`validate`] pointing at a synthetic
/// directory laid out the same way.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("demo-server has a parent directory")
        .to_path_buf()
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// validate — the function this module exists to test; now a plain `Result`-returning function rather than one
// that calls `process::exit` itself, so each failure path can be exercised directly instead of only indirectly
// through its side effects.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_validate_the_real_project_successfully() {
    assert!(validate(&workspace_root()).is_ok());
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_report_io_error_for_unreadable_lib_rs() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let err = validate(tmp.path()).expect_err("a missing demo-app/src/lib.rs must be reported");
    assert!(matches!(err, ValidationError::Io { .. }), "wrong error variant: {err}");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_report_duplicate_gallery_id() {
    // The duplicate check runs before the gallery ids are compared against the real MANIFEST, so a synthetic
    // `lib.rs` can use ids that do not exist in the real catalogue at all and still exercise this path.
    let tmp = tempfile::tempdir().expect("create temp dir");
    let src_dir = tmp.path().join("demo-app").join("src");
    fs::create_dir_all(&src_dir).expect("create demo-app/src");
    fs::write(
        src_dir.join("lib.rs"),
        r#"
            demo_gallery! {
                "panel-synthetic" => shapes::demo_synthetic,
                "panel-synthetic" => shapes::some_other_demo,
            }
        "#,
    )
    .expect("write lib.rs");

    let err = validate(tmp.path()).expect_err("a duplicated gallery id must be rejected");
    assert!(
        matches!(&err, ValidationError::DuplicateGalleryId(id) if id == "panel-synthetic"),
        "wrong error variant: {err}"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_report_catalogue_mismatch() {
    // No demo_gallery! entries at all is the simplest way to guarantee every real MANIFEST id is "missing from
    // the gallery", without needing to know any of their names.
    let tmp = tempfile::tempdir().expect("create temp dir");
    let src_dir = tmp.path().join("demo-app").join("src");
    fs::create_dir_all(&src_dir).expect("create demo-app/src");
    fs::write(src_dir.join("lib.rs"), "no demo_gallery! entries here").expect("write lib.rs");

    let err = validate(tmp.path()).expect_err("an empty gallery must be reported as a catalogue mismatch");
    assert!(matches!(err, ValidationError::CatalogueMismatch(_)), "wrong error variant: {err}");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// extract_gallery_panel_ids
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_extract_every_panel_id_in_order() {
    let lib_rs = r#"
        demo_gallery! {
            "panel-rect" => shapes::demo_rect,
            "panel-circle" => shapes::demo_circle,
        }
    "#;
    assert_eq!(extract_gallery_panel_ids(lib_rs), vec!["panel-rect", "panel-circle"]);
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_extract_every_occurrence_including_duplicates() {
    let lib_rs = r#"
        "panel-rect" => shapes::demo_rect,
        "panel-rect" => shapes::some_other_demo,
    "#;
    assert_eq!(extract_gallery_panel_ids(lib_rs), vec!["panel-rect", "panel-rect"]);
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_ignore_panel_strings_not_followed_by_fat_arrow() {
    // A doc comment or string that merely mentions a panel id, without the `=> module::func` shape that only
    // `demo_gallery!`'s own entries have, must not be picked up.
    let lib_rs = r#"//! See "panel-rect" for an example.
        "panel-circle" => shapes::demo_circle,
    "#;
    assert_eq!(extract_gallery_panel_ids(lib_rs), vec!["panel-circle"]);
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_extract_nothing_from_text_with_no_gallery_entries() {
    assert!(extract_gallery_panel_ids("no panel ids in here at all").is_empty());
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// find_duplicate_gallery_id
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_detect_duplicate_gallery_id() {
    let ids = ["panel-rect".to_string(), "panel-circle".to_string(), "panel-rect".to_string()];
    assert_eq!(find_duplicate_gallery_id(&ids), Some("panel-rect"));
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_accept_unique_gallery_ids() {
    let ids = ["panel-rect".to_string(), "panel-circle".to_string()];
    assert_eq!(find_duplicate_gallery_id(&ids), None);
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_accept_empty_gallery_ids() {
    assert_eq!(find_duplicate_gallery_id(&[]), None);
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Against the real project — this is the test that actually protects against the class of drift this module
// exists to catch, the same role should_detect_real_manifest_has_no_duplicate_ids plays in panels/unit_tests.rs.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_detect_real_gallery_has_no_duplicate_ids() {
    let lib_rs_path = workspace_root().join("demo-app").join("src").join("lib.rs");
    let lib_rs = fs::read_to_string(&lib_rs_path).expect("read demo-app/src/lib.rs");
    let ids = extract_gallery_panel_ids(&lib_rs);
    assert_eq!(find_duplicate_gallery_id(&ids), None, "demo_gallery! has a duplicated panel id");
}
