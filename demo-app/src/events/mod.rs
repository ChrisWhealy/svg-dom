// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// One file per demo, each exporting its own `fn demo()` — the same split `texts/mod.rs` uses, and for the same
// reason: `demo_gallery!`'s `$path:path` grammar accepts any nesting depth, so `events::demo_events_click::demo`
// works exactly like `texts::demo_text::demo`, and DEMO_SOURCE_FILES (lib.rs) keys each file by its own module path
// so two files can each define `fn demo()` without colliding on lookup. No shared helper functions live here: every
// demo below only calls into lib.rs's own `cached_label`/`caption`/`keep_demo_node`, nothing module-local.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) mod demo_events_classlist;
pub(crate) mod demo_events_click;
pub(crate) mod demo_events_colour;
pub(crate) mod demo_events_drag_drop_touch;
pub(crate) mod demo_events_group;
pub(crate) mod demo_events_keyboard_wheel;
pub(crate) mod demo_events_modifiers;
pub(crate) mod demo_events_passive;
pub(crate) mod demo_events_pointer_lifecycle;
pub(crate) mod demo_events_press;
