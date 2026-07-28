//! Cross-checks `demo-server`'s own panel manifest ([`panels::panel_ids`]) against `demo-app`'s `demo_gallery!`
//! list, so the two id lists — declared in separate crates, in different forms (HTML-oriented here,
//! function-call-oriented there), for reasons explained in each one's own doc comment — cannot silently drift
//! apart the way the old hand-written `DEMO_SOURCES` array and the old hand-written menu once did (see
//! `demo-app/src/lib.rs`'s `demo_gallery!` doc comment for that history).
//!
//! This reads `demo-app/src/lib.rs` as plain text and extracts every panel id from the `demo_gallery!` invocation,
//! rather than depending on `demo-app` as a library: that crate builds to a wasm `cdylib` for the browser, not
//! something a native binary like `demo-server` can link against.

use std::{fs, path::Path, process};

use crate::panels;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Checks that every panel id in `demo-server`'s [`panels::panel_ids`] has a matching `demo_gallery!` entry in
/// `demo-app/src/lib.rs`, and vice versa. A failure here is fatal, for the same reason a stale `index.html` or a
/// failed wasm build is: better to refuse to serve a gallery already known to be inconsistent than to leave someone
/// debugging a blank panel or a missing menu entry by hand.
pub fn validate(root: &Path) {
    let lib_rs_path = root.join("demo-app").join("src").join("lib.rs");
    let lib_rs = match fs::read_to_string(&lib_rs_path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("aborting: could not read {} ({err})", lib_rs_path.display());
            process::exit(1);
        },
    };

    let gallery_ids = extract_gallery_panel_ids(&lib_rs);
    let manifest_ids = panels::panel_ids();

    let missing_from_gallery: Vec<_> = manifest_ids.iter().filter(|id| !gallery_ids.iter().any(|g| g == *id)).collect();
    let missing_from_manifest: Vec<_> = gallery_ids.iter().filter(|id| !manifest_ids.contains(&id.as_str())).collect();

    if !missing_from_gallery.is_empty() || !missing_from_manifest.is_empty() {
        eprintln!("aborting: demo-server's panel manifest and demo-app's demo_gallery! have drifted apart");
        if !missing_from_gallery.is_empty() {
            eprintln!(
                "  in demo-server/src/panels.rs's MANIFEST but missing from demo-app's demo_gallery!: {missing_from_gallery:?}"
            );
        }
        if !missing_from_manifest.is_empty() {
            eprintln!(
                "  in demo-app's demo_gallery! but missing from demo-server/src/panels.rs's MANIFEST: {missing_from_manifest:?}"
            );
        }
        process::exit(1);
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Extracts every `"panel-..."` string literal immediately followed by `=>` from `demo_gallery!`'s invocation — the
/// only place that exact shape (`"panel-id" => module::func,`) appears anywhere in `lib.rs`.
fn extract_gallery_panel_ids(lib_rs: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = lib_rs;
    while let Some(start) = rest.find("\"panel-") {
        let after_open_quote = &rest[start + 1..];
        let Some(close) = after_open_quote.find('"') else { break };
        let id = &after_open_quote[..close];
        let after_id = &after_open_quote[close + 1..];
        if after_id.trim_start().starts_with("=>") {
            ids.push(id.to_string());
        }
        rest = after_id;
    }
    ids
}
