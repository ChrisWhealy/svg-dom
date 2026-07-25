//! Static file server for the `svg-dom` element gallery.
//!
//! Run from the project root with:
//! ```sh
//! cargo demo
//! ```
//! This rebuilds the `svg-dom-demo` crate's wasm package (`wasm-pack build demo-app --target web --out-dir ../pkg`,
//! so the built `pkg/` lands at the project root exactly where `demo/index.html` already expects it, rather than
//! inside `demo-app/` itself) and then serves the project root, so the demo lives at:
//! <http://127.0.0.1:8080/demo/>.
//!
//! The port number can be overridden using the `PORT` environment variable, e.g. `PORT=9000 cargo demo`.

use std::{
    path::{Path, PathBuf},
    process::{self, Command},
};

use actix_files::Files;
use actix_web::{App, HttpServer, middleware::Logger};

const DEFAULT_PORT: u16 = 8080;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // The demo-server crate lives one level below the project root.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| std::io::Error::other("demo-server must live inside the project"))?
        .to_path_buf();

    // Run wasm-pack
    build_wasm(&root);

    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(DEFAULT_PORT);
    let addr = ("127.0.0.1", port);

    println!("\n  svg-dom-demo running on http://127.0.0.1:{port}/demo/\n");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(Files::new("/", root.clone()).index_file("index.html"))
    })
    .bind(addr)?
    .run()
    .await
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Rebuilds the `svg-dom-demo` crate's wasm package so the served `pkg/` is up to date.
/// A failure here is fatal: rather than silently serving a stale `pkg/`, the error is reported and the process exits.
fn build_wasm(root: &Path) {
    let cmd = "wasm-pack build demo-app --target web --out-dir ../pkg";
    println!("Building wasm package: {cmd}");

    // `demo-app` is a separate workspace crate (`svg-dom-demo`) consuming `svg-dom` only through its public API —
    // see that crate's own doc comment for why. `--out-dir ../pkg` is resolved relative to the crate path
    // (`demo-app`), not the current directory, so the built package still lands at the project root's `pkg/`,
    // matching `demo/index.html`'s `import ... from '../pkg/svg_dom_demo.js'`.
    match Command::new("wasm-pack")
        .current_dir(root)
        .args(["build", "demo-app", "--target", "web", "--out-dir", "../pkg"])
        .status()
    {
        Ok(status) if status.success() => {},
        Ok(status) => {
            eprintln!("aborting: wasm-pack exited with {status}");
            process::exit(1);
        },
        Err(err) => {
            eprintln!("aborting: could not run wasm-pack ({err})");
            process::exit(1);
        },
    }
}
