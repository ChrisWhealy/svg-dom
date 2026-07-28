use super::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// StagePaths
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_derive_stage_paths_from_target_dir() {
    let stage = StagePaths::new(Path::new("/tmp/target"));
    assert_eq!(stage.stage_dir, Path::new("/tmp/target/demo-gallery"));
    assert_eq!(stage.demo_dir, Path::new("/tmp/target/demo-gallery/demo"));
    assert_eq!(stage.pkg_dir, Path::new("/tmp/target/demo-gallery/pkg"));
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// copy_asset
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_copy_asset_successfully() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let src = tmp.path().join("src.txt");
    let dest = tmp.path().join("dest.txt");
    fs::write(&src, b"hello").expect("write src");

    assert!(copy_asset(&src, &dest).is_ok());
    assert_eq!(fs::read_to_string(&dest).expect("read dest"), "hello");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_report_copy_asset_error_with_both_paths() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let src = tmp.path().join("missing.txt");
    let dest = tmp.path().join("dest.txt");

    let err = copy_asset(&src, &dest).expect_err("copying a missing file must fail");
    assert!(
        matches!(&err, BuildError::CopyAsset { src: s, dest: d, .. } if *s == src && *d == dest),
        "wrong error variant: {err}"
    );
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_wasm
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_report_spawn_error_when_wasm_pack_is_not_on_path() {
    // Points `PATH` at an empty directory for the duration of this one call, so `Command::new("wasm-pack")` fails
    // to resolve to a binary at all — this is what exercises `WasmSpawn` without needing to actually run (or
    // deliberately break) a real wasm-pack build. `cargo-nextest` runs each test in its own process, so mutating
    // process-wide `PATH` here cannot affect any other test.
    let empty_path_dir = tempfile::tempdir().expect("create temp dir");
    let out_dir = tempfile::tempdir().expect("create temp dir");
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

    let err = result.expect_err("wasm-pack must not be found on an empty PATH");
    assert!(matches!(err, BuildError::WasmSpawn(_)), "wrong error variant: {err}");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_gallery — phase short-circuiting
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn should_stop_at_validate_without_reaching_later_phases() {
    // An empty `root` has no `demo-app/src/lib.rs`, so `validate::validate` must fail — and, because
    // `build_gallery` short-circuits via `?`, `panels::assemble` and `build_wasm` (which would otherwise spawn a
    // real `wasm-pack`) must never run. This is what actually proves the phases are chained in order, not just
    // that each one works in isolation.
    let root = tempfile::tempdir().expect("create temp dir");
    let target_dir = tempfile::tempdir().expect("create temp dir");
    let stage = StagePaths::new(target_dir.path());

    let err = build_gallery(root.path(), &stage, 8080).expect_err("an empty root must fail validation");
    assert!(matches!(err, BuildError::Validate(_)), "wrong error variant: {err}");
}
