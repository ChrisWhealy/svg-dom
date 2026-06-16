pub mod circle;
pub mod group;
pub mod line;
pub mod path;
pub mod rect;
pub mod svg_root;
pub mod text;
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

pub(crate) fn set(el: &impl AsRef<web_sys::Element>, name: &str, value: &str) -> Result<(), Error> {
    el.as_ref()
        .set_attribute(name, value)
        .map_err(|e| Error::Dom(format!("{e:?}")))
}
