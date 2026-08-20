use super::*;

/// The workspace root — `demo-server`'s own parent directory. Used both by tests that read the real
/// `demo-app/src/lib.rs` and by tests that need a `root` argument for [`validate`] pointing at a synthetic
/// directory laid out the same way.
fn workspace_root() -> Result<PathBuf, String> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "demo-server has a parent directory".to_owned())?
        .to_path_buf())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// validate — the function this module exists to test; now a plain `Result`-returning function rather than one
// that calls `process::exit` itself, so each failure path can be exercised directly instead of only indirectly
// through its side effects.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_validate_the_real_project_successfully() -> Result<(), String> {
    validate(&workspace_root()?).map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_report_io_error_for_unreadable_lib_rs() -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let err = match validate(tmp.path()) {
        Err(e) => e,
        Ok(()) => return Err("a missing demo-app/src/lib.rs must be reported".to_owned()),
    };
    if !matches!(err, ValidationError::Io { .. }) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_report_duplicate_gallery_id() -> Result<(), String> {
    // The duplicate check runs before the gallery ids are compared against the real MANIFEST, so a synthetic
    // `lib.rs` can use ids that do not exist in the real catalogue at all and still exercise this path.
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let src_dir = tmp.path().join("demo-app").join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("create demo-app/src: {e:?}"))?;
    fs::write(
        src_dir.join("lib.rs"),
        r#"
            demo_gallery! {
                "panel-synthetic" => shapes::demo_synthetic,
                "panel-synthetic" => shapes::some_other_demo,
            }
        "#,
    )
    .map_err(|e| format!("write lib.rs: {e:?}"))?;

    let err = match validate(tmp.path()) {
        Err(e) => e,
        Ok(()) => return Err("a duplicated gallery id must be rejected".to_owned()),
    };
    if !matches!(&err, ValidationError::DuplicateGalleryId(id) if id == "panel-synthetic") {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_report_catalogue_mismatch() -> Result<(), String> {
    // No demo_gallery! entries at all is the simplest way to guarantee every real MANIFEST id is "missing from
    // the gallery", without needing to know any of their names.
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let src_dir = tmp.path().join("demo-app").join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("create demo-app/src: {e:?}"))?;
    fs::write(src_dir.join("lib.rs"), "no demo_gallery! entries here").map_err(|e| format!("write lib.rs: {e:?}"))?;

    let err = match validate(tmp.path()) {
        Err(e) => e,
        Ok(()) => return Err("an empty gallery must be reported as a catalogue mismatch".to_owned()),
    };
    if !matches!(err, ValidationError::CatalogueMismatch(_)) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// extract_gallery_panel_ids
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_extract_every_panel_id_in_order() -> Result<(), String> {
    let lib_rs = r#"
        demo_gallery! {
            "panel-rect" => shapes::demo_rect,
            "panel-circle" => shapes::demo_circle,
        }
    "#;
    let ids = extract_gallery_panel_ids(lib_rs);
    if ids != vec!["panel-rect", "panel-circle"] {
        return Err(format!("expected [\"panel-rect\", \"panel-circle\"], got {ids:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_extract_every_occurrence_including_duplicates() -> Result<(), String> {
    let lib_rs = r#"
        demo_gallery! {
            "panel-rect" => shapes::demo_rect,
            "panel-rect" => shapes::some_other_demo,
        }
    "#;
    let ids = extract_gallery_panel_ids(lib_rs);
    if ids != vec!["panel-rect", "panel-rect"] {
        return Err(format!("expected [\"panel-rect\", \"panel-rect\"], got {ids:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_ignore_panel_strings_not_followed_by_fat_arrow() -> Result<(), String> {
    // Even inside the invocation body, a bare panel-shaped string with no `=>` immediately after it must not be
    // picked up as an entry.
    let lib_rs = r#"
        demo_gallery! {
            "panel-not-an-entry",
            "panel-circle" => shapes::demo_circle,
        }
    "#;
    let ids = extract_gallery_panel_ids(lib_rs);
    if ids != vec!["panel-circle"] {
        return Err(format!("expected [\"panel-circle\"], got {ids:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_ignore_panel_strings_outside_the_gallery_invocation() -> Result<(), String> {
    // The exact false positive this function's brace-scoped extraction exists to rule out: a comment showing an
    // example entry, sitting outside demo_gallery!'s own braces, has the same `"id" => path::func` shape as a real
    // entry but must not be picked up as one just because it appears somewhere in the file.
    let lib_rs = r#"
        // e.g. "panel-example" => module::function
        demo_gallery! {
            "panel-circle" => shapes::demo_circle,
        }
    "#;
    let ids = extract_gallery_panel_ids(lib_rs);
    if ids != vec!["panel-circle"] {
        return Err(format!("expected [\"panel-circle\"], got {ids:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_skip_demo_gallery_mentions_that_are_not_the_invocation() -> Result<(), String> {
    // Mirrors demo-app/src/lib.rs's own shape: plain-text and doc-comment mentions of `demo_gallery!` (by name,
    // not immediately followed by `{`) sit above the real invocation there — see gallery_invocation_body's own doc
    // comment.
    let lib_rs = r#"
        // generated below by demo_gallery!
        /// see the `demo_gallery!` macro
        demo_gallery! {
            "panel-circle" => shapes::demo_circle,
        }
    "#;
    let ids = extract_gallery_panel_ids(lib_rs);
    if ids != vec!["panel-circle"] {
        return Err(format!("expected [\"panel-circle\"], got {ids:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_extract_nothing_from_text_with_no_gallery_entries() -> Result<(), String> {
    if !extract_gallery_panel_ids("no panel ids in here at all").is_empty() {
        return Err("expected no panel ids".to_owned());
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// find_duplicate_gallery_id
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_detect_duplicate_gallery_id() -> Result<(), String> {
    let ids = ["panel-rect".to_string(), "panel-circle".to_string(), "panel-rect".to_string()];
    let found = find_duplicate_gallery_id(&ids);
    if found != Some("panel-rect") {
        return Err(format!("expected Some(\"panel-rect\"), got {found:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_accept_unique_gallery_ids() -> Result<(), String> {
    let ids = ["panel-rect".to_string(), "panel-circle".to_string()];
    let found = find_duplicate_gallery_id(&ids);
    if found.is_some() {
        return Err(format!("expected None, got {found:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_accept_empty_gallery_ids() -> Result<(), String> {
    let found = find_duplicate_gallery_id(&[]);
    if found.is_some() {
        return Err(format!("expected None, got {found:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Against the real project — this is the test that actually protects against the class of drift this module
// exists to catch, the same role should_detect_real_manifest_has_no_duplicate_ids plays in panels/unit_tests.rs.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_detect_real_gallery_has_no_duplicate_ids() -> Result<(), String> {
    let lib_rs_path = workspace_root()?.join("demo-app").join("src").join("lib.rs");
    let lib_rs = fs::read_to_string(&lib_rs_path).map_err(|e| format!("read demo-app/src/lib.rs: {e:?}"))?;
    let ids = extract_gallery_panel_ids(&lib_rs);
    let found = find_duplicate_gallery_id(&ids);
    if found.is_some() {
        return Err(format!("demo_gallery! has a duplicated panel id: {found:?}"));
    }
    Ok(())
}
