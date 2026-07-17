//! Geometry read-back for [`SvgNode`](crate::SvgNode) — bounding boxes, path measurement, and coordinate-system
//! transforms.
//!
//! Every method here reads real, current geometry from the browser, which forces a layout/reflow: none of them
//! belong on a hot per-frame or per-pointer-move path without dedup/throttling (the same caveat
//! [`computed_text_length`](crate::SvgNode::computed_text_length) already carries for the same reason).

use crate::{
    Error, SvgNode, dom_err,
    root::utils::{Matrix2D, Point, Rect, Size},
};
use wasm_bindgen::JsCast;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    /// # Bounding box
    ///
    /// Returns this element's bounding box in its own **local, user-space** coordinates, by wrapping
    /// [`SVGGraphicsElement.getBBox()`]. This is unaffected by any transform applied to the element or its ancestors —
    /// see [`Rect`]'s own doc comment for how this differs from [`bounding_client_rect`](Self::bounding_client_rect).
    ///
    /// **Performance:** this call triggers a browser layout read. Do not call it inside a hot animation or pointer-move
    /// callback unless you have determined that this cost is acceptable.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser rejects the call, or if this element does not implement
    /// `SVGGraphicsElement` (every element type this crate's factories return as a plain `SvgNode` does, so this is a
    /// defensive case rather than one reachable through this crate's own API).
    ///
    /// [`SVGGraphicsElement.getBBox()`]: https://developer.mozilla.org/docs/Web/API/SVGGraphicsElement/getBBox
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(10.0, 10.0), Size::new(80.0, 40.0))?;
    ///
    /// let bbox = rect.bounding_box()?;
    /// assert_eq!((bbox.origin.x, bbox.origin.y), (10.0, 10.0));
    /// assert_eq!((bbox.size.width, bbox.size.height), (80.0, 40.0));
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn bounding_box(&self) -> Result<Rect, Error> {
        let graphics = self
            .inner
            .element
            .dyn_ref::<web_sys::SvgGraphicsElement>()
            .ok_or_else(|| Error::Dom("getBBox: element does not implement SVGGraphicsElement".into()))?;
        graphics
            .get_b_box()
            .map(|r| Rect {
                origin: Point::new(f64::from(r.x()), f64::from(r.y())),
                size: Size::new(f64::from(r.width()), f64::from(r.height())),
            })
            .map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Current transformation matrix
    ///
    /// Returns this element's current transformation matrix — the accumulated transform from this element's own
    /// coordinate system to its nearest viewport ancestor — by wrapping [`SVGGraphicsElement.getCTM()`]. Returns `None`
    /// if the element is not currently part of a rendered tree, or does not implement `SVGGraphicsElement`.
    ///
    /// Unlike [`screen_ctm`](Self::screen_ctm), this does **not** include any ancestor's transform beyond the nearest
    /// viewport — see [`screen_ctm`](Self::screen_ctm) for the full-chain equivalent.
    ///
    /// **Performance:** this call triggers a browser layout read. Do not call it inside a hot animation or
    /// pointer-move callback unless you have determined that this cost is acceptable.
    ///
    /// [`SVGGraphicsElement.getCTM()`]: https://developer.mozilla.org/docs/Web/API/SVGGraphicsElement/getCTM
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    ///
    /// if let Some(ctm) = rect.ctm() {
    ///     println!("translation: ({}, {})", ctm.h_trans, ctm.v_trans);
    /// }
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn ctm(&self) -> Option<Matrix2D> {
        self.inner
            .element
            .dyn_ref::<web_sys::SvgGraphicsElement>()
            .and_then(web_sys::SvgGraphicsElement::get_ctm)
            .map(svg_matrix_to_matrix2d)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Screen-space transformation matrix
    ///
    /// Returns the accumulated transform from this element's own coordinate system all the way to the top-level
    /// viewport (screen space), by wrapping [`SVGGraphicsElement.getScreenCTM()`]. Returns `None` if the element is not
    /// currently part of a rendered tree, or does not implement `SVGGraphicsElement`.
    ///
    /// # Includes every ancestor's transform
    ///
    /// Unlike [`ctm`](Self::ctm), this includes **every ancestor's** transform, not just this element's own. Do not
    /// write the result straight back as this element's own local `transform` via [`set_matrix`](Self::set_matrix) /
    /// [`set_matrix_precise`](Self::set_matrix_precise) — that would double-apply whatever transform the ancestors
    /// already contribute. To convert a coordinate from screen space into this element's local space (or vice versa),
    /// invert (or compose with the inverse of) the parent's own `screen_ctm` first.
    ///
    /// **Performance:** this call triggers a browser layout read. Do not call it inside a hot animation or pointer-move
    /// callback unless you have determined that this cost is acceptable.
    ///
    /// [`SVGGraphicsElement.getScreenCTM()`]: https://developer.mozilla.org/docs/Web/API/SVGGraphicsElement/getScreenCTM
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    ///
    /// let screen = rect.screen_ctm();
    /// let local  = rect.ctm();
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn screen_ctm(&self) -> Option<Matrix2D> {
        self.inner
            .element
            .dyn_ref::<web_sys::SvgGraphicsElement>()
            .and_then(web_sys::SvgGraphicsElement::get_screen_ctm)
            .map(svg_matrix_to_matrix2d)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Total path length
    ///
    /// Returns the total length of this element's path, in user units, by wrapping
    /// [`SVGGeometryElement.getTotalLength()`]. Returns `None` for elements that do not implement `SVGGeometryElement` —
    /// this includes `<text>`, `<tspan>`, `<textPath>`, `<use>`, `<image>`, `<g>`, and the root `<svg>`; only `<rect>`,
    /// `<circle>`, `<ellipse>`, `<line>`, `<polyline>`, `<polygon>`, and `<path>` support it.
    ///
    /// **Performance:** this call triggers a browser layout read. Do not call it inside a hot animation or
    /// pointer-move callback unless you have determined that this cost is acceptable.
    ///
    /// [`SVGGeometryElement.getTotalLength()`]: https://developer.mozilla.org/docs/Web/API/SVGGeometryElement/getTotalLength
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    ///
    /// let perimeter = rect.total_length().unwrap_or(0.0);
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn total_length(&self) -> Option<f64> {
        self.inner
            .element
            .dyn_ref::<web_sys::SvgGeometryElement>()
            .map(|g| f64::from(g.get_total_length()))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Point at distance along a path
    ///
    /// Returns the point at `distance` user units along this element's path, by wrapping
    /// [`SVGGeometryElement.getPointAtLength()`]. `distance` is clamped to `[0, `[`total_length`](Self::total_length)`]`
    /// by the browser, so an out-of-range value does not error — it returns the start or end point.
    ///
    /// **Performance:** this call triggers a browser layout read. Do not call it inside a hot animation or
    /// pointer-move callback unless you have determined that this cost is acceptable.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser rejects the call, or if this element does not implement
    /// `SVGGeometryElement` — see [`total_length`](Self::total_length) for which element types that excludes.
    ///
    /// [`SVGGeometryElement.getPointAtLength()`]: https://developer.mozilla.org/docs/Web/API/SVGGeometryElement/getPointAtLength
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    ///
    /// let start = rect.point_at_length(0.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn point_at_length(&self, distance: f64) -> Result<Point, Error> {
        let geometry = self
            .inner
            .element
            .dyn_ref::<web_sys::SvgGeometryElement>()
            .ok_or_else(|| Error::Dom("getPointAtLength: element does not implement SVGGeometryElement".into()))?;
        geometry
            .get_point_at_length(distance as f32)
            .map(|p| Point::new(f64::from(p.x()), f64::from(p.y())))
            .map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Bounding rect in viewport coordinates
    ///
    /// Returns this element's rendered bounding rect in **CSS pixels relative to the browser viewport**, by wrapping
    /// [`Element.getBoundingClientRect()`]. Unlike [`bounding_box`](Self::bounding_box), this reflects every transform,
    /// `viewBox` scale, and CSS zoom currently in effect — see [`Rect`]'s own doc comment for the full distinction.
    ///
    /// This is infallible and available on every element.
    ///
    /// **Performance:** this call triggers a browser layout read. Do not call it inside a hot animation or
    /// pointer-move callback unless you have determined that this cost is acceptable.
    ///
    /// [`Element.getBoundingClientRect()`]: https://developer.mozilla.org/docs/Web/API/Element/getBoundingClientRect
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    ///
    /// let client_rect = rect.bounding_client_rect();
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn bounding_client_rect(&self) -> Rect {
        let r = self.inner.element.get_bounding_client_rect();
        Rect {
            origin: Point::new(r.x(), r.y()),
            size: Size::new(r.width(), r.height()),
        }
    }
}

/// Maps an `SVGMatrix`'s `a, b, c, d, e, f` fields onto [`Matrix2D`]'s role-named fields, using the exact same
/// mapping [`SvgNode::set_matrix`] writes out: `a`→`h_scale`, `b`→`v_skew`, `c`→`h_skew`, `d`→`v_scale`,
/// `e`→`h_trans`, `f`→`v_trans`.
fn svg_matrix_to_matrix2d(m: web_sys::SvgMatrix) -> Matrix2D {
    Matrix2D {
        h_scale: f64::from(m.a()),
        v_skew: f64::from(m.b()),
        h_skew: f64::from(m.c()),
        v_scale: f64::from(m.d()),
        h_trans: f64::from(m.e()),
        v_trans: f64::from(m.f()),
    }
}
