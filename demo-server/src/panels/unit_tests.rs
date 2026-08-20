use super::*;
use std::fs;

const PORT: u16 = 8080;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The real project's `demo/` directory — the actual, current catalogue this whole module exists to assemble.
/// Tests only ever read from here; nothing in this module writes into it. Locating it via `CARGO_MANIFEST_DIR`
/// (rather than a relative path, which would depend on the test binary's working directory) is what makes this
/// work the same way under `cargo test`, `cargo test -p demo-server`, and `cargo llvm-cov`/`nextest` alike.
fn project_demo_dir() -> Result<PathBuf, String> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "demo-server has a parent directory".to_owned())?
        .join("demo"))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Copies the real project's `index.template.html` and every `panels/*.html` fragment into `dest`, so a test
/// can then corrupt exactly one file in isolation without ever touching the real ones. Only the tests that
/// deliberately break something need this — [`assembles_the_real_catalogue_without_error`] below reads
/// straight from [`project_demo_dir`] instead, since it has nothing to corrupt.
fn seed_fixtures(dest: &Path) -> Result<(), String> {
    let src = project_demo_dir()?;
    fs::create_dir_all(dest.join("panels")).map_err(|e| format!("create panels dir: {e:?}"))?;
    fs::copy(src.join("index.template.html"), dest.join("index.template.html"))
        .map_err(|e| format!("copy template: {e:?}"))?;
    for entry in fs::read_dir(src.join("panels")).map_err(|e| format!("read real panels dir: {e:?}"))? {
        let entry = entry.map_err(|e| format!("dir entry: {e:?}"))?;
        fs::copy(entry.path(), dest.join("panels").join(entry.file_name()))
            .map_err(|e| format!("copy fragment: {e:?}"))?;
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Catalogue invariants, checked directly against the real MANIFEST/menu/fragments — these are the tests that
// actually protect against the class of drift this module exists to catch; everything below them instead
// targets one check's logic in isolation with a small synthetic fixture.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_detect_real_manifest_has_no_duplicate_ids() -> Result<(), String> {
    check_unique_manifest_ids(MANIFEST).map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_assemble_real_catalogue() -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let out_path = tmp.path().join("index.html");

    let result = assemble(&project_demo_dir()?, &out_path, PORT);
    if result.is_err() {
        return Err(format!("assemble failed against the real catalogue: {:?}", result.err()));
    }

    let html = fs::read_to_string(&out_path).map_err(|e| format!("read assembled output: {e:?}"))?;
    if !html.contains(r#"id="panel-rect""#) {
        return Err("assembled output is missing a known real panel".to_owned());
    }
    if html.contains("{{") {
        return Err("assembled output still contains an unresolved placeholder".to_owned());
    }
    // One <section> and one canvas <div> per panel, and nothing left over from a stray duplicate id.
    let count = html.matches(r#"class="section" id="panel-rect""#).count();
    if count != 1 {
        return Err(format!("expected exactly one panel-rect section, got {count}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// assemble, exercised against deliberately corrupted copies of the real fixtures
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_reject_duplicate_placeholder() -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    seed_fixtures(tmp.path())?;

    let template_path = tmp.path().join("index.template.html");
    let template = fs::read_to_string(&template_path).map_err(|e| format!("read template: {e:?}"))?;
    let doubled = template.replacen(PANELS_PLACEHOLDER, &format!("{PANELS_PLACEHOLDER}\n{PANELS_PLACEHOLDER}"), 1);
    fs::write(&template_path, doubled).map_err(|e| format!("write corrupted template: {e:?}"))?;

    let out_path = tmp.path().join("index.html");
    let err = match assemble(tmp.path(), &out_path, PORT) {
        Err(e) => e,
        Ok(_) => return Err("a doubled placeholder must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::DuplicatePlaceholder { .. }) {
        return Err(format!("wrong error variant: {err}"));
    }
    if out_path.exists() {
        return Err("must not write output after a failed assembly".to_owned());
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_missing_placeholder() -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    seed_fixtures(tmp.path())?;

    let template_path = tmp.path().join("index.template.html");
    let template = fs::read_to_string(&template_path).map_err(|e| format!("read template: {e:?}"))?;
    fs::write(&template_path, template.replace(MENU_PLACEHOLDER, ""))
        .map_err(|e| format!("write corrupted template: {e:?}"))?;

    let out_path = tmp.path().join("index.html");
    let err = match assemble(tmp.path(), &out_path, PORT) {
        Err(e) => e,
        Ok(_) => return Err("a missing placeholder must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::MissingPlaceholder { .. }) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_fragment_id_mismatch() -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    seed_fixtures(tmp.path())?;

    let fragment_path = tmp.path().join("panels").join("panel-rect.html");
    let fragment = fs::read_to_string(&fragment_path).map_err(|e| format!("read fragment: {e:?}"))?;
    fs::write(
        &fragment_path,
        fragment.replace(r#"id="panel-rect""#, r#"id="panel-rect-oops""#),
    )
    .map_err(|e| format!("write corrupted fragment: {e:?}"))?;

    let out_path = tmp.path().join("index.html");
    let err = match assemble(tmp.path(), &out_path, PORT) {
        Err(e) => e,
        Ok(_) => return Err("a fragment/id mismatch must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::FragmentIdMismatch { .. }) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_orphaned_fragment() -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    seed_fixtures(tmp.path())?;

    // A fragment file with no matching MANIFEST entry — the "removed from MANIFEST but left on disk" case.
    fs::write(
        tmp.path().join("panels").join("panel-orphan.html"),
        "<section class=\"section\" id=\"panel-orphan\"></section>\n",
    )
    .map_err(|e| format!("write orphaned fragment: {e:?}"))?;

    let out_path = tmp.path().join("index.html");
    let err = match assemble(tmp.path(), &out_path, PORT) {
        Err(e) => e,
        Ok(_) => return Err("an orphaned fragment must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::CatalogueMismatch(_)) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_missing_fragment_file() -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    seed_fixtures(tmp.path())?;

    // A MANIFEST entry with no fragment ever created for it — the "added to MANIFEST but never created" case.
    fs::remove_file(tmp.path().join("panels").join("panel-rect.html"))
        .map_err(|e| format!("remove fragment: {e:?}"))?;

    let out_path = tmp.path().join("index.html");
    let err = match assemble(tmp.path(), &out_path, PORT) {
        Err(e) => e,
        Ok(_) => return Err("a missing fragment file must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::Io { .. }) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Smaller unit tests of individual checks, against synthetic input rather than the real catalogue
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_detect_duplicate_manifest_id() -> Result<(), String> {
    const DUPLICATED: &[Entry] = &[
        Entry::Panel { id: "panel-a", label: "a" },
        Entry::Panel { id: "panel-b", label: "b" },
        Entry::Panel {
            id: "panel-a",
            label: "a again",
        },
    ];
    let err = match check_unique_manifest_ids(DUPLICATED) {
        Err(e) => e,
        Ok(()) => return Err("a duplicated id must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::DuplicateManifestId("panel-a")) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_accept_unique_manifest_ids() -> Result<(), String> {
    const UNIQUE: &[Entry] = &[
        Entry::Category("Category"),
        Entry::Panel { id: "panel-a", label: "a" },
        Entry::Panel { id: "panel-b", label: "b" },
    ];
    check_unique_manifest_ids(UNIQUE).map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_count_zero_placeholders_as_missing() -> Result<(), String> {
    let err = match check_placeholder_count("no placeholder here", Path::new("t.html"), PANELS_PLACEHOLDER) {
        Err(e) => e,
        Ok(()) => return Err("zero occurrences must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::MissingPlaceholder { .. }) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_count_one_placeholder_as_ok() -> Result<(), String> {
    check_placeholder_count("one {{PANELS}} here", Path::new("t.html"), PANELS_PLACEHOLDER)
        .map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_count_two_placeholders_as_duplicate() -> Result<(), String> {
    let err = match check_placeholder_count("{{PANELS}} and {{PANELS}}", Path::new("t.html"), PANELS_PLACEHOLDER) {
        Err(e) => e,
        Ok(()) => return Err("two occurrences must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::DuplicatePlaceholder { count: 2, .. }) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_extract_every_data_target() -> Result<(), String> {
    let menu = r#"<button data-target="panel-a">a</button><button data-target="panel-b">b</button>"#;
    let targets = extract_data_targets(menu);
    let expected = HashSet::from(["panel-a".to_string(), "panel-b".to_string()]);
    if targets != expected {
        return Err(format!("expected {expected:?}, got {targets:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_escape_amp_lt_and_gt() -> Result<(), String> {
    let cases = [
        ("Clipping & Masking", "Clipping &amp; Masking"),
        ("a < b", "a &lt; b"),
        ("a > b", "a &gt; b"),
        ("plain text", "plain text"),
    ];
    for (input, expected) in cases {
        let actual = escape_text(input);
        if actual != expected {
            return Err(format!("escape_text({input:?}): expected {expected:?}, got {actual:?}"));
        }
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_detect_real_manifest_has_valid_panel_id_format() -> Result<(), String> {
    check_panel_id_format(MANIFEST).map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_accept_well_formed_panel_ids() -> Result<(), String> {
    const VALID: &[Entry] = &[
        Entry::Panel { id: "panel-a", label: "a" },
        Entry::Panel { id: "panel-rect-2", label: "b" },
    ];
    check_panel_id_format(VALID).map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_panel_id_missing_the_prefix() -> Result<(), String> {
    const INVALID: &[Entry] = &[Entry::Panel { id: "rect", label: "a" }];
    let err = match check_panel_id_format(INVALID) {
        Err(e) => e,
        Ok(()) => return Err("a missing panel- prefix must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::InvalidPanelId("rect")) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_panel_id_with_no_suffix() -> Result<(), String> {
    const INVALID: &[Entry] = &[Entry::Panel { id: "panel-", label: "a" }];
    let err = match check_panel_id_format(INVALID) {
        Err(e) => e,
        Ok(()) => return Err("an empty suffix must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::InvalidPanelId("panel-")) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_panel_id_with_disallowed_characters() -> Result<(), String> {
    const INVALID: &[Entry] = &[Entry::Panel { id: "panel-Rect", label: "a" }];
    let err = match check_panel_id_format(INVALID) {
        Err(e) => e,
        Ok(()) => return Err("an uppercase character must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::InvalidPanelId("panel-Rect")) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_not_leave_any_placeholders_in_clean_output() -> Result<(), String> {
    check_no_leftover_placeholders("<html>no placeholders left</html>")
        .map_err(|e| format!("expected Ok, got {e:?}"))?;
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_detect_a_leftover_placeholder() -> Result<(), String> {
    let err = match check_no_leftover_placeholders("<html>{{OOPS}}</html>") {
        Err(e) => e,
        Ok(()) => return Err("a leftover token must be rejected".to_owned()),
    };
    if !matches!(err, AssembleError::LeftoverPlaceholder(_)) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}
