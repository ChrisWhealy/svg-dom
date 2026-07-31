// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// One file per demo, each exporting its own `fn demo()` — the same split `texts/mod.rs` uses, and for the same
// reason: `demo_gallery!`'s `$path:path` grammar accepts any nesting depth, so
// `geometry::demo_geometry_bounding_box::demo` works exactly like `texts::demo_text::demo`, and DEMO_SOURCE_FILES
// (lib.rs) keys each file by its own module path so two files can each define `fn demo()` without colliding on
// lookup. No shared helper functions live here: each demo below only calls into lib.rs's own
// `caption`/`keep_demo_anim`/`keep_demo_node`, nothing module-local.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) mod demo_geometry_bounding_box;
pub(crate) mod demo_geometry_path_follow;
