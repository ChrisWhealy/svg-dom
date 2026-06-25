// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// All failure modes that can arise when working with the SVG DOM.
///
/// Every fallible function in this crate returns `Result<_, Error>`.
///
/// The Browser DOM generates three categories of error:
///
/// - you asked for an non-existent element by id ([`Error::ElementNotFound`])
/// - a `web-sys` call returned a JavaScript error ([`Error::Dom`])
/// - a JavaScript value couldn't be cast to the expected Rust type ([`Error::CastFailed`])
///
/// The variants in `enum Error` map directly to one these categories.
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
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl From<std::fmt::Error> for Error {
    /// Maps a formatting failure into [`Error::Dom`].
    ///
    /// Writing into a `String` scratch buffer (as the `set_translate`/`set_transform_fmt` helpers do) is infallible in
    /// practice, but `write!` is typed to return `std::fmt::Error`. So this conversion lets those helpers use `?`
    /// without a dedicated error variant.
    fn from(e: std::fmt::Error) -> Self {
        Error::Dom(e.to_string())
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
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[cfg(test)]
mod unit_tests;
