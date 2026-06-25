/// The reusable attribute writer [`SvgAttrs`](crate::SvgAttrs) and chainable [`AttrWriter`](crate::AttrWriter).
pub mod attrs;
/// The [`SvgBatch`](crate::SvgBatch) builder that appends many elements to the DOM in one operation.
pub mod batch;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Factories and helper types
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) mod factory;
/// The [`SvgRoot::circle`](crate::SvgRoot::circle) factory.
pub mod circle;
/// The [`SvgRoot::ellipse`](crate::SvgRoot::ellipse) factory.
pub mod ellipse;
/// The [`SvgRoot::group`](crate::SvgRoot::group) factory.
pub mod group;
/// The [`SvgRoot::line`](crate::SvgRoot::line) factory.
pub mod line;
/// The [`SvgRoot::path`](crate::SvgRoot::path) factory.
pub mod path;
/// The [`SvgRoot::polygon`](crate::SvgRoot::polygon) factory.
pub mod polygon;
/// The [`SvgRoot::polyline`](crate::SvgRoot::polyline) factory.
pub mod polyline;
/// The [`SvgRoot::rect`](crate::SvgRoot::rect) factory.
pub mod rect;
/// The `<svg>` root wrapper [`SvgRoot`](crate::SvgRoot).
pub mod svg_root;
/// The [`SvgRoot::text`](crate::SvgRoot::text) factory.
pub mod text;
/// Geometry helper types [`Point`](utils::Point) and [`Size`](utils::Size).
pub mod utils;

use crate::error::Error;
use web_sys::Document;

pub(crate) const SVG_NS: &str = "http://www.w3.org/2000/svg";

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// DOM helpers
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn document() -> Result<Document, Error> {
    web_sys::window()
        .ok_or_else(|| Error::Dom("no available window".into()))?
        .document()
        .ok_or_else(|| Error::Dom("window has no document".into()))
}
