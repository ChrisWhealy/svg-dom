use super::*;

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
    let lib_rs_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("demo-server has a parent directory")
        .join("demo-app")
        .join("src")
        .join("lib.rs");
    let lib_rs = fs::read_to_string(&lib_rs_path).expect("read demo-app/src/lib.rs");
    let ids = extract_gallery_panel_ids(&lib_rs);
    assert_eq!(find_duplicate_gallery_id(&ids), None, "demo_gallery! has a duplicated panel id");
}
