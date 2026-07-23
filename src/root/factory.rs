use std::cell::RefCell;

use web_sys::{Document, SvgElement};

use crate::{Error, SvgNode};

use super::{
    attrs::SvgAttrs,
    path::path_def::{PathDef, validate_starts_with_moveto},
    utils::{Point, Size},
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
        super::create_svg_element(self.document(), tag, "SvgElement")
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
    fn create_ellipse(&self, centre: Point, radii: Size) -> Result<SvgNode, Error> {
        let node = self.make_node("ellipse")?;
        {
            let mut attrs = self.attrs().borrow_mut();
            attrs.display(&node, "cx", centre.x)?;
            attrs.display(&node, "cy", centre.y)?;
            attrs.display(&node, "rx", radii.width)?;
            attrs.display(&node, "ry", radii.height)?;
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
    // Writes `d` through the factory's own retained scratch buffer (see `create_rect` etc. above) instead of `build_d`,
    // which allocates a fresh `String` on every call.
    fn create_path_from_defs(&self, defs: &[PathDef]) -> Result<SvgNode, Error> {
        // Validated once, here, before any DOM node exists — d_from_validated_defs (rather than d_from_defs) skips
        // the redundant second check that would otherwise re-inspect the same already-validated slice.
        validate_starts_with_moveto(defs)?;
        let node = self.make_node("path")?;
        self.attrs().borrow_mut().d_from_validated_defs(&node, defs)?;
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_polyline(&self, points: &[Point]) -> Result<SvgNode, Error> {
        let node = self.make_node("polyline")?;
        self.attrs().borrow_mut().points(&node, points)?;
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_polygon(&self, points: &[Point]) -> Result<SvgNode, Error> {
        let node = self.make_node("polygon")?;
        self.attrs().borrow_mut().points(&node, points)?;
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
        node.set_text(content);
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_group(&self) -> Result<SvgNode, Error> {
        let node = self.make_node("g")?;
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_image(&self, href: &str, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        let node = self.make_node("image")?;
        node.set_attr("href", href)?;
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
    fn create_use(&self, href: &str, at: Point) -> Result<SvgNode, Error> {
        let node = self.make_node("use")?;
        node.set_attr("href", href)?;
        {
            let mut attrs = self.attrs().borrow_mut();
            attrs.display(&node, "x", at.x)?;
            attrs.display(&node, "y", at.y)?;
        }
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_anchor(&self, href: &str) -> Result<SvgNode, Error> {
        let node = self.make_node("a")?;
        node.set_attr("href", href)?;
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_switch(&self) -> Result<SvgNode, Error> {
        let node = self.make_node("switch")?;
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_style(&self, css: &str) -> Result<SvgNode, Error> {
        let node = self.make_node("style")?;
        node.set_text(css);
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_metadata(&self, content: &str) -> Result<SvgNode, Error> {
        let node = self.make_node("metadata")?;
        node.set_text(content);
        self.append_node(&node)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    fn create_foreign_object(&self, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        let node = self.make_node("foreignObject")?;
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
}
