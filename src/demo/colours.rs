//! Named colours for the demo gallery.
//!
//! Values are plain CSS colour strings, suitable for passing straight to [`SvgNode::set_fill`](crate::SvgNode::set_fill)
//! and friends.

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Paint keywords (not really colours, but valid paint values)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// No paint at all — e.g. a shape with a stroke but no fill.
pub const NONE: &str = "none";
/// Fully transparent — used for invisible pointer-capture surfaces.
pub const TRANSPARENT: &str = "transparent";

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Theme — the dark GitHub-ish chrome shared with style.css
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Canvas background; also used to make a shape blend into the panel.
pub const CANVAS_BG: &str = "#161b22";
/// The darkest background tone; used here as dark text on a light fill.
pub const INK: &str = "#0d1117";
/// Raised panel fill for the drag-and-drop drop zone.
pub const DROP_ZONE_FILL: &str = "#1b222c";
/// Subtle border around the drop zone.
pub const DROP_ZONE_BORDER: &str = "#30363d";
/// Primary readout / body text.
pub const TEXT: &str = "#c9d1d9";
/// Secondary, de-emphasised text.
pub const TEXT_MUTED: &str = "#8b949e";
/// Small grey captions beneath each demo.
pub const CAPTION: &str = "#777";
/// Light text used for the plain-text sample.
pub const PLAIN_TEXT: &str = "#d0d0d0";
/// Primary accent (group 1, draggable card, …).
pub const ACCENT_BLUE: &str = "#58a6ff";
/// Secondary accent (group 2).
pub const ACCENT_AMBER: &str = "#d29922";
/// Child shape — warm orange.
pub const LEAF_ORANGE: &str = "#f0883e";
/// Child shape — green.
pub const LEAF_GREEN: &str = "#3fb950";
/// Darkened teal shown while the press-state button is held down.
pub const TEAL_PRESSED: &str = "#0a3d3d";
/// Thin connector / guide strokes.
pub const GUIDE: &str = "#444";
/// A plain mid-grey wire/line.
pub const WIRE: &str = "#aaa";
/// The colour swatch before any hue has been sampled.
pub const SWATCH_EMPTY: &str = "#222";
/// The reset button while there is nothing to reset.
pub const RESET_IDLE: &str = "#555";

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Named CSS palette — the bright fills used across the shape demos
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub const WHITE: &str = "white";
pub const STEELBLUE: &str = "steelblue";
pub const SLATE_BLUE: &str = "slateblue";
pub const SLATE_GRAY: &str = "slategray";
pub const LIGHT_SKY_BLUE: &str = "lightskyblue";
pub const CORAL: &str = "coral";
pub const TOMATO: &str = "tomato";
pub const GOLD: &str = "gold";
pub const GOLDENROD: &str = "goldenrod";
pub const MEDIUM_SEA_GREEN: &str = "mediumseagreen";
pub const TEAL: &str = "teal";
pub const ORCHID: &str = "orchid";
pub const MEDIUM_ORCHID: &str = "mediumorchid";
pub const DARK_ORANGE: &str = "darkorange";
pub const CRIMSON: &str = "crimson";
