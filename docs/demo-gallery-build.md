# Demo Gallery Build Sequence

1. Based on the contents of `.cargo/config.toml`, the command `cargo demo` expands to:

   ```shell
   cargo run -p demo-server
   ```

1. `demo-server`'s `main()` function does the following things:

   1. Check that `demo-app`'s `demo_gallery!` list and its own panel manifest still agree
   with each other.
   1. Assemble the gallery's `index.html` by inserting the individual `demo/panels/*.html` fragments into
     `demo/index.template.html`.
   1. Copies `demo/style.css` and `demo/view-demo.svg` alongside it.
   1. All of this is written to `target/demo-gallery/demo/`

1. Next, `demo-server` spawns `wasm-pack` as a direct child process (via Rust's `Command`) running:

   ```shell
   wasm-pack build demo-app --target web --out-dir <absolute path to target/demo-gallery/pkg>
   ```

   That one command does several things internally:

   - Runs `cargo build --release --target wasm32-unknown-unknown` scoped to the `demo-app` package.
     Because `demo-app/Cargo.toml` declares `svg-dom = { path = ".." }`, Cargo first compiles `svg-dom`, then compiles `demo-app` against it, producing a raw `.wasm` binary.
   - Runs `wasm-bindgen` over that binary to generate the JS/TS glue files (`svg_dom_demo.js`, `svg_dom_demo_bg.wasm` and `.d.ts`).
     These files wrap the generated WASM module such that JavaScript can execute it.
   - Runs `wasm-opt` to shrink/optimize the final `.wasm`.
   - Writes all of it to `--out-dir`, landing at `target/demo-gallery/pkg`, next to the `demo/` directory from step 2.

1. Back in `demo-server`, `main()` starts actix-web's `HttpServer` bound to `127.0.0.1:8080`, serving
   `target/demo-gallery/` — and only that directory, not the project root — as static files.

1. The browser does the rest

