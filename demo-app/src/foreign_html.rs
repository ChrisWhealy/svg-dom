use crate::{NS_XHTML, dom_err};
use svg_dom::{Error, SvgNode};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Shared plumbing for building plain HTML inside an SVG <foreignObject>. Used across the gallery, not by one demo
// module: `texts::radio_group` and every demo module that builds its own `<foreignObject>` control directly (e.g.
// `structure::demo_marker_view_box`'s slider) are all callers. Kept crate-local (`pub(crate)`), not part of
// `demo-app`'s own public surface, since nothing outside this crate builds these controls.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Creates an XHTML-namespaced element — the `create_element_ns(Some(NS_XHTML), tag).map_err(dom_err)` every plain
/// HTML element built inside a `<foreignObject>` needs, shortened to one call.
pub(crate) fn xhtml(document: &web_sys::Document, tag: &str) -> Result<web_sys::Element, Error> {
    document.create_element_ns(Some(NS_XHTML), tag).map_err(dom_err)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The owner `Document` of an SVG node built via `SvgRoot::foreign_object` — every interactive control needs it, to
/// build the plain HTML it puts inside that `<foreignObject>`.
pub(crate) fn foreign_object_document(fo: &SvgNode) -> Result<web_sys::Document, Error> {
    fo.as_element()
        .owner_document()
        .ok_or_else(|| Error::Dom("cannot identify owner document for SVG foreignObject".into()))
}
