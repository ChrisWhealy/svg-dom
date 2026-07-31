# Demo Gallery Changelog

All notable changes to the `svg-dom-demo` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project's crate version follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The `svg-dom-demo` gallery is not published as part of the crate's release, so the changes listed here are not tied to any version number known to `crates.io`.

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
