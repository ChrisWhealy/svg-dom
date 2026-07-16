# Change Log

## 0.1.56 — 2026-07-16
- `4a4c889` Implement missing tree navigation functions (2026-07-16)

## 0.1.55 — 2026-07-16
- `db135c4` Implement filter effect colorMatrix (2026-07-15)
- `0a9e1d5` Remove the temporary owned filter ID (2026-07-15)
- `ce339e8` Add typed setters for filter region and coordinat-space attributes (2026-07-15)
- `1f0e84a` Doc Only: Add a performance warning about excessively large filter regions (2026-07-16)
- `9f4411f` Correct stale set_attr documentation and test descriptions (2026-07-16)
- `81a131c` Doc Only: Replace "pixel coordinates" with "user-space coordinates" (2026-07-16)

## 0.1.54 — 2026-07-15

- `061beb8` Cache complete URL reference, not just bare id (2026-07-15)
- `8c1ed4d` Reject idea to implement primitive builder closures (2026-07-15)
- `50ba29c` Implement filter effects flood and composite (2026-07-15)
- `b07bc5e` Implement dropShadow filter effect (2026-07-15)
- `2431df7` Update README (2026-07-15)

## 0.1.53 — 2026-07-14

- `4bc16bb` Start implementation of \<filter\> with Gaussian blur (2026-07-14)
- `f20c907` Implement offset and merge filter effects (2026-07-14)
- `2420868` Add an allocation-light two-axis Gaussian blur factory (2026-07-14)
- `6af69d5` Avoid redundant id revalidation in set_filter_ref (2026-07-14)
- `cab4312` Reuse the cached ID allocation in SvgFilter::set_id (2026-07-14)

## 0.1.52 — 2026-07-14

- `2c38d77` Ensure PathDef setters use a scratch buffer (2026-07-14)
- `4d9dcce` Remove unnecessary whitespace from path strings (2026-07-14)
- `ca0c75f` Add fixed-precision path formatting (2026-07-14)
- `4fda5d4` Pathdef internal writers borrow their commands (2026-07-14)
- `3be798e` PathDef write_d reserves initial capacity (2026-07-14)
- `83cb748` Restrict EllipticalArc::write visibilty (2026-07-14)
- `bec8994` Apply stricter path validation checks (2026-07-14)
- `01f343b` Remove duplicate validation check (2026-07-14)
- `c699e7d` Clamp precision once per path, not once pr command (2026-07-14)
- `bd3c3d1` Make build_d_fixed capacity estimation precision-aware (2026-07-14)
- `2f3b571` Measure PathDef size before considering enum flattening (2026-07-14)

## 0.1.51 — 2026-07-14

- `30a0e20` Implement \<textPath\> (2026-07-14)
- `7809614` Correct doc comment references (2026-07-14)
- `47b99b0` Add PathDef to avoid creating malformed path strings (2026-07-14)

## 0.1.50 — 2026-07-14

- `3cdcbcb` Bump version number (2026-07-14)
- `06827b6` Update design notes docs (2026-07-14)
- `5d75448` Correct documentation related to setTimeout failure (2026-07-14)

## 0.1.49 — 2026-07-13 to 2026-07-14

- `cc973e2` Add pattern tests and bump version no (2026-07-13)
- `a8c96bb` Ensure that AnimationLoop::stop() is genuinely idempotent (2026-07-13)
- `cc1dce0` Correct error in tspan_dy logic for text alignment (2026-07-14)

## 0.1.48 — 2026-07-13

- `64a050b` Implement \<symbol\> (2026-07-13)
- `3518ba1` Update README (2026-07-13)
- `b6f52af` Update docs (2026-07-13)
- `664d451` Update doc comment (2026-07-13)
- `71a832b` Update doc comment (2026-07-13)
- `0088b53` Implement \<pattern\> (2026-07-13)

## 0.1.47 — 2026-07-13

- `69ae2ce` Implement text helper functions (2026-07-13)
- `db728a7` Update README (2026-07-13)
- `97488b6` Implement \<tspan\> (2026-07-13)
- `6746226` Update docs (2026-07-13)
- `6004e53` Use title case in headings (2026-07-13)
- `40bac2e` Refactor the demo module (2026-07-13)

## 0.1.46 — 2026-07-13

- `7a7b015` Implement \<clipPath\> (2026-07-13)

## 0.1.45 — 2026-07-13

- `2dc1689` Add passive event handler variants for high frequency events (2026-07-13)
- `f67e617` Reject the idea that cached typed setters should be provided for small scalar states (2026-07-13)
- `a037a9d` Reject idea to reduce error-path formatting machinery (2026-07-13)
- `e91b6bc` Reject idea to feature-gate event families (2026-07-13)
- `aebc1c7` Refactor event listeners by event type (2026-07-13)
- `c80c08b` Refactor event listeners by event type (2026-07-13)
- `b4cbf2d` Reject idea to add RAF scheduler (2026-07-13)
- `f505d4b` Reject idea to implement delegated event handlers for dense interactive scenes (2026-07-13)
- `de5e188` Refactor functions that add event listeners (2026-07-13)

## 0.1.44 — 2026-07-13

- `856e3a3` Implement linear and radial gradients (2026-07-13)
- `a6b959c` Update README (2026-07-13)
- `1cee478` Correct cargo docs errors (2026-07-13)
- `bd6a5dd` Correct CI step for v2 format (2026-07-13)
- `6b3b4ed` Update gaps.md to describe how non-coalesced, high frequency events can be handled (2026-07-13)
- `09b04f3` Replace CI EmbarkStudios/cargo-deny-action with taiki-e/install-action (2026-07-13)

## 0.1.43 — 2026-06-29 to 2026-07-10

- `62231bb` Implement the \<image\> element (2026-06-29)
- `339c844` Bump actions/upload-artifact from 4 to 7 (2026-07-01)
- `1127bea` Bump actions/checkout from 4 to 7 (2026-07-01)
- `348472d` Merge pull request #3 from ChrisWhealy/dependabot/github_actions/actions/upload-artifact-7 (2026-07-10)
- `3175d49` Merge pull request #4 from ChrisWhealy/dependabot/github_actions/actions/checkout-7 (2026-07-10)

## 0.1.42 — 2026-06-29

- `fe54278` Remove polygon_raw() to keep API consistent (2026-06-29)
- `2ab5bac` Reject the assertion that listener removal has documented unsafe lifecycle caveats (2026-06-29)
- `389e58a` Reject the idea that AnimationLoop::start and start_with_frame are two operating styles (2026-06-29)
- `a01c13f` Refactor node module (2026-06-29)
- `0289262` Implement SVG element \<use\> (2026-06-29)

## 0.1.41 — 2026-06-27 to 2026-06-29

- `775a433` Implement \<defs\> and \<marker\> (2026-06-27)
- `66b4205` Correct doc comments (2026-06-27)
- `b21a633` Reject idea to hide raw web_sys access behind a new function or Cargo feature (2026-06-29)
- `b350d69` Reject the idea that SvgNode::parent() should create a second managed handle for the same DOM node (2026-06-29)
- `dcd94fe` Reject idea to make construction API atomicity consistent (2026-06-29)
- `56419dd` Reject idea to reduce attribute mutation surface (2026-06-29)
- `b1e8f3b` Correct item ordering (2026-06-29)
- `15da556` Reject idea to unify marker refs on handles (2026-06-29)

## 0.1.40 — 2026-06-26

- `782e7b8` Add #[must_use] hint for SvgBatch and AnimationLoop (2026-06-26)
- `1fd81e5` Bump micro version (2026-06-26)
- `d340a9f` Remove remaining expect statement (2026-06-26)
- `a459fa7` Correct documented demo server port (2026-06-26)
- `4fd2e76` Rejected the idea to defer dropping a handler while it is dispatching (2026-06-26)
- `f007cc0` AnimationFrame::stop() sets `stop_requested` to avoid cancelling the running closure (2026-06-26)
- `9b5736c` Reject idea to flatten EventClosure enum (2026-06-26)
- `b046ffa` Improve demo hot path code (2026-06-26)
- `214fd70` Close edge case if an AnimationLoop is dropped from inside its callback (2026-06-26)
- `c91615c` Qualify the scope of the safety statement "never uses innerHTML" (2026-06-26)
- `38714e0` Make error msg formatting consistent (2026-06-26)
- `a5101bb` Improve parsing of SVG width/height on SvgRoot::attach (2026-06-26)
- `87ffecf` AnimationLoop::drop() now owns the deferred clean-up logic (2026-06-26)
- `71d2770` Add helper for one-shot events (2026-06-26)
- `bfd37c6` Remove unchecked cast inSvgNode::on_event_once (2026-06-26)
- `2ab1f52` Update cache immediately after each successful DOM write (2026-06-26)
- `7ad479d` Updatd docs (2026-06-26)
- `eedbd7e` Close rare failure paths (2026-06-26)
- `04b2bf2` Reserve capacity for point list (2026-06-26)
- `63d32e1` Reject idea to add lightweight version of parent() for hot path callers (2026-06-26)

## 0.1.39 — 2026-06-26

- `e2c7da2` Use node.set_text(...) inside create_text (2026-06-26)
- `0962a56` Use build_batch_into() consistently across demos (2026-06-26)
- `7149c6d` Precompute static event labels in demos (2026-06-26)
- `042767f` Use scratch buffer for demo code on the hot path (2026-06-26)
- `46998e7` Reject the idea that CSS-sized viewport dimensions of 0x0 create an inconsistency (2026-06-26)
- `adffd3b` Reject the idea to hide SvgRoot::root (2026-06-26)
- `f30c091` Add functions for explicit event listener removal (2026-06-26)
- `ade7267` Deprecate Point/Size string helpers (2026-06-26)
- `56b87a5` Change colour-wheel demo to use on_pointermove rather than on_mousemove (2026-06-26)
- `bd8c32f` In drag/drop demo, borrow the drag scratch buffer once not twice (2026-06-26)
- `a5a6094` Use CachedAttr in the drag/touch demos repeated readout writes (2026-06-26)
- `cfde0ad` Reject idea to simplify ListenerStore::push (2026-06-26)
- `d32d5b5` Narrow visibility for enums and struct (2026-06-26)
- `8a63f4d` Make fields AttrWrite struct private (2026-06-26)
- `a86e3b6` Add test coverage for deprecated mouseover/mouseout event handlers (2026-06-26)
- `de84fc5` Correct demo server port in README (2026-06-26)

## 0.1.38 — 2026-06-26

- `5ac5648` Add factory parity tests between SvgRoot and SvgBatch (2026-06-26)

## 0.1.37 — 2026-06-25

- `27c0b90` Document another rejected idea (2026-06-25)
- `16ecb56` Ensure demos use allocation-light API (2026-06-25)
- `11d4713` Centralise DOM error conversion (2026-06-25)

## 0.1.36 — 2026-06-25

- `3e0e210` Avoid redundant DOM writes in SvgRoot::set_viewport (2026-06-25)

## 0.1.35 — 2026-06-25

- `962c2a3` Add formatted CachedAttr helpers (2026-06-25)

## 0.1.34 — 2026-06-25

- `e253f0b` Reuse existing SvgAttrs in SvgRoot::create_in() (2026-06-25)

## 0.1.33 — 2026-06-25

- `267446c` Use fixed decimals for points when sub-pixel precision is not needed (2026-06-25)

## 0.1.32 — 2026-06-25

- `102b520` Add AnimationFrame::set_points helper (2026-06-25)

## 0.1.31 — 2026-06-25

- `5ff8e76` Drop use of std::mem::forget() (2026-06-25)

## 0.1.30 — 2026-06-25

- `6c87bf1` Use build_batch_into() in colour wheel demo (2026-06-25)

## 0.1.29 — 2026-06-25

- `664e321` Stop storing a cloned SvgElement in every EventListener (2026-06-25)

## 0.1.28 — 2026-06-25

- `497b83f` Change managed event handlers from Fn to FnMut (2026-06-25)

## 0.1.27 — 2026-06-25

- `353ff90` Avoid two heap allocations when adding first listener (2026-06-25)
- `bf6ec63` Correct demos to remove hot-path allocations (2026-06-25)
- `d0f7d9e` Reject recommendation to add path_fmt and text_fmt factory helpers (2026-06-25)
- `a917bbb` Reject recommendation to add handle-light APIs for large static scenes (2026-06-25)
- `2853608` Reject recommendation to use an EventName enum instead of &'static str (2026-06-25)
- `6535b83` Reject recommendation to reduce WASM pkg size at the crate level (2026-06-25)
- `c29c5f4` Refactor documentation (2026-06-25)
- `528a525` Number rejected idea headings (2026-06-25)
- `4939ac8` Remove redundant statements (2026-06-25)

## 0.1.26 — 2026-06-25

- `4645cc8` Use scratch buffer when formatting points list (2026-06-25)

## 0.1.25 — 2026-06-25

- `f7c47c9` Generalise SvgBatch to commit to any SVG parent (2026-06-25)

## 0.1.24 — 2026-06-25

- `9522450` Introduce WeakSvgNode to prevent Rc cycle forming managed event listeners (2026-06-25)

## 0.1.23 — 2026-06-25

- `621e4e3` Implement parent() tree function (2026-06-25)

## 0.1.22 — 2026-06-25

- `d637246` Bump micro version (2026-06-25)

## 0.1.21 — 2026-06-25

- `daffde7` Add tree functions clear() and replace_with() (2026-06-25)
- `cc287b7` Apply rustfmt edition_style = 2024 (2026-06-25)
- `fc9802e` Add helpers for ellipse, polygon and polyline (2026-06-25)

## 0.1.20 — 2026-06-25

- `d1fb44d` Implement remove & insert_before tree methods (2026-06-25)
- `ec97200` Avoid cargo fmt tripping over trailing whitespace (2026-06-25)
- `0a9b55f` Merge pull request #2 from ChrisWhealy/dependabot/github_actions/actions/checkout-7 (2026-06-25)
- `e0c7eb3` Add rustfmt.toml and apply formatting across codebase (2026-06-25)
- `e082420` Merge branch 'main' of github.com:ChrisWhealy/svg-dom (2026-06-25)

## 0.1.19 — 2026-06-25

- `50cd296` Bump micro version (2026-06-25)

## 0.1.18 — 2026-06-24 to 2026-06-25

- `fccf9c1` Bump micro version (2026-06-24)
- `e80d858` Schedule cargo-deny to run as a weekly cron job (2026-06-25)
- `8459d07` Schedule cargo-deny to run as a weekly cron job (2026-06-25)
- `d4d04d4` Update README (2026-06-25)
- `2c53702` Adjust minimal demo to not use std::mem:forget() (2026-06-25)
- `656b722` Update README (2026-06-25)
- `421b63a` Update README (2026-06-25)
- `40e31d2` Update README (2026-06-25)
- `d1fb865` Update README (2026-06-25)

## 0.1.17 — 2026-06-24

- `21ed1fe` Bump micro version (2026-06-24)
- `1b69a8b` Bump actions/checkout from 4 to 7 (2026-06-24)
- `aeefee0` Add security disclosure policy (2026-06-24)

## 0.1.16 — 2026-06-24

- `823df3d` Bump micro version (2026-06-24)
- `c540b0a` Add code coverage measurement only for host-runnable tests (2026-06-24)

## 0.1.15 — 2026-06-24

- `694e493` Bump micro version (2026-06-24)
- `9f3b2bc` Remove runtime panic points (2026-06-24)

## 0.1.14 — 2026-06-24

- `6be8cb8` Bump micro version (2026-06-24)
- `c8d3486` Reformat gaps.md (2026-06-24)
- `758c35f` Add dependabot and deny YAML files (2026-06-24)
- `36c802d` Add dependabot and deny YAML files (2026-06-24)

## 0.1.13 — 2026-06-24

- `41289e3` Bump micro version (2026-06-24)
- `3a3b414` Add missing docs (2026-06-24)

## 0.1.12 — 2026-06-24

- `7ad5f3e` Bump micro version (2026-06-24)
- `29ec7f1` Run clippy on src tree. Add CI workflow. Remove panic sites (2026-06-24)
- `45aba30` Fix failing headless firefox test using TouchEvent (2026-06-24)
- `8802abc` Account for older CI version of wasm-pack (2026-06-24)

## 0.1.11 — 2026-06-24

- `bc284ad` Bump micro version (2026-06-24)
- `8ec15a0` Update README (2026-06-24)

## 0.1.10 — 2026-06-24

- `88e7c8f` Bump micro version number (2026-06-24)
- `3dc3ea0` Add code samples to demos (2026-06-24)

## 0.1.9 — 2026-06-24

- `7ee96e4` Remove unnecessary String allocations inside text and numeric value setters (2026-06-24)
- `03b900a` Avoid unnecessary string allocation in text setters for event handlers (2026-06-24)

## 0.1.8 — 2026-06-24

- `63b38d4` Use CachedAttr to avoid unnecessary String allocation in setters called in hot path (2026-06-24)
- `b028fb6` Update design_notes.md (2026-06-24)

## 0.1.7 — 2026-06-24

- `f66a863` Bump micro version number (2026-06-24)
- `af6e4e6` Update design notes (2026-06-24)
- `0e849f3` Update design notes (2026-06-24)
- `89d57cf` Update design notes (2026-06-24)

## 0.1.6 — 2026-06-24

- `7c11820` Avoid changing unchanged node attributes (2026-06-24)
- `ede42c1` Avoid unnecessary String allocation in hot-path coding (2026-06-24)

## 0.1.5 — 2026-06-24

- `e37fc12` Add managed wrappers for more event types (2026-06-24)

## 0.1.4 — 2026-06-24

- `585ccae` Store event listener types as static str slices (2026-06-24)

## 0.1.3 — 2026-06-24

- `8b5ef48` Move common functionality from SvgRoot & SvgBatch into factory trait (2026-06-24)

## 0.1.2 — 2026-06-24

- `5cac68b` Add reusable SvgAttrs/AttrWriter to avoid repeated formatting inside callbacks (2026-06-24)

## 0.1.1 — 2026-06-24

- `d40b568` Reusable scratch buffer for numeric formatting (2026-06-24)

## 0.1.0 — 2026-06-10 to 2026-06-23

- `a6f097f` Initial commit (2026-06-10)
- `04292c1` Update README (2026-06-10)
- `74b0e4f` Correct doc comment formatting (2026-06-10)
- `cecb478` Unit tests for error handling (2026-06-10)
- `3735f68` Run rustfmt (2026-06-10)
- `673e0c7` Add headless browser tests for SVG node functionality (2026-06-10)
- `a22aeb8` Update README (2026-06-10)
- `5857f7e` Update README (2026-06-10)
- `e269755` Refactor documentation (2026-06-10)
- `4101503` Update README (2026-06-10)
- `80d9454` Update README (2026-06-10)
- `8980326` Update gaps.md (2026-06-10)
- `6e5a67c` Update gaps.md (2026-06-10)
- `f42287c` Update gaps.md (2026-06-10)
- `308907a` Update README (2026-06-10)
- `9caa8f6` Add basic RAF tests (2026-06-10)
- `81066a5` Update README (2026-06-11)
- `e69194f` Add basic demo server (2026-06-11)
- `64d7df9` Update README (2026-06-11)
- `9bf3d49` Remove unnecessary clone (2026-06-11)
- `6685022` Add event listener demos (2026-06-11)
- `6da657d` Update event description (2026-06-11)
- `5cf2fda` Update docs (2026-06-15)
- `d106fc7` Add draggable object demo (2026-06-15)
- `04a6fe2` Adjust demo screen layout and fix mouse press demo (2026-06-15)
- `fa0a8bb` Improve demo layout (2026-06-15)
- `ff14cb4` Rename unit tests (2026-06-15)
- `e729a38` Refactor SvgRoot node (2026-06-15)
- `8bf0c5b` Add Point and Size structs (2026-06-16)
- `d30cd32` Abort running demo server if wasm-pack fails (2026-06-16)
- `fbd19b0` Add derive traits for Size and Point (2026-06-16)
- `4e1d644` Correct integration tests (2026-06-16)
- `2520b46` Correct integration tests - again (2026-06-16)
- `fd479da` Use lazy MouseClosure (2026-06-23)
- `2dea752` Create detached element, set attributes, then append to DOM (2026-06-23)
- `bd9be57` Allow elements to be added in batches (2026-06-23)
- `ea8ffbd` Avoid allocating storage for EventListener the node does not have one (2026-06-23)
- `e1610a5` Drop event listener from DOM when dropping from memory (2026-06-23)
- `87c1563` Deprecate use of mouseover mouseout events (2026-06-23)
- `b816df9` Reduce per-frame string allocation (2026-06-23)
- `2775db9` Implement multi-attribute setter for node (2026-06-23)
- `7b027d1` Cache viewport size in SVGRoot (2026-06-23)
- `694df3c` Update docs (2026-06-23)
- `be0a9ff` Update README (2026-06-23)
- `0025b02` Update README (2026-06-23)
- `8787881` Update README (2026-06-23)
- `a350780` Update README (2026-06-23)
- `1517fbe` Update README (2026-06-23)
