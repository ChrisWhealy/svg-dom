use super::*;
use std::fs;

const PORT: u16 = 8080;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The real project's `demo/` directory — the actual, current catalogue this whole module exists to assemble.
/// Tests only ever read from here; nothing in this module writes into it. Locating it via `CARGO_MANIFEST_DIR`
/// (rather than a relative path, which would depend on the test binary's working directory) is what makes this
/// work the same way under `cargo test`, `cargo test -p demo-server`, and `cargo llvm-cov`/`nextest` alike.
fn project_demo_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("demo-server has a parent directory")
        .join("demo")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Copies the real project's `index.template.html` and every `panels/*.html` fragment into `dest`, so a test
/// can then corrupt exactly one file in isolation without ever touching the real ones. Only the tests that
/// deliberately break something need this — [`assembles_the_real_catalogue_without_error`] below reads
/// straight from [`project_demo_dir`] instead, since it has nothing to corrupt.
fn seed_fixtures(dest: &Path) {
    let src = project_demo_dir();
    fs::create_dir_all(dest.join("panels")).expect("create panels dir");
    fs::copy(src.join("index.template.html"), dest.join("index.template.html")).expect("copy template");
    for entry in fs::read_dir(src.join("panels")).expect("read real panels dir") {
        let entry = entry.expect("dir entry");
        fs::copy(entry.path(), dest.join("panels").join(entry.file_name())).expect("copy fragment");
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Catalogue invariants, checked directly against the real MANIFEST/menu/fragments — these are the tests that
// actually protect against the class of drift this module exists to catch; everything below them instead
// targets one check's logic in isolation with a small synthetic fixture.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_detect_real_manifest_has_no_duplicate_ids() {
    assert!(check_unique_manifest_ids(MANIFEST).is_ok());
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_assemble_real_catalogue() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let out_path = tmp.path().join("index.html");

    let result = assemble(&project_demo_dir(), &out_path, PORT);
    assert!(result.is_ok(), "assemble failed against the real catalogue: {:?}", result.err());

    let html = fs::read_to_string(&out_path).expect("read assembled output");
    assert!(
        html.contains(r#"id="panel-rect""#),
        "assembled output is missing a known real panel"
    );
    assert!(
        !html.contains("{{"),
        "assembled output still contains an unresolved placeholder"
    );
    // One <section> and one canvas <div> per panel, and nothing left over from a stray duplicate id.
    assert_eq!(html.matches(r#"class="section" id="panel-rect""#).count(), 1);
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// assemble, exercised against deliberately corrupted copies of the real fixtures
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_reject_duplicate_placeholder() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    seed_fixtures(tmp.path());

    let template_path = tmp.path().join("index.template.html");
    let template = fs::read_to_string(&template_path).expect("read template");
    let doubled = template.replacen(PANELS_PLACEHOLDER, &format!("{PANELS_PLACEHOLDER}\n{PANELS_PLACEHOLDER}"), 1);
    fs::write(&template_path, doubled).expect("write corrupted template");

    let out_path = tmp.path().join("index.html");
    let err = assemble(tmp.path(), &out_path, PORT).expect_err("a doubled placeholder must be rejected");
    assert!(
        matches!(err, AssembleError::DuplicatePlaceholder { .. }),
        "wrong error variant: {err}"
    );
    assert!(!out_path.exists(), "must not write output after a failed assembly");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_missing_placeholder() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    seed_fixtures(tmp.path());

    let template_path = tmp.path().join("index.template.html");
    let template = fs::read_to_string(&template_path).expect("read template");
    fs::write(&template_path, template.replace(MENU_PLACEHOLDER, "")).expect("write corrupted template");

    let out_path = tmp.path().join("index.html");
    let err = assemble(tmp.path(), &out_path, PORT).expect_err("a missing placeholder must be rejected");
    assert!(
        matches!(err, AssembleError::MissingPlaceholder { .. }),
        "wrong error variant: {err}"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_fragment_id_mismatch() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    seed_fixtures(tmp.path());

    let fragment_path = tmp.path().join("panels").join("panel-rect.html");
    let fragment = fs::read_to_string(&fragment_path).expect("read fragment");
    fs::write(
        &fragment_path,
        fragment.replace(r#"id="panel-rect""#, r#"id="panel-rect-oops""#),
    )
    .expect("write corrupted fragment");

    let out_path = tmp.path().join("index.html");
    let err = assemble(tmp.path(), &out_path, PORT).expect_err("a fragment/id mismatch must be rejected");
    assert!(
        matches!(err, AssembleError::FragmentIdMismatch { .. }),
        "wrong error variant: {err}"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_orphaned_fragment() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    seed_fixtures(tmp.path());

    // A fragment file with no matching MANIFEST entry — the "removed from MANIFEST but left on disk" case.
    fs::write(
        tmp.path().join("panels").join("panel-orphan.html"),
        "<section class=\"section\" id=\"panel-orphan\"></section>\n",
    )
    .expect("write orphaned fragment");

    let out_path = tmp.path().join("index.html");
    let err = assemble(tmp.path(), &out_path, PORT).expect_err("an orphaned fragment must be rejected");
    assert!(matches!(err, AssembleError::CatalogueMismatch(_)), "wrong error variant: {err}");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_reject_missing_fragment_file() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    seed_fixtures(tmp.path());

    // A MANIFEST entry with no fragment ever created for it — the "added to MANIFEST but never created" case.
    fs::remove_file(tmp.path().join("panels").join("panel-rect.html")).expect("remove fragment");

    let out_path = tmp.path().join("index.html");
    let err = assemble(tmp.path(), &out_path, PORT).expect_err("a missing fragment file must be rejected");
    assert!(matches!(err, AssembleError::Io { .. }), "wrong error variant: {err}");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Smaller unit tests of individual checks, against synthetic input rather than the real catalogue
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_detect_duplicate_manifest_id() {
    const DUPLICATED: &[Entry] = &[
        Entry::Panel { id: "panel-a", label: "a" },
        Entry::Panel { id: "panel-b", label: "b" },
        Entry::Panel {
            id: "panel-a",
            label: "a again",
        },
    ];
    let err = check_unique_manifest_ids(DUPLICATED).expect_err("a duplicated id must be rejected");
    assert!(
        matches!(err, AssembleError::DuplicateManifestId("panel-a")),
        "wrong error variant: {err}"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_accept_unique_manifest_ids() {
    const UNIQUE: &[Entry] = &[
        Entry::Category("Category"),
        Entry::Panel { id: "panel-a", label: "a" },
        Entry::Panel { id: "panel-b", label: "b" },
    ];
    assert!(check_unique_manifest_ids(UNIQUE).is_ok());
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_count_zero_placeholders_as_missing() {
    let err = check_placeholder_count("no placeholder here", Path::new("t.html"), PANELS_PLACEHOLDER)
        .expect_err("zero occurrences must be rejected");
    assert!(matches!(err, AssembleError::MissingPlaceholder { .. }));
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_count_one_placeholder_as_ok() {
    assert!(check_placeholder_count("one {{PANELS}} here", Path::new("t.html"), PANELS_PLACEHOLDER).is_ok());
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_count_two_placeholders_as_duplicate() {
    let err = check_placeholder_count("{{PANELS}} and {{PANELS}}", Path::new("t.html"), PANELS_PLACEHOLDER)
        .expect_err("two occurrences must be rejected");
    assert!(matches!(err, AssembleError::DuplicatePlaceholder { count: 2, .. }));
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_extract_every_data_target() {
    let menu = r#"<button data-target="panel-a">a</button><button data-target="panel-b">b</button>"#;
    let targets = extract_data_targets(menu);
    assert_eq!(targets, HashSet::from(["panel-a".to_string(), "panel-b".to_string()]));
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_escape_ampersands_only() {
    assert_eq!(escape_amp("Clipping & Masking"), "Clipping &amp; Masking");
    assert_eq!(escape_amp("plain text"), "plain text");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_not_leave_any_placeholders_in_clean_output() {
    assert!(check_no_leftover_placeholders("<html>no placeholders left</html>").is_ok());
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_detect_a_leftover_placeholder() {
    let err = check_no_leftover_placeholders("<html>{{OOPS}}</html>").expect_err("a leftover token must be rejected");
    assert!(matches!(err, AssembleError::LeftoverPlaceholder(_)));
}
