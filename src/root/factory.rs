use std::cell::RefCell;

use wasm_bindgen::JsCast;
use web_sys::{Document, SvgElement};

use crate::{Error, SvgNode};

use super::{
    attrs::SvgAttrs,
    utils::{Point, Size},
    SVG_NS,
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Shared implementation used by both [`SvgRoot`](crate::SvgRoot) and [`SvgBatch`](crate::SvgBatch).
///
/// The destination differs — `SvgRoot` appends directly to the live `<svg>`, whereas `SvgBatch` appends to a
/// `DocumentFragment` — but the element creation and initial attribute writing are identical.
pub(crate) trait SvgFactory {
    fn document(&self) -> &Document;
    fn attrs(&self) -> &RefCell<SvgAttrs>;
    fn append_node(&self, node: &SvgNode) -> Result<(), Error>;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn make_element(&self, tag: &str) -> Result<SvgElement, Error> {
        self.document()
            .create_element_ns(Some(SVG_NS), tag)
            .map_err(|e| Error::Dom(format!("{e:?}")))?
            .dyn_into::<SvgElement>()
            .map_err(|_| Error::CastFailed("SvgElement"))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn make_node(&self, tag: &str) -> Result<SvgNode, Error> {
        self.make_element(tag).map(SvgNode::new)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_rect(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        let node = self.make_node("rect")?;
        {
            let mut attrs = self.attrs().borrow_mut();
            attrs.display(&node, "x", top_left.x)?;
            attrs.display(&node, "y", top_left.y)?;
            attrs.display(&node, "width", size.width)?;
            attrs.display(&node, "height", size.height)?;
        }
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_circle(&self, centre: Point, radius: f64) -> Result<SvgNode, Error> {
        let node = self.make_node("circle")?;
        {
            let mut attrs = self.attrs().borrow_mut();
            attrs.display(&node, "cx", centre.x)?;
            attrs.display(&node, "cy", centre.y)?;
            attrs.display(&node, "r", radius)?;
        }
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_line(&self, start: Point, end: Point) -> Result<SvgNode, Error> {
        let node = self.make_node("line")?;
        {
            let mut attrs = self.attrs().borrow_mut();
            attrs.display(&node, "x1", start.x)?;
            attrs.display(&node, "y1", start.y)?;
            attrs.display(&node, "x2", end.x)?;
            attrs.display(&node, "y2", end.y)?;
        }
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_path(&self, d: &str) -> Result<SvgNode, Error> {
        let node = self.make_node("path")?;
        node.set_attr("d", d)?;
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_text(&self, anchored_at: Point, content: &str) -> Result<SvgNode, Error> {
        let node = self.make_node("text")?;
        {
            let mut attrs = self.attrs().borrow_mut();
            attrs.display(&node, "x", anchored_at.x)?;
            attrs.display(&node, "y", anchored_at.y)?;
        }
        node.as_element().set_text_content(Some(content));
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_group(&self) -> Result<SvgNode, Error> {
        let node = self.make_node("g")?;
        self.append_node(&node)?;
        Ok(node)
    }
}
