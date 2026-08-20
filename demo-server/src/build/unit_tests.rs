use super::*;

/// The workspace root — `demo-server`'s own parent directory. See `validate::unit_tests`'s identical helper.
fn workspace_root() -> Result<PathBuf, String> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "demo-server has a parent directory".to_owned())?
        .to_path_buf())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// prepare_gallery — the end-to-end staging check: everything build_gallery does except the wasm rebuild, run
// against the real project. This is what actually proves catalogue validation, fragment validation, template
// assembly, port substitution, and asset copying stay wired together correctly as one pipeline, not just that
// each phase's own unit tests (in panels::unit_tests and validate::unit_tests) pass in isolation — without paying
// for a real wasm-pack build to do it.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_prepare_the_real_gallery_without_building_wasm() -> Result<(), String> {
    let stage_root = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let stage = StagePaths::new(stage_root.path());

    let result = prepare_gallery(&workspace_root()?, &stage, 8080);
    if result.is_err() {
        return Err(format!("prepare_gallery failed against the real project: {:?}", result.err()));
    }

    let html =
        fs::read_to_string(stage.demo_dir.join("index.html")).map_err(|e| format!("read staged index.html: {e:?}"))?;
    if !html.contains(r#"id="panel-rect""#) {
        return Err("staged index.html is missing a known real panel".to_owned());
    }
    if !html.contains("http://127.0.0.1:8080/demo/") {
        return Err("staged index.html did not substitute the port placeholder".to_owned());
    }
    if html.contains("{{") {
        return Err("staged index.html still contains an unresolved placeholder".to_owned());
    }

    if !stage.demo_dir.join("style.css").is_file() {
        return Err("style.css was not staged".to_owned());
    }
    if !stage.demo_dir.join("view-demo.svg").is_file() {
        return Err("view-demo.svg was not staged".to_owned());
    }

    // The whole point of prepare_gallery vs. build_gallery: staging must not touch pkg/ at all.
    if stage.pkg_dir.exists() {
        return Err("prepare_gallery must not build (or even create a directory for) the wasm package".to_owned());
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// StagePaths
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_derive_stage_paths_from_target_dir() -> Result<(), String> {
    let stage = StagePaths::new(Path::new("/tmp/target"));
    if stage.stage_dir != Path::new("/tmp/target/demo-gallery") {
        return Err(format!("unexpected stage_dir: {:?}", stage.stage_dir));
    }
    if stage.demo_dir != Path::new("/tmp/target/demo-gallery/demo") {
        return Err(format!("unexpected demo_dir: {:?}", stage.demo_dir));
    }
    if stage.pkg_dir != Path::new("/tmp/target/demo-gallery/pkg") {
        return Err(format!("unexpected pkg_dir: {:?}", stage.pkg_dir));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// copy_asset
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_copy_asset_successfully() -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let src = tmp.path().join("src.txt");
    let dest = tmp.path().join("dest.txt");
    fs::write(&src, b"hello").map_err(|e| format!("write src: {e:?}"))?;

    copy_asset(&src, &dest).map_err(|e| format!("expected Ok, got {e:?}"))?;
    let contents = fs::read_to_string(&dest).map_err(|e| format!("read dest: {e:?}"))?;
    if contents != "hello" {
        return Err(format!("expected \"hello\", got {contents:?}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_report_copy_asset_error_with_both_paths() -> Result<(), String> {
    let tmp = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let src = tmp.path().join("missing.txt");
    let dest = tmp.path().join("dest.txt");

    let err = match copy_asset(&src, &dest) {
        Err(e) => e,
        Ok(()) => return Err("copying a missing file must fail".to_owned()),
    };
    if !matches!(&err, BuildError::CopyAsset { src: s, dest: d, .. } if *s == src && *d == dest) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_wasm
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_report_spawn_error_when_wasm_pack_is_not_on_path() -> Result<(), String> {
    // Points `PATH` at an empty directory for the duration of this one call, so `Command::new("wasm-pack")` fails
    // to resolve to a binary at all — this is what exercises `WasmSpawn` without needing to actually run (or
    // deliberately break) a real wasm-pack build. `cargo-nextest` runs each test in its own process, so mutating
    // process-wide `PATH` here cannot affect any other test.
    let empty_path_dir = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let out_dir = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let original_path = std::env::var_os("PATH");

    // SAFETY: this process is single-threaded for the duration of this test (cargo-nextest gives each test its
    // own process), so no other thread can observe `PATH` in an inconsistent state between the set and restore.
    unsafe {
        std::env::set_var("PATH", empty_path_dir.path());
    }
    let result = build_wasm(Path::new("."), out_dir.path());
    unsafe {
        match &original_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }
    }

    let err = match result {
        Err(e) => e,
        Ok(()) => return Err("wasm-pack must not be found on an empty PATH".to_owned()),
    };
    if !matches!(err, BuildError::WasmSpawn(_)) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_gallery — phase short-circuiting
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_stop_at_validate_without_reaching_later_phases() -> Result<(), String> {
    // An empty `root` has no `demo-app/src/lib.rs`, so `validate::validate` must fail — and, because
    // `build_gallery` short-circuits via `?`, `panels::assemble` and `build_wasm` (which would otherwise spawn a
    // real `wasm-pack`) must never run. This is what actually proves the phases are chained in order, not just
    // that each one works in isolation.
    let root = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let target_dir = tempfile::tempdir().map_err(|e| format!("create temp dir: {e:?}"))?;
    let stage = StagePaths::new(target_dir.path());

    let err = match build_gallery(root.path(), &stage, 8080) {
        Err(e) => e,
        Ok(()) => return Err("an empty root must fail validation".to_owned()),
    };
    if !matches!(err, BuildError::Validate(_)) {
        return Err(format!("wrong error variant: {err}"));
    }
    Ok(())
}
