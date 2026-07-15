/// The reusable attribute writer [`SvgAttrs`](crate::SvgAttrs) and chainable [`AttrWriter`](crate::AttrWriter).
pub mod attrs;
/// The [`SvgBatch`](crate::SvgBatch) builder that appends many elements to the DOM in one operation.
pub mod batch;
/// The [`SvgClipPath`](crate::SvgClipPath) element and its [`ClipPathUnits`](crate::ClipPathUnits) enum.
pub mod clip_path;
/// The [`SvgDefs`](crate::SvgDefs) container for reusable SVG assets.
pub mod defs;
/// The [`SvgFilter`](crate::SvgFilter) element, its filter-primitive builder methods, and the
/// [`CompositeOperator`](crate::CompositeOperator) / [`ColorMatrixType`](crate::ColorMatrixType) enums.
pub mod filter;
/// The [`SvgLinearGradient`](crate::SvgLinearGradient), [`SvgRadialGradient`](crate::SvgRadialGradient),
/// [`GradientUnits`](crate::GradientUnits), and [`SpreadMethod`](crate::SpreadMethod) types.
pub mod gradient;
/// The [`SvgMarker`](crate::SvgMarker) element and its [`MarkerUnits`](crate::MarkerUnits) enum.
pub mod marker;
/// The [`SvgRoot::path`](crate::SvgRoot::path) / [`SvgRoot::path_from_defs`](crate::SvgRoot::path_from_defs)
/// factories and the type-safe [`PathDef`](crate::PathDef) path-segment builder.
pub mod path;
/// The [`SvgPattern`](crate::SvgPattern) element and its [`PatternUnits`](crate::PatternUnits) enum.
pub mod pattern;
/// The [`SvgSymbol`](crate::SvgSymbol) element.
pub mod symbol;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Factories and helper types
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) mod factory;
/// The `<svg>` root wrapper [`SvgRoot`](crate::SvgRoot).
pub mod svg_root;
/// Geometry helper types [`Point`](utils::Point) and [`Size`](utils::Size).
pub mod utils;

/// The [`SvgRoot::circle`](crate::SvgRoot::circle) factory.
mod circle;
/// The [`SvgRoot::ellipse`](crate::SvgRoot::ellipse) factory.
mod ellipse;
/// The [`SvgRoot::group`](crate::SvgRoot::group) factory.
mod group;
/// The [`SvgRoot::image`](crate::SvgRoot::image) factory.
mod image;
/// The [`SvgRoot::line`](crate::SvgRoot::line) factory.
mod line;
/// The [`SvgRoot::polygon`](crate::SvgRoot::polygon) factory.
mod polygon;
/// The [`SvgRoot::polyline`](crate::SvgRoot::polyline) factory.
mod polyline;
/// The [`SvgRoot::rect`](crate::SvgRoot::rect) factory.
mod rect;
/// The [`SvgRoot::text`](crate::SvgRoot::text) factory.
mod text;
/// The [`SvgRoot::use_node`](crate::SvgRoot::use_node) factory.
mod use_node;

use crate::{dom_err, error::Error};
use wasm_bindgen::JsCast;
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Creates a namespaced SVG element `tag` and casts it to `T`, mapping the two failure modes to [`Error::Dom`] /
/// [`Error::CastFailed`] (`type_name` names the target type).
///
/// Centralises the create-and-cast pattern shared by [`SvgFactory::make_element`](factory::SvgFactory) and
/// [`SvgRoot::create_in`](svg_root::SvgRoot::create_in).
pub(crate) fn create_svg_element<T: JsCast>(
    document: &Document,
    tag: &str,
    type_name: &'static str,
) -> Result<T, Error> {
    document
        .create_element_ns(Some(SVG_NS), tag)
        .map_err(dom_err)?
        .dyn_into::<T>()
        .map_err(|_| Error::CastFailed(type_name))
}
