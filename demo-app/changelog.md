# Demo Gallery Changelog

All notable changes to the `svg-dom-demo` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project's crate version follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The `svg-dom-demo` gallery is not published as part of the crate's release, so the changes listed here are not tied to any version number known to `crates.io`.

## [0.1.14] - 2026-08-06

### Added

- Make `feMorphology` demo interactive (`beb85aa`)

### Fixed

- Fix test assertion after colour change (`4b6027b`)
- Doc only: correct description of outline at radius zero (`bde9121`)
- Improve test coverage of outline filter graph (``)

## [0.1.13] - 2026-08-05

### Added

- Make `feTurbulence`/`feDisplacementMap` demo interactive (`0088502`)
- Add specific turbulence scale 0 render test (`17d0f1d`)
- Add positive control to the raster test (`9415632`)

### Changed

- Rename the shared CDP test infrastructure (`5df4eb9`)

### Fixed

- Doc only: correct description of `scale` (`6005461`)
- Label channel slider positions (`8986654`)
- Prove all four channel mappings in browser test (`471e4bf`)
- Doc only: correct rester test docs (`7c49ecf`)
- Reconcile the Chrome sandbox documentation with the launcher (`4655451`)
- Doc only: correct stale testing documentation comments (`8926717`)

## [0.1.12] - 2026-08-04

### Added

- Make feColorMatrix demo interactive (`d6d3669`)

### Fixed

- Fix saturate value by resolving to 2dp (`52ba939`)
- Doc only: correct fourth feColorMatrix demo description (`e2305aa`)
- Allow slider range to extend beyond 1.0 to support oversaturation (`da64f7d`)
- HueRotate slider updates `aria-valuetext` to show degrees unit (`4b1383f`)

## [0.1.11] - 2026-08-04

### Added

- Make feComponentTransfer demo interactive (`27d7d16`)

### Fixed

- Doc only: minor corrections (`41ad3b2`)

### Changed

- Refactor browser tests (`7cbe902`)
- Bump demo-app version (`2e7fdb9`)

## [0.1.10] - 2026-08-03

### Added

- Make feBlend demo interactive (`3b6d30a`)

### Changed

- Extend browser test to verify complete filter chain (`1ceadc0`)

### Fixed

- Remove hard-coded default-option index (`6c3c0fb`)
- Associate visible label programmatically (`ef4d586`)
- Doc only: correct explanation of feBlend alpha-compositing behaviour (`e8bd15a`)
- `<label>` should contain `<span>` not `<div>` (`185bc9a`)
- Test that BlendMode's members and in order and labels are correct (`e7bfe86`)

## [0.1.9] - 2026-08-01

### Added

- Make feFilter demos interactive (`d87401c`)

### Fixed

- Expand filter region for ma stdDeviation value (`52dce95`)
- Correct feDropShadow expansion order (`8657f80`)
- Correct demo to expose intended API feature (`a230dbd`)
- Doc only: correct stale docs & comments (`0dd3182`)
- Improve shadow region test to document font-family change (`3c3f0a7`)

### Changed

- Strengthen browser test coverage (`5eacaff`)

## [0.1.8] - 2026-07-31

### Added

- Make radialGradient demos interactive (`0308224`)

### Fixed

- Explain SVG 2 edge-case when focal point move outside circle (`205a04a`)
- Test shared vertical-slider sizing contract (`d0e89f9`)

## [0.1.7] - 2026-07-31

### Added

- Make the linearGradient demos interactive (`ea0f6e4`)
- Add regression test for linearGradient demos (`74b08b4`)
- Add initial `aria-valuetext` values to linearGradient demo sliders (`dfdf881`)
- Expose spectrum constraints to UI controls (`ef81567`)
- Improve descriptive clarity of `aria-label` names (`0a0ac89`)
- Improve `aria-valuetext` test coverage (`cfd9301`)

### Fixed

- Correct `aria-valuetext` for linearGradient demo slider (``)
- Correct stale doc comments (`c6c40b6`)
- Correct accuracy of spectrum bounds (`8ce802e`)
- Correct `aria-orientation` for vertical slider (`5b3ab74`)

## [0.1.6] - 2026-07-30

### Changed

- Rerun prepare_gallery on each HTTP request for live reload (`4f1fc2c`)
- Adapt all demo documentation to use simplified technical English style guide (`2e59696`)
- Tidy up foreignObject demo description and bump version number (`2c67bb1`)
- Refactor remaining demo modules (`0900e53`)

### Fixed

- Doc only: Update stale file references (`781b1db`)

## [0.1.5] - 2026-07-30

### Added

- Doc only: clarify that tick mark test does not cover WebKit/Safari or pixel-perfect alignment (`413e6d6`)
- Make `set_view_box()` demo interactive (`e54b3ab`)
- Add regression test to `set_view_box()` test (`4c76973`)
- Make image demo interactive (`eb9abbc`)
- Test shared radio button behaviour in `svg-dom-demo` (`2e610df`)

### Fixed

- Avoid hotpath String allocation in textPath demo (`91cce99`)
- Correct description of `set_view_box()` demo (`32da274`)
- Avoid hotpath String allocation in `set_view_box()` demo (`3291dab`)
- Remove redundant caption (`b137ab5`)
- Resize foreignObject dashed bounding box (`d21ee6d`)
- Correct stale comments in CSS file (`0809e54`)

### Changed

- Improve radio button test (`853209e`)
- Refactor XHTML helpers into their own module (`8dbfff1`)
- Refactor paint and structure demos (`1398d54`)

## [0.1.4] - 2026-07-29

### Added

- Doc only: explain demo-app architecture is suitable for a demo, not a live application (`39d0beb`)

### Fixed

- Slider maximum can exceed the actual path length (`2fb3c88`)
- foreignObject controls lack complete accessible names and grouping (`9b87f85`)
- Test new interactive test behaviour (`f810ff2`)
- Correct typos (`dfd6d8c`)
- Correct broken wasm-pack argument order (`64bdab4`)

### Changed

- Refactor text demos (`60f676a`)

## [0.1.3] - 2026-07-28

### Added

- Make text-anchor demo interactive (`3dfbc85`)
- Make dominant-baseline demo interactive (`26a7e69`)
- Make start-offset demo interactive (`f07f570`)

### Fixed

- Correct broken links in demo (`891510f`)

## [0.1.2] - 2026-07-28

### Added

- Doc only: animations are started lazily but keep running when no longer selected (`2c03f1e`)
- Add gallery pipeline tests to CI (`546316b`)
- Doc only: update `append_source_frame()` documentation (`c9f3027`)

### Changed

- Make port number a placeholder in the HTML page (`acb2329`)
- Extract all the build steps from `main.rs` into `build.rs` (`686c54e`)

### Fixed

- Update demo style sheet (`a12837b`)
- Detect duplicate ids in demo_gallery! (`753a5f7`)
- The validate module should return an error instead of terminating the process (`d331b8d`)
- Harden catalogue validation against false positives (`a005a3a`)
- Improve text-node escape function to include `<` and `>` (`98fd2e7`)
- Handle invalid PORT number gracefully (`81f0592`)
- Doc only: update stale docs (`500e1cb`)

## [0.1.1] - 2026-07-28

### Changed

- Refactor demo gallery into individual panels (`d987feb`)
- Significant restructuring of demo server (`5664c2d`)

## [0.1.0] - 2026-07-25

### Added

- Split the demo gallery into its own workspace crate `svg-dom-demo` (`d739ade`)
