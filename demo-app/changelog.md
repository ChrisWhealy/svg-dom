# Demo Gallery Changelog

All notable changes to the `svg-dom-demo` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project's crate version follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The `svg-dom-demo` gallery is not published as part of the crate's release, so the changes listed here are not tied to any version number known to `crates.io`.

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
