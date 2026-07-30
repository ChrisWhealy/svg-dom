// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// One file per demo, each exporting its own `fn demo()` — the same split `texts/mod.rs` uses, and for the same
// reason: `demo_gallery!`'s `$path:path` grammar accepts any nesting depth, so `structure::demo_symbol::demo` works
// exactly like `texts::demo_text::demo`, and DEMO_SOURCE_FILES (lib.rs) keys each file by its own module path so
// two files can each define `fn demo()` without colliding on lookup. No shared helper functions live here: unlike
// `texts`'s `radio_group`/`sine_wave_path`, nothing below is used by more than one of these files.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) mod demo_accessibility;
pub(crate) mod demo_anchor;
pub(crate) mod demo_anim;
pub(crate) mod demo_foreign_object;
pub(crate) mod demo_group;
pub(crate) mod demo_image;
pub(crate) mod demo_marker;
pub(crate) mod demo_marker_view_box;
pub(crate) mod demo_style;
pub(crate) mod demo_switch;
pub(crate) mod demo_symbol;
pub(crate) mod demo_tree_nav;
pub(crate) mod demo_use;
pub(crate) mod demo_view;
pub(crate) mod demo_view_box;
