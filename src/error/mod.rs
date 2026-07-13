// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// All failure modes that can arise when working with the SVG DOM.
///
/// Every fallible function in this crate returns `Result<_, Error>`.
///
/// The variants cover eight categories:
///
/// - you asked for a non-existent element by id ([`Error::ElementNotFound`])
/// - a `web-sys` call returned a JavaScript error ([`Error::Dom`])
/// - a JavaScript value couldn't be cast to the expected Rust type ([`Error::CastFailed`])
/// - a marker id string was rejected by crate-level validation ([`Error::InvalidMarkerId`])
/// - a gradient id string was rejected by crate-level validation ([`Error::InvalidGradientId`])
/// - a clip-path id string was rejected by crate-level validation ([`Error::InvalidClipPathId`])
/// - a symbol id string was rejected by crate-level validation ([`Error::InvalidSymbolId`])
/// - a generic setter was called with an attribute name that has a dedicated typed setter ([`Error::ReservedAttribute`])
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

    /// A generic attribute setter was called with an attribute name that is managed by a dedicated typed setter.
    ///
    /// The `id` attribute on [`SvgMarker`](crate::SvgMarker) is managed by [`SvgMarker::set_id`](crate::SvgMarker::set_id),
    /// which keeps the cached id in sync with the DOM.
    /// Passing `"id"` (case-insensitively) to [`SvgMarker::set_attr`](crate::SvgMarker::set_attr) or
    /// [`SvgMarker::set_attr_display`](crate::SvgMarker::set_attr_display) returns this error.
    ///
    /// The inner `&'static str` names the reserved attribute.
    ReservedAttribute(&'static str),
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
            Error::ReservedAttribute(name) => {
                write!(f, "attribute {name:?} is reserved; use the dedicated setter")
            },
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[cfg(test)]
mod unit_tests;
