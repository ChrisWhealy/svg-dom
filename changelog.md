# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project's crate version follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# [Unreleased]

# [Released]

## [0.1.73] - 2026-07-24

### Added

- Implement the filter effect `feConvolveMatrix` (`4aca9a2`)

### Fixed

- Exclude convolve matrix order value of zero (`d5203df`)
- Doc only: warn that cost of large feConvolveMatrix rises with the square of the order (`0c63761`)
- Doc only: kernelUnitLength is deprecated and unreliable for platform-independent rendering (`62bebae`)
- Fix escape-hatch test comment and add a zero-divisor serialization test for feConvolveMatrix (`10aa53e`)

## [0.1.72] - 2026-07-23

### Added

- Implement the SVG element `<foreignObject>` (`566c5b4`)
- Add tests for foreign content rendering (`3a3919d`)
- Doc only: improve error description (`7e3132c`)

### Fixed

- Doc only: <foreignObject> does not always apply clipping (`c13ff6d`)
- Doc only: clarify innerHTML safety guarantee (`650a324`)
- Doc only: clarify requiredExtensions attribute of <foreignObject> (`6046f51`)
- Doc only: tree-navigation opacity depends on which method is being called (`d2343f3`)
- Doc only: qualify the Document::createElement() namespace comment (`79a6de0`)

## [0.1.71] - 2026-07-23

### Added

- Implement the SVG element `<metadata>` (`522459c`)
- Doc only: <metadata> content permitted by this crate is narrower than that permitted by SVG 2 (`2677cec`)
- Doc only: qualify that <metadata> inside <defs> is permitted, but not conventional (`ff586d5`)
- Test successful serialisation of Unicode chars (`ef86ef8`)

### Fixed

- Doc only: <metadata> content is accessible only through the DOM API or after serialisation (`e45855e`)
- Doc only: relax statement about RDF/XML <matadata> content (`c79840f`)
- Doc only: <metadata> content can be manipulated through the crate's API (`b23dff2`)
- Doc only: clarify supported content for <metadata> (`d3cc142`)

## [0.1.70] - 2026-07-23

### Added 

- Implement the SVG elements `<view>` and `<style>` (`1ae69d6`)
- Test fragment-navigation behaviour (`3336bd6`)
- Doc only: id validation is intentionally narrower than that allowed by SVG's grammar (`28933f1`)
- Add tests that pipeline API functionality into browser behaviour (`5db8a8a`)

### Fixed

- Doc only: <view> is not strictly childless (`f2c108b`)
- Doc only: update missing docs for supported SVG elements (`48d6919`)
- Doc only: use consistent <view> navigation description (`d982c6a`)
- Fix race and listener lifetime problems in async view test (`f05cd0f`)
- Doc only: correct outdated statement about `<view>` child content (`957155f`)
- Remove error/load event listeners in both success and failure cases (`5a420a1`)

## [0.1.69] - 2026-07-22

### Added 

- Implement the SVG elements <a> and <switch> (`66bc9f2`)
- Doc only: Qualify anchor tag analogy with <g> (`fe7c729`)
- Add tests for <a> and <switch> elements (`6bbdbe2`)

### Fixed

- Fix CI error caused by AppArmor's new default that blocks access to browser sandbox (`4797789`)
- Doc only: correct description of `<switch>` rendering (`88fac96`)
- Correct switch condition in demo and doc comments (`52357db`)
- Doc only: explain that `<switch>` `requiredFeatures` has been deprecated in SVG 2 (`1ab61c1`)
- Doc only: correct documentation accuracy (`ebded47`)

## [0.1.68] - 2026-07-22

### Added

- Implement the filter effect feTile (`419e55d`)

### Fixed

- Doc only: not all primitives accept the "in" parameter (`48e1ee6`)
- Doc only: correct description of feTile's default subregion (`4ee684b`)
- Doc only: correct test comment (`5f9bd59`)
- Doc only: immediate input does not always need explicit narrowing (`a2d0f93`)

## [0.1.67] - 2026-07-22

### Added

- Implement the filter effect feImage (`2c25218`)
- Add extra test for CORS check and update docs (`3787f30`)
- Add extra test to cover feImage href pointing to an internal reference (`f7a6218`)

### Changed

- Doc only: clarify the circumstances under which feImage returns an Error (`4b0bb13`)
- Refactor src/root/utils by type + impl (`306c0d9`)
- Doc only: correct feImage element-scaling description (`93b40f5`)

### Fixed

- Correct distinction between \<image\> and feImage (`fd4aaa2`)
- Doc only: correct explanation of feImage result attribute (`0abd111`)
- Correct description of taint (`a9438df`)
- Minor doc corrections (`2d5cc21`)

## [0.1.66] - 2026-07-21

### Added

- Implement filter effects feTurbulence and feDisplacementMap (`27bbc33`)
- Doc only: add description fractional seed truncation behaviour (`9805315`)
- Doc warning: testing cannot acounnt for turbulence effect looking different in different browsers (`4a107ff`)
- Doc only: clarify that user agents might clamp the numOctaves value (`8cc123a`)
- Implement the filter effect feMorphology (`fba2622`)
- Doc only: warn about filter-region clipping (`d173ea5`)
- Doc only: update stale docs (`84eeddc`)

### Changed

- Refactor categorisation of demo examples (`677c390`)
- Do not use SVG default of alpha/alpha channel selector in displacement_map() (`0c317ae`)
- Doc only: adjust description of displacement_map() channel behaviour (`0774bc1`)
- Adjust statement about wasm-bindgen testing (`9017703`)

### Fixed

- Doc only: correct description of negative numOctave values (`6efb89e`)
- Doc only: correct description of turbulence (`319d805`)
- Doc only: correct description of zero-axis behaviour in morphology_xy() (`2df8442`)
- Doc only: correct description of negative radius behaviour (`623175e`)
- Doc only: correct overgeneralised "opaque regions" claims (`405693b`)
- Doc only: correct statement about erosion followed dilation (`9ec91a8`)
- Doc only: correct description how erode and dilation work (`db18d24`)
- Doc only: correct ObjectBoundingBox wording ambiguity (`c0d7f20`)

## [0.1.65] - 2026-07-21

### Added

- Implement filter effect feComponentTransfer (`3d0c95a`)
- Add doc warning and test about last-channel-function wins (`f06b146`)

### Changed

- Refactor wasm-pack filter tests (`49729b6`)

### Fixed

- Exclude TransferFunction table with only one entry (`ccdc15f`)
- Correct docs; alpha fn can create pixels outside original opaque bounding box (`fce1c99`)
- Reject empty discrete transfer function (`3a3c90e`)
- Correct Channel::Alpha rustdoc safety advice (`9917cfd`)

## [0.1.64] - 2026-07-21

### Added

- Add accessibility forwarding methods on SvgRoot (`f665b75`)
- Add CI accessibility-tree-test and an extra accessibility test (`38ff774`)
- Implement filter effect feBlend (`5e6fb6f`)
- Extend a11y tests to check rendered result, not just DOM structure (`a2d01d9`)

### Changed

- Split a11y tests into five seperate functions (`41b5ad4`)
- Clarify difference between feBlend and CSS mix-blend-mode (`5ced25e`)
- Refactor defs and svgNode tests (`766cc0e`)

### Fixed

- Correct rustdoc about multilingual title selection (`7a709ea`)
- Correct comment error (`b4beede`)
- Correct comment typos (`201a0a5`)
- Add missing composite step to source transparency in feBlend demo tinting chain (`a622bef`)

## [0.1.63] - 2026-07-21

### Added

- Add SvgMask to path factory table (`8a5f6d5`)
- Add \<textPath\> as valid parent for \<tspan\> (`4131fb5`)
- Add explanation that viewBox disables the effect of patternContentUnits (`a76ddc1`)
- Describe Marker applicability in SVG 1.1 and 2 (`2f14fc4`)
- Implement the \<desc\> and \<title\> accessibility elements (`02204cd`)
- Enforce title/desc singularity on pre-existing DOM elements (`440dbdf`)
- Caution against setting titles and descriptions where they are not needed (`dfdceac`)
- Add a11y tests (`3b6ebcd`)

### Changed

- Bump version number and remove Cargo.lock from version management (`a69e1f6`)
- Clarify that \<use\> is not restricted to local fragments (`452ad50`)

### Fixed

- Correct alpha-mask wording (`c17addc`)
- Filter primitive values are not always user units (`1411b3d`)
- Correct various typos (`833f8b1`)
- Reject empty or whitespace-only values (`26ac6d6`)
- Distinguish values set by set_title from ARIA values in accessible name calculation (`3ba62eb`)
- Adjust title/desc demo so it does not imply button behaviour (`d51e114`)


## [0.1.62] - 2026-07-20

### Added

- Add warning about leaving Arithmetic coefficients unspecified (`4795f71`)
- Per-instance styling of \<use\> needs an inheritence qualification (`92c067f`)

### Changed

- Update README and reformat changelog (`a992932`)
- Bump version number (`f17a2fe`)
- Refactor SVG element documentation (`f1fef65`)
- Remove SvgClipPath::group() and correct documentation (`74f9bfb`)
- Remove conflation of create constraint as an SVG 2 constraint (`5e5ae05`)

### Fixed

- Correct multiline text guidance (`8aa6559`)
- Keep clippy happy (`4859966`)
- Correct clip_path documentation (`4144a89`)
- Set mask-mode as CSS atribute, not ordinary SVG attribute (`01b9a49`)
- Correct \<textPath\> specification errors (`342155a`)
- Correct radial-gradient fr explanation (`5196680`)
- Correct various typos (`b14fb4`)

## [0.1.61] - 2026-07-17

### Added

- Implement the \<mask\> element (`d210e0e`)
- Add alpha multiplier to luminence mask docs (`9cea53b`)
- Add end-to-end cached-reference rename test (`bb9af7d`)

### Changed

- Refactor rejected ideas document (`7392be2`)
- Update stale geometry documentation (`065c9b8`)
- Update gap analysis document to cover previously untracked features (`c2972f1`)
- Update gaps.md (`06c71d9`)
- Improve line() documentation (`6222cea`)
- Warn that excessively large mask regions have a rendering cost (`60c4c58`)
- Update repository level summary docs (`5e01a41`)

### Fixed

- Correct interface-failure reachability claim (`4047145`)
- Correct positive path-distance endpoint docs (`fa83536`)
- Correct polyline docs (`46080e7`)
- Correct gradient example (`a1ea2f9`)

## [0.1.60] - 2026-07-17

### Added

- Add 4 geometry read-back helpers (`55b973f`)
  - Add f32-precision explanation to public methods for docs.rs completeness (`03eff35`)
- Add #!\[deny(rustdoc::broken_intra_doc_link\]) to catch link mistakes (`ffa2825`)

### Changed

- Document bounding_box() inclusion semantics (`745ed91`)
- Public f64s may carry only f32 precision (`db78c2b`)
- Refactor design notes documentation (`2df0553`)
- Use f64 -> f32 saturation instead of casting (`45a1441`)
- Improve test coverage (`ef4bf9a`)

### Fixed

- Correct ctm() write-back round trip docs (`ff1736b`)
- Correct screen_ctm() coordinate-conversion docs (`8a2a91f`)
- Correct layout/reflow statements (`124fa5f`)

## [0.1.59] - 2026-07-16

### Added

- Implement 4 CSS class helpers on SvgNode (`fddc1cd`)
- Add dependency and bump version number (`e626f1e`)
- Implement set_class_enabled wrapper around websys::toggle_with_force (`951b82c`)

### Changed

- Document CSS class errors and broaden tests (`151d73d`)
- Explain why a test ignores the result of a fallible function (`4f97681`)
- Clean up stale doc comments (`066c640`)

## [0.1.58] - 2026-07-16

### Added

- Implement SvgRoot::set_view_box (`726bb4a`)
- Implement SvgMarker::set_view_box() (`bbdc233`)

### Changed

- Extend tests to cover input domain (`7c2fecb`)
- Minor doc updates (`d94ab77`)
- Minor doc fix (`10f08db`)

## [0.1.57] - 2026-07-16

### Added

- Implement 2D affine matrix transform helper (`18e7100`)
- Implement set_matrix_precise to avoid possible artefacts created by fixed-precision rounding (`d2f64a0`)

### Changed

- Strengthen the argument-order tests (`e7b7301`)
- Use deterministic values in precise rotation test (`3020ea3`)
- Improve docs concerning the rounding safety of other typed setters (`98d3ec0`)

### Fixed

- Correct stale documentation (`9ae53c7`)
- Correct fixed-output performance guide (`008ab81`)

## [0.1.56] - 2026-07-16

### Added

- Implement missing tree navigation functions (`4a4c889`)
- Add tests for the documented non-SVG behaviour (`7813478`)

### Changed

- Preallocate the collection result vectors for tree navigation (`a2847ef`)

## [0.1.55] - 2026-07-16

### Added

- Implement filter effect colorMatrix (`db135c4`)
- Add typed setters for filter region and coordinat-space attributes (`ce339e8`)

### Changed

- Doc Only: Add a performance warning about excessively large filter regions (`1f0e84a`)
- Doc Only: Replace "pixel coordinates" with "user-space coordinates" (`81a131c`)

### Fixed

- Correct stale set_attr documentation and test descriptions (`9f4411f`)

### Removed

- Remove the temporary owned filter ID (`0a9e1d5`)

## [0.1.54] - 2026-07-15

### Added

- Implement filter effects flood and composite (`50ba29c`)
- Implement dropShadow filter effect (`b07bc5e`)

### Changed

- Cache complete URL reference, not just bare id (`061beb8`)
- Reject idea to implement primitive builder closures (`8c1ed4d`)
- Update README (`2431df7`)

## [0.1.53] - 2026-07-14

### Added

- Start implementation of \<filter\> with Gaussian blur (`4bc16bb`)
- Implement offset and merge filter effects (`f20c907`)
- Add an allocation-light two-axis Gaussian blur factory (`2420868`)

### Changed

- Avoid redundant id revalidation in set_filter_ref (`6af69d5`)
- Reuse the cached ID allocation in SvgFilter::set_id (`cab4312`)

## [0.1.52] - 2026-07-14

### Added

- Add fixed-precision path formatting (`ca0c75f`)

### Changed

- Ensure PathDef setters use a scratch buffer (`2c38d77`)
- Pathdef internal writers borrow their commands (`4fda5d4`)
- PathDef write_d reserves initial capacity (`3be798e`)
- Restrict EllipticalArc::write visibilty (`83cb748`)
- Apply stricter path validation checks (`bec8994`)
- Clamp precision once per path, not once pr command (`c699e7d`)
- Make build_d_fixed capacity estimation precision-aware (`bd3c3d1`)
- Measure PathDef size before considering enum flattening (`2f3b571`)

### Removed

- Remove unnecessary whitespace from path strings (`4d9dcce`)
- Remove duplicate validation check (`01f343b`)

## [0.1.51] - 2026-07-14

### Added

- Implement \<textPath\> (`30a0e20`)
- Add PathDef to avoid creating malformed path strings (`47b99b0`)

### Fixed

- Correct doc comment references (`7809614`)

## [0.1.50] - 2026-07-14

### Changed

- Bump version number (`3cdcbcb`)
- Update design notes docs (`06827b6`)

### Fixed

- Correct documentation related to setTimeout failure (`5d75448`)

## [0.1.49] - 2026-07-14

### Added

- Add pattern tests and bump version no (`cc973e2`)

### Changed

- Ensure that AnimationLoop::stop() is genuinely idempotent (`a8c96bb`)

### Fixed

- Correct error in tspan_dy logic for text alignment (`cc1dce0`)

## [0.1.48] - 2026-07-13

### Added

- Implement \<symbol\> (`64a050b`)
- Implement \<pattern\> (`0088b53`)

### Changed

- Update README (`3518ba1`)
- Update docs (`b6f52af`)
- Update doc comment (`664d451`)
- Update doc comment (`71a832b`)

## [0.1.47] - 2026-07-13

### Added

- Implement text helper functions (`69ae2ce`)
- Implement \<tspan\> (`97488b6`)

### Changed

- Update README (`db728a7`)
- Update docs (`6746226`)
- Use title case in headings (`6004e53`)
- Refactor the demo module (`40bac2e`)

## [0.1.46] - 2026-07-13

### Added

- Implement \<clipPath\> (`7a7b015`)

## [0.1.45] - 2026-07-13

### Added

- Add passive event handler variants for high frequency events (`2dc1689`)

### Changed

- Reject the idea that cached typed setters should be provided for small scalar states (`f67e617`)
- Reject idea to reduce error-path formatting machinery (`a037a9d`)
- Reject idea to feature-gate event families (`e91b6bc`)
- Refactor event listeners by event type (`aebc1c7`)
- Refactor event listeners by event type (`c80c08b`)
- Reject idea to add RAF scheduler (`b4cbf2d`)
- Reject idea to implement delegated event handlers for dense interactive scenes (`f505d4b`)
- Refactor functions that add event listeners (`de5e188`)

## [0.1.44] - 2026-07-13

### Added

- Implement linear and radial gradients (`856e3a3`)

### Changed

- Update README (`a6b959c`)
- Update gaps.md to describe how non-coalesced, high frequency events can be handled (`6b3b4ed`)
- Replace CI EmbarkStudios/cargo-deny-action with taiki-e/install-action (`09b04f3`)

### Fixed

- Correct cargo docs errors (`1cee478`)
- Correct CI step for v2 format (`bd6a5dd`)

## [0.1.43] - 2026-07-10

### Added

- Implement the \<image\> element (`62231bb`)

### Changed

- Bump actions/upload-artifact from 4 to 7 (`339c844`)
- Bump actions/checkout from 4 to 7 (`1127bea`)
- Merge pull request #3 from ChrisWhealy/dependabot/github_actions/actions/upload-artifact-7 (`348472d`)
- Merge pull request #4 from ChrisWhealy/dependabot/github_actions/actions/checkout-7 (`3175d49`)

## [0.1.42] - 2026-06-29

### Added

- Implement SVG element \<use\> (`0289262`)

### Changed

- Reject the assertion that listener removal has documented unsafe lifecycle caveats (`2ab5bac`)
- Reject the idea that AnimationLoop::start and start_with_frame are two operating styles (`389e58a`)
- Refactor node module (`a01c13f`)

### Removed

- Remove polygon_raw() to keep API consistent (`fe54278`)

## [0.1.41] - 2026-06-29

### Added

- Implement \<defs\> and \<marker\> (`775a433`)

### Changed

- Reject idea to hide raw web_sys access behind a new function or Cargo feature (`b21a633`)
- Reject the idea that SvgNode::parent() should create a second managed handle for the same DOM node (`b350d69`)
- Reject idea to make construction API atomicity consistent (`dcd94fe`)
- Reject idea to reduce attribute mutation surface (`56419dd`)
- Reject idea to unify marker refs on handles (`15da556`)

### Fixed

- Correct doc comments (`66b4205`)
- Correct item ordering (`b1e8f3b`)

## [0.1.40] - 2026-06-26

### Added

- Add #[must_use] hint for SvgBatch and AnimationLoop (`782e7b8`)
- Add helper for one-shot events (`71d2770`)

### Changed

- Bump micro version (`1fd81e5`)
- Rejected the idea to defer dropping a handler while it is dispatching (`4fd2e76`)
- AnimationFrame::stop() sets `stop_requested` to avoid cancelling the running closure (`f007cc0`)
- Reject idea to flatten EventClosure enum (`9b5736c`)
- Improve demo hot path code (`b046ffa`)
- Qualify the scope of the safety statement "never uses innerHTML" (`c91615c`)
- Make error msg formatting consistent (`38714e0`)
- Improve parsing of SVG width/height on SvgRoot::attach (`a5101bb`)
- AnimationLoop::drop() now owns the deferred clean-up logic (`87ffecf`)
- Update cache immediately after each successful DOM write (`2ab1f52`)
- Updatd docs (`7ad479d`)
- Reserve capacity for point list (`04b2bf2`)
- Reject idea to add lightweight version of parent() for hot path callers (`63d32e1`)

### Fixed

- Correct documented demo server port (`a459fa7`)
- Close edge case if an AnimationLoop is dropped from inside its callback (`214fd70`)
- Close rare failure paths (`eedbd7e`)

### Removed

- Remove remaining expect statement (`d340a9f`)
- Remove unchecked cast inSvgNode::on_event_once (`bfd37c6`)

## [0.1.39] - 2026-06-26

### Added

- Add functions for explicit event listener removal (`f30c091`)
- Add test coverage for deprecated mouseover/mouseout event handlers (`a86e3b6`)

### Changed

- Use node.set_text(...) inside create_text (`e2c7da2`)
- Use build_batch_into() consistently across demos (`0962a56`)
- Precompute static event labels in demos (`7149c6d`)
- Use scratch buffer for demo code on the hot path (`042767f`)
- Reject the idea that CSS-sized viewport dimensions of 0x0 create an inconsistency (`46998e7`)
- Reject the idea to hide SvgRoot::root (`adffd3b`)
- Change colour-wheel demo to use on_pointermove rather than on_mousemove (`56b87a5`)
- In drag/drop demo, borrow the drag scratch buffer once not twice (`bd8c32f`)
- Use CachedAttr in the drag/touch demos repeated readout writes (`a5a6094`)
- Reject idea to simplify ListenerStore::push (`cfde0ad`)
- Narrow visibility for enums and struct (`d32d5b5`)
- Make fields AttrWrite struct private (`8a63f4d`)

### Deprecated

- Deprecate Point/Size string helpers (`ade7267`)

### Fixed

- Correct demo server port in README (`de84fc5`)

## [0.1.38] - 2026-06-26

### Added

- Add factory parity tests between SvgRoot and SvgBatch (`5ac5648`)

## [0.1.37] - 2026-06-25

### Changed

- Document another rejected idea (`27c0b90`)
- Ensure demos use allocation-light API (`16ecb56`)
- Centralise DOM error conversion (`11d4713`)

## [0.1.36] - 2026-06-25

### Changed

- Avoid redundant DOM writes in SvgRoot::set_viewport (`3e0e210`)

## [0.1.35] - 2026-06-25

### Added

- Add formatted CachedAttr helpers (`962c2a3`)

## [0.1.34] - 2026-06-25

### Changed

- Reuse existing SvgAttrs in SvgRoot::create_in() (`e253f0b`)

## [0.1.33] - 2026-06-25

### Changed

- Use fixed decimals for points when sub-pixel precision is not needed (`267446c`)

## [0.1.32] - 2026-06-25

### Added

- Add AnimationFrame::set_points helper (`102b520`)

## [0.1.31] - 2026-06-25

### Changed

- Drop use of std::mem::forget() (`5ff8e76`)

## [0.1.30] - 2026-06-25

### Changed

- Use build_batch_into() in colour wheel demo (`6c87bf1`)

## [0.1.29] - 2026-06-25

### Changed

- Stop storing a cloned SvgElement in every EventListener (`664e321`)

## [0.1.28] - 2026-06-25

### Changed

- Change managed event handlers from Fn to FnMut (`497b83f`)

## [0.1.27] - 2026-06-25

### Changed

- Avoid two heap allocations when adding first listener (`353ff90`)
- Reject recommendation to add path_fmt and text_fmt factory helpers (`d0f7d9e`)
- Reject recommendation to add handle-light APIs for large static scenes (`a917bbb`)
- Reject recommendation to use an EventName enum instead of &'static str (`2853608`)
- Reject recommendation to reduce WASM pkg size at the crate level (`6535b83`)
- Refactor documentation (`c29c5f4`)
- Number rejected idea headings (`528a525`)

### Fixed

- Correct demos to remove hot-path allocations (`bf6ec63`)

### Removed

- Remove redundant statements (`4939ac8`)

## [0.1.26] - 2026-06-25

### Changed

- Use scratch buffer when formatting points list (`4645cc8`)

## [0.1.25] - 2026-06-25

### Changed

- Generalise SvgBatch to commit to any SVG parent (`f7c47c9`)

## [0.1.24] - 2026-06-25

### Added

- Introduce WeakSvgNode to prevent Rc cycle forming managed event listeners (`9522450`)

## [0.1.23] - 2026-06-25

### Added

- Implement parent() tree function (`621e4e3`)

## [0.1.22] - 2026-06-25

### Changed

- Bump micro version (`d637246`)

## [0.1.21] - 2026-06-25

### Added

- Add tree functions clear() and replace_with() (`daffde7`)
- Add helpers for ellipse, polygon and polyline (`fc9802e`)

### Changed

- Apply rustfmt edition_style = 2024 (`cc287b7`)

## [0.1.20] - 2026-06-25

### Added

- Implement remove & insert_before tree methods (`d1fb44d`)
- Add rustfmt.toml and apply formatting across codebase (`e0c7eb3`)

### Changed

- Avoid cargo fmt tripping over trailing whitespace (`ec97200`)
- Merge pull request #2 from ChrisWhealy/dependabot/github_actions/actions/checkout-7 (`0a9b55f`)
- Merge branch 'main' of github.com:ChrisWhealy/svg-dom (`e082420`)

## [0.1.19] - 2026-06-25

### Changed

- Bump micro version (`50cd296`)

## [0.1.18] - 2026-06-25

### Changed

- Bump micro version (`fccf9c1`)
- Schedule cargo-deny to run as a weekly cron job (`e80d858`)
- Schedule cargo-deny to run as a weekly cron job (`8459d07`)
- Update README (`d4d04d4`)
- Adjust minimal demo to not use std::mem:forget() (`2c53702`)
- Update README (`656b722`)
- Update README (`421b63a`)
- Update README (`40e31d2`)
- Update README (`d1fb865`)

## [0.1.17] - 2026-06-24

### Added

- Add security disclosure policy (`aeefee0`)

### Changed

- Bump micro version (`21ed1fe`)
- Bump actions/checkout from 4 to 7 (`1b69a8b`)

## [0.1.16] - 2026-06-24

### Added

- Add code coverage measurement only for host-runnable tests (`c540b0a`)

### Changed

- Bump micro version (`823df3d`)

## [0.1.15] - 2026-06-24

### Changed

- Bump micro version (`694e493`)

### Removed

- Remove runtime panic points (`9f3b2bc`)

## [0.1.14] - 2026-06-24

### Added

- Add dependabot and deny YAML files (`758c35f`)
- Add dependabot and deny YAML files (`36c802d`)

### Changed

- Bump micro version (`6be8cb8`)
- Reformat gaps.md (`c8d3486`)

## [0.1.13] - 2026-06-24

### Added

- Add missing docs (`3a3b414`)

### Changed

- Bump micro version (`41289e3`)

## [0.1.12] - 2026-06-24

### Changed

- Bump micro version (`7ad5f3e`)
- Run clippy on src tree. Add CI workflow. Remove panic sites (`29ec7f1`)

### Fixed

- Fix failing headless firefox test using TouchEvent (`45aba30`)
- Account for older CI version of wasm-pack (`8802abc`)

## [0.1.11] - 2026-06-24

### Changed

- Bump micro version (`bc284ad`)
- Update README (`8ec15a0`)

## [0.1.10] - 2026-06-24

### Added

- Add code samples to demos (`3dc3ea0`)

### Changed

- Bump micro version number (`88e7c8f`)

## [0.1.9] - 2026-06-24

### Changed

- Avoid unnecessary string allocation in text setters for event handlers (`03b900a`)

### Removed

- Remove unnecessary String allocations inside text and numeric value setters (`7ee96e4`)

## [0.1.8] - 2026-06-24

### Changed

- Use CachedAttr to avoid unnecessary String allocation in setters called in hot path (`63b38d4`)
- Update design_notes.md (`b028fb6`)

## [0.1.7] - 2026-06-24

### Changed

- Bump micro version number (`f66a863`)
- Update design notes (`af6e4e6`)
- Update design notes (`0e849f3`)
- Update design notes (`89d57cf`)

## [0.1.6] - 2026-06-24

### Changed

- Avoid changing unchanged node attributes (`7c11820`)
- Avoid unnecessary String allocation in hot-path coding (`ede42c1`)

## [0.1.5] - 2026-06-24

### Added

- Add managed wrappers for more event types (`e37fc12`)

## [0.1.4] - 2026-06-24

### Changed

- Store event listener types as static str slices (`585ccae`)

## [0.1.3] - 2026-06-24

### Changed

- Move common functionality from SvgRoot & SvgBatch into factory trait (`8b5ef48`)

## [0.1.2] - 2026-06-24

### Added

- Add reusable SvgAttrs/AttrWriter to avoid repeated formatting inside callbacks (`5cac68b`)

## [0.1.1] - 2026-06-24

### Changed

- Reusable scratch buffer for numeric formatting (`d40b568`)

## 0.1.0 - 2026-06-23

### Added

- Add headless browser tests for SVG node functionality (`673e0c7`)
- Add basic RAF tests (`9caa8f6`)
- Add basic demo server (`e69194f`)
- Add event listener demos (`6685022`)
- Add draggable object demo (`d106fc7`)
- Add Point and Size structs (`8bf0c5b`)
- Add derive traits for Size and Point (`fbd19b0`)
- Implement multi-attribute setter for node (`2775db9`)

### Changed

- Initial commit (`a6f097f`)
- Update README (`04292c1`)
- Unit tests for error handling (`cecb478`)
- Run rustfmt (`3735f68`)
- Update README (`a22aeb8`)
- Update README (`5857f7e`)
- Refactor documentation (`e269755`)
- Update README (`4101503`)
- Update README (`80d9454`)
- Update gaps.md (`8980326`)
- Update gaps.md (`6e5a67c`)
- Update gaps.md (`f42287c`)
- Update README (`308907a`)
- Update README (`81066a5`)
- Update README (`64d7df9`)
- Update event description (`6da657d`)
- Update docs (`5cf2fda`)
- Adjust demo screen layout and fix mouse press demo (`04a6fe2`)
- Improve demo layout (`fa0a8bb`)
- Rename unit tests (`ff14cb4`)
- Refactor SvgRoot node (`e729a38`)
- Abort running demo server if wasm-pack fails (`d30cd32`)
- Use lazy MouseClosure (`fd479da`)
- Create detached element, set attributes, then append to DOM (`2dea752`)
- Allow elements to be added in batches (`bd9be57`)
- Avoid allocating storage for EventListener the node does not have one (`ea8ffbd`)
- Drop event listener from DOM when dropping from memory (`e1610a5`)
- Deprecate use of mouseover mouseout events (`87c1563`)
- Reduce per-frame string allocation (`b816df9`)
- Cache viewport size in SVGRoot (`7b027d1`)
- Update docs (`694df3c`)
- Update README (`be0a9ff`)
- Update README (`0025b02`)
- Update README (`8787881`)
- Update README (`a350780`)
- Update README (`1517fbe`)

### Fixed

- Correct doc comment formatting (`74b0e4f`)
- Correct integration tests (`4e1d644`)
- Correct integration tests - again (`2520b46`)

### Removed

- Remove unnecessary clone (`9bf3d49`)
