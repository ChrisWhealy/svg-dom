// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// All failure modes that can arise when working with the SVG DOM.
///
/// Every fallible function in this crate returns `Result<_, Error>`.
///
/// The variants cover seven categories, one of which contains six subtypes:
///
/// - you asked for a non-existent element by id ([`Error::ElementNotFound`])
/// - a `web-sys` call returned a JavaScript error ([`Error::Dom`])
/// - a JavaScript value couldn't be cast to the expected Rust type ([`Error::CastFailed`])
/// - a generic setter was called with an attribute name that has a dedicated typed setter ([`Error::ReservedAttribute`])
/// - a non-empty [`PathDef`](crate::PathDef) sequence was supplied that did not begin with a `MoveTo` command ([`Error::InvalidPathData`])
/// - a `viewBox` was supplied with a non-finite component, or a negative width/height ([`Error::InvalidViewBox`])
/// - Crate-level validation errors for various id strings
///   - a bad marker id ([`Error::InvalidMarkerId`])
///   - a bad gradient id ([`Error::InvalidGradientId`])
///   - a bad clip-path id ([`Error::InvalidClipPathId`])
///   - a bad symbol id ([`Error::InvalidSymbolId`])
///   - a bad pattern id ([`Error::InvalidPatternId`])
///   - a bad filter id ([`Error::InvalidFilterId`])
#[derive(Debug)]
pub enum Error {
    /// No element with the given id exists in the current document.
    ///
    /// Returned by [`SvgRoot::attach`](crate::SvgRoot::attach) and [`SvgRoot::create_in`](crate::SvgRoot::create_in)
    /// when `document.getElementById(id)` returns `null`.
    ElementNotFound(String),

    /// A `web-sys` DOM operation returned a JavaScript error.
    ///
    /// The `JsValue` error returned by the browser is debug-formatted then passed back as the inner `String`.
    Dom(String),

    /// A `JsCast` conversion to the expected DOM type failed.
    ///
    /// This typically means the element found in the DOM is not the type the function expected — for example, calling
    /// [`SvgRoot::attach`](crate::SvgRoot::attach) with the id of a `<div>` rather than an `<svg>`.  The inner `&str`
    /// names the target type.
    CastFailed(&'static str),

    /// A marker `id` string was rejected before reaching the DOM.
    ///
    /// Valid marker ids must match `[A-Za-z_][A-Za-z0-9_-]*`: an ASCII letter or underscore followed by
    /// zero or more ASCII letters, digits, underscores, or hyphens.
    /// Passing a string that does not match this pattern to [`SvgDefs::marker`](crate::SvgDefs::marker),
    /// [`SvgDefs::build_marker`](crate::SvgDefs::build_marker), or the `set_marker_*` setters returns this error.
    ///
    /// The inner `String` is the rejected id.
    InvalidMarkerId(String),

    /// A gradient `id` string was rejected before reaching the DOM.
    ///
    /// Valid gradient ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`: an ASCII letter or underscore followed by
    /// zero or more ASCII letters, digits, underscores, or hyphens.
    ///
    /// This error will be returned if a string that does not match this pattern is passed to any of these functions:
    /// * [`SvgDefs::linear_gradient`](crate::SvgDefs::linear_gradient),
    /// * [`SvgDefs::build_linear_gradient`](crate::SvgDefs::build_linear_gradient),
    /// * [`SvgDefs::radial_gradient`](crate::SvgDefs::radial_gradient),
    /// * [`SvgDefs::build_radial_gradient`](crate::SvgDefs::build_radial_gradient),
    /// * or the `set_fill_gradient` / `set_stroke_gradient` setters.
    ///
    /// The inner `String` is the rejected id.
    InvalidGradientId(String),

    /// A clip-path `id` string was rejected before reaching the DOM.
    ///
    /// Valid clip-path ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`: an ASCII letter or underscore followed
    /// by zero or more ASCII letters, digits, underscores, or hyphens.
    ///
    /// This error is returned when a non-conforming string is passed to
    /// [`SvgDefs::clip_path`](crate::SvgDefs::clip_path),
    /// [`SvgDefs::build_clip_path`](crate::SvgDefs::build_clip_path), or
    /// [`SvgNode::set_clip_path`](crate::SvgNode::set_clip_path).
    ///
    /// The inner `String` is the rejected id.
    InvalidClipPathId(String),

    /// A symbol `id` string was rejected before reaching the DOM.
    ///
    /// Valid symbol ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`: an ASCII letter or underscore followed
    /// by zero or more ASCII letters, digits, underscores, or hyphens.
    ///
    /// This error is returned when a non-conforming string is passed to
    /// [`SvgDefs::symbol`](crate::SvgDefs::symbol) or
    /// [`SvgDefs::build_symbol`](crate::SvgDefs::build_symbol).
    ///
    /// The inner `String` is the rejected id.
    InvalidSymbolId(String),

    /// A pattern `id` string was rejected before reaching the DOM.
    ///
    /// Valid pattern ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`: an ASCII letter or underscore followed
    /// by zero or more ASCII letters, digits, underscores, or hyphens.
    ///
    /// This error is returned when a non-conforming string is passed to
    /// [`SvgDefs::pattern`](crate::SvgDefs::pattern),
    /// [`SvgDefs::build_pattern`](crate::SvgDefs::build_pattern), or
    /// [`SvgNode::set_fill_pattern`](crate::SvgNode::set_fill_pattern) /
    /// [`SvgNode::set_stroke_pattern`](crate::SvgNode::set_stroke_pattern).
    ///
    /// The inner `String` is the rejected id.
    InvalidPatternId(String),

    /// A filter `id` string was rejected before reaching the DOM.
    ///
    /// Valid filter ids must match the pattern `[A-Za-z_][A-Za-z0-9_-]*`: an ASCII letter or underscore followed
    /// by zero or more ASCII letters, digits, underscores, or hyphens.
    ///
    /// This error is returned when a non-conforming string is passed to
    /// [`SvgDefs::filter`](crate::SvgDefs::filter),
    /// [`SvgDefs::build_filter`](crate::SvgDefs::build_filter), or
    /// [`SvgNode::set_filter`](crate::SvgNode::set_filter).
    ///
    /// The inner `String` is the rejected id.
    InvalidFilterId(String),

    /// A generic attribute setter was called with an attribute name that is managed by a dedicated typed setter.
    ///
    /// The `id` attribute on [`SvgMarker`](crate::SvgMarker) is managed by [`SvgMarker::set_id`](crate::SvgMarker::set_id),
    /// which keeps the cached id in sync with the DOM.
    /// Passing `"id"` (case-insensitively) to [`SvgMarker::set_attr`](crate::SvgMarker::set_attr) or
    /// [`SvgMarker::set_attr_display`](crate::SvgMarker::set_attr_display) returns this error.
    ///
    /// The inner `&'static str` names the reserved attribute.
    ReservedAttribute(&'static str),

    /// A non-empty [`PathDef`](crate::PathDef) sequence did not begin with a `MoveTo` command.
    ///
    /// The SVG path grammar requires every non-empty path to start with a moveto (`M`/`m`).
    /// All compliant SVG user agents will silently render nothing for a path whose first command is anything else,
    /// neither will they report any error to the browser.
    ///
    /// This check catches that problem before it reaches the DOM.
    ///
    /// Returned by the `path_from_defs` factory method (on [`SvgRoot`](crate::SvgRoot) and its siblings),
    /// [`SvgNode::set_d_from_defs`](crate::SvgNode::set_d_from_defs), [`SvgAttrs::d_from_defs`](crate::SvgAttrs::d_from_defs)
    /// / [`d_from_defs_fixed`](crate::SvgAttrs::d_from_defs_fixed), and their
    /// [`AnimationFrame`](crate::AnimationFrame) counterparts.
    ///
    /// Not returned by the lower-level [`build_d`](crate::build_d) / [`write_d`](crate::write_d) (or their `_fixed`
    /// siblings): those are general-purpose formatters that may legitimately be used to build a path-data *fragment*
    /// that is not meant to stand alone, so they format whatever `PathDef` sequence they are given without this check.
    ///
    /// This error is raised only by functions existing at the boundary where a path sequence about to be committed to
    /// an element's live `d` attribute.
    ///
    /// The inner `String` describes the problem.
    InvalidPathData(String),

    /// A `viewBox` was rejected before reaching the DOM.
    ///
    /// SVG defines `viewBox` as 4 SVG numbers (`x`, `y`, `width`, `height`); this crate additionally requires `width`
    /// and `height` to be non-negative (the SVG spec treats a negative value as an error for the whole attribute) and
    /// every component to be finite (`NaN`/`±infinity` are not valid SVG numbers, even though `f64` can represent them
    /// and `Display` can format them).
    ///
    /// A `width`/`height` of exactly `0.0` is valid syntax and is therefore accepted.  As per the SVG spec, this is a
    /// trick by which rendering of an element can be disabled if `width` or `height` is `0.0`.
    ///
    /// Returned by [`SvgRoot::set_view_box`](crate::SvgRoot::set_view_box),
    /// [`SvgSymbol::set_view_box`](crate::SvgSymbol::set_view_box), and
    /// [`SvgPattern::set_view_box`](crate::SvgPattern::set_view_box).
    ///
    /// The inner `&'static str` describes which check failed.
    InvalidViewBox(&'static str),
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl From<std::fmt::Error> for Error {
    /// Maps a formatting failure into [`Error::Dom`].
    ///
    /// Writing into a `String` scratch buffer (as the `set_translate`/`set_transform_fmt` helpers do) is infallible in
    /// practice, but `write!` is typed to return `std::fmt::Error`. So this conversion lets those helpers use `?`
    /// without a dedicated error variant.
    fn from(_: std::fmt::Error) -> Self {
        Error::Dom("failed to format SVG value".into())
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Converts a `web-sys` `JsValue` error into [`Error::Dom`] by debug-formatting it.
///
/// Centralises the `.map_err(|e| Error::Dom(format!("{e:?}")))` that every fallible `web-sys` DOM call would otherwise
/// repeat; used crate-wide as `.map_err(dom_err)`.
pub(crate) fn dom_err(e: wasm_bindgen::JsValue) -> Error {
    Error::Dom(format!("{e:?}"))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ElementNotFound(id) => write!(f, "element not found: #{id}"),
            Error::Dom(msg) => write!(f, "DOM error: {msg}"),
            Error::CastFailed(ty) => write!(f, "JsCast to {ty} failed"),
            Error::InvalidMarkerId(id) => write!(f, "invalid svg marker id: {id:?}"),
            Error::InvalidGradientId(id) => write!(f, "invalid svg gradient id: {id:?}"),
            Error::InvalidClipPathId(id) => write!(f, "invalid svg clip-path id: {id:?}"),
            Error::InvalidSymbolId(id) => write!(f, "invalid svg symbol id: {id:?}"),
            Error::InvalidPatternId(id) => write!(f, "invalid svg pattern id: {id:?}"),
            Error::InvalidFilterId(id) => write!(f, "invalid svg filter id: {id:?}"),
            Error::ReservedAttribute(name) => {
                write!(f, "attribute {name:?} is reserved; use the dedicated setter")
            },
            Error::InvalidPathData(msg) => write!(f, "invalid path data: {msg}"),
            Error::InvalidViewBox(msg) => write!(f, "invalid viewBox: {msg}"),
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[cfg(test)]
mod unit_tests;
