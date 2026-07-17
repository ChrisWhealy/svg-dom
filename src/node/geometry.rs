//! Geometry read-back for [`SvgNode`](crate::SvgNode) — bounding boxes, path measurement, and coordinate-system
//! transforms.
//!
//! Every method here crosses into the browser and may trigger synchronous style or layout calculation — if the
//! relevant geometry is already up to date the browser need not redo that work, but a caller has no way to know
//! which case applies from here. None of these methods belong on a hot per-frame or per-pointer-move path without
//! profiling for the actual scene/browser combination in use (the same caveat
//! [`computed_text_length`](crate::SvgNode::computed_text_length) already carries for the same reason).
//!
//! # These `f64` values may carry only `f32` precision
//!
//! This crate's own numeric types are consistently `f64`, matching every other geometry type in the crate
//! ([`Point`](crate::root::utils::Point), [`Size`](crate::root::utils::Size),
//! [`Matrix2D`](crate::root::utils::Matrix2D)). That consistency is worth keeping even though the browser values
//! behind five of these six methods do not actually carry `f64` precision:
//!
//! [`SvgNode::bounding_box`], [`SvgNode::ctm`], [`SvgNode::screen_ctm`], [`SvgNode::total_length`], and
//! [`SvgNode::point_at_length`] all go through legacy SVG DOM types — `SVGRect`, `SVGMatrix`, `SVGGeometryElement`'s
//! length/point methods — whose IDL has always used `float` (32-bit), never updated to `double`. `web-sys` mirrors
//! this faithfully: `SvgRect`/`SvgMatrix`/`SvgPoint`'s field getters and `get_total_length` return `f32`, and
//! `get_point_at_length` takes an `f32` distance. This module widens every such value to `f64` on the way out (and
//! narrows [`point_at_length`](SvgNode::point_at_length)'s `distance` argument to `f32` on the way in) purely for
//! API uniformity — it does not, and cannot, recover precision the browser never had.
//!
//! Narrowing that `distance` argument is not a plain `as f32` cast, either: see
//! [`point_at_length`](SvgNode::point_at_length)'s own doc comment for why an out-of-`f32`-range *finite* `f64` is
//! saturated rather than allowed to become an actual `f32` infinity, which the browser's binding rejects outright.
//!
//! [`SvgNode::bounding_client_rect`] is the one exception: it wraps `Element.getBoundingClientRect()`, a modern Web
//! API whose `DOMRect` has always used `double` (`f64`) fields — no widening happens there, and no precision is
//! lost at that boundary.
//!
//! In practice this rarely matters: `f32` still carries roughly 7 significant decimal digits, comfortably more than
//! any realistic SVG coordinate or pixel measurement needs. It matters if a caller feeds one of these `f64` results
//! into a calculation expecting genuine double precision (for example, accumulating many small deltas over a long
//! session) — the extra `f64` bits beyond `f32`'s precision are exactly zero, not "unknown," so such a calculation
//! degrades to `f32` accuracy rather than gaining anything from the wider Rust type.

use crate::{
    Error, SvgNode, dom_err,
    root::utils::{Matrix2D, Point, Rect, Size},
};
use wasm_bindgen::JsCast;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    /// # Bounding box
    ///
    /// Returns this element's bounding box in its own **local, user-space** coordinates, by wrapping the
    /// **no-argument** form of [`SVGGraphicsElement.getBBox()`]. This is unaffected by any transform applied to the
    /// element or its ancestors — see [`Rect`]'s own doc comment for how this differs from
    /// [`bounding_client_rect`](Self::bounding_client_rect).
    ///
    /// The underlying SVG DOM API uses 32-bit floating-point values; the returned coordinates are widened to `f64`
    /// for consistency with the rest of this crate's geometry types.
    ///
    /// # Only the object/fill bounding box
    ///
    /// The no-argument form of `getBBox()` returns the **object bounding box** — geometry only, `fill = true`,
    /// `stroke = false`, `markers = false`, `clipped = false` per the SVG specification's default
    /// `SVGBoundingBoxOptions`. A wide `stroke-width`, marker decorations (arrowheads and the like), and any
    /// `clip-path` applied to the element are **not** included, so the returned rect can be visibly smaller than
    /// everything actually painted on screen. This crate does not currently expose the options-taking overload
    /// (`getBBox(options)`) that would let a caller opt into the stroke/decorated/visible box instead — see
    /// `docs/design_notes/geometry.md` ("`bounding_box` wraps only the no-argument `getBBox`...") for why.
    ///
    /// **Performance:** this call crosses into the browser and may trigger synchronous style or layout calculation.
    /// Profile before calling it inside a hot animation or pointer-move callback — the actual cost depends on the
    /// scene's complexity and whether the browser's geometry is already current.
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
    /// The underlying SVG DOM API uses 32-bit floating-point values; the returned matrix components are widened to
    /// `f64` for consistency with the rest of this crate's geometry types.
    ///
    /// # Not generally this element's own local transform
    ///
    /// `ctm()` accumulates **every** ancestor transform between this element and its nearest viewport ancestor, not
    /// just this element's own `transform` attribute. Writing it straight back via
    /// [`set_matrix`](Self::set_matrix)/[`set_matrix_precise`](Self::set_matrix_precise) is only correct when there is
    /// no relevant transformed ancestor in that chain (equivalently: the parent-to-viewport transform is the
    /// identity matrix) — otherwise the ancestor component gets double-applied. See `docs/design_notes/geometry.md`
    /// ("`ctm`/`screen_ctm` are accumulated matrices...") for the general formula to recover just this element's own
    /// local matrix from `ctm()` readings.
    ///
    /// **Performance:** this call crosses into the browser and may trigger synchronous style or layout calculation.
    /// Profile before calling it inside a hot animation or pointer-move callback — the actual cost depends on the
    /// scene's complexity and whether the browser's geometry is already current.
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
    ///     println!("accumulated translation: ({}, {})", ctm.h_trans, ctm.v_trans);
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
    /// # Document-viewport transformation matrix
    ///
    /// Returns the accumulated transform from this element's own coordinate system all the way to the document's
    /// viewport, by wrapping [`SVGGraphicsElement.getScreenCTM()`]. Despite the DOM method's name, this is **not**
    /// physical monitor/screen coordinates — per the SVG specification it maps into the document viewport's
    /// CSS-pixel coordinates, incorporating the relevant SVG, CSS-box, and page-position transforms. Returns `None`
    /// if the element is not currently part of a rendered tree, or does not implement `SVGGraphicsElement`.
    ///
    /// The underlying SVG DOM API uses 32-bit floating-point values; the returned matrix components are widened to
    /// `f64` for consistency with the rest of this crate's geometry types.
    ///
    /// # Two different uses — do not conflate them
    ///
    /// - **Converting a point** between viewport CSS-pixel coordinates and this element's own local coordinates:
    ///   invert *this element's own* `screen_ctm()` and apply it — a parent's `screen_ctm()` is not involved at all.
    ///   `local_point = inverse(element.screen_ctm()) · viewport_point`, and conversely
    ///   `viewport_point = element.screen_ctm() · local_point`.
    /// - **Recovering this element's own local `transform`** (the matrix you would pass to
    ///   [`set_matrix`](Self::set_matrix)/[`set_matrix_precise`](Self::set_matrix_precise) to reproduce it) is a
    ///   *different* operation: it compares this element's [`ctm()`](Self::ctm) against its parent's `ctm()`, not
    ///   `screen_ctm()` at all.
    ///
    /// See `docs/design_notes/geometry.md` ("`ctm`/`screen_ctm` are accumulated matrices...") for the derivation and a worked
    /// example of each. Do not write a `screen_ctm()` reading straight back as this element's own local `transform`
    /// — that would double-apply whatever the ancestors and the page already contribute.
    ///
    /// **Performance:** this call crosses into the browser and may trigger synchronous style or layout calculation.
    /// Profile before calling it inside a hot animation or pointer-move callback — the actual cost depends on the
    /// scene's complexity and whether the browser's geometry is already current.
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
    /// // Maps this element's local coordinates into document-viewport CSS pixels; not a local transform.
    /// let viewport_ctm = rect.screen_ctm();
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
    /// The underlying SVG DOM API uses a 32-bit floating-point value; the returned length is widened to `f64` for
    /// consistency with the rest of this crate's geometry types.
    ///
    /// **Performance:** this call crosses into the browser and may trigger synchronous style or layout calculation.
    /// Profile before calling it inside a hot animation or pointer-move callback — the actual cost depends on the
    /// scene's complexity and whether the browser's geometry is already current.
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
    /// [`SVGGeometryElement.getPointAtLength()`]. `distance` is clamped to the interval from `0.0` to
    /// [`total_length()`](Self::total_length) — a **finite** `distance` outside that range does not error, it
    /// clamps to the path's start or end point, per the SVG specification.
    ///
    /// The underlying SVG DOM API uses 32-bit floating-point values on both sides of this call: the returned point's
    /// coordinates are widened to `f64` for consistency with the rest of this crate's geometry types, and `distance`
    /// itself is narrowed to `f32` before crossing into the browser — see the next section for why that narrowing
    /// needs more care than a plain cast.
    ///
    /// # `distance` is saturated to `f32` range, not just narrowed
    ///
    /// `getPointAtLength` takes an IDL `float` (32-bit), so `distance` is narrowed from `f64` before crossing into
    /// the browser. A plain `distance as f32` cast is not enough on its own: Rust saturates an out-of-`f32`-range
    /// finite `f64` (for example `f64::MAX`) to `f32::INFINITY`, and unlike an ordinary out-of-range *finite*
    /// distance, the browser's binding rejects an actually-infinite argument outright instead of clamping it —
    /// confirmed empirically (Chromium): passing `Infinity` throws `TypeError: ... non-finite`, while a large but
    /// still-finite `f32`-representable distance (e.g. `1e30`) clamps to the path start exactly as documented.
    /// To keep the clamping behaviour intuitive across the full `f64` domain, an out-of-`f32`-range *finite*
    /// `distance` is saturated to `f32::MIN`/`f32::MAX` (which then clamps to the path's end/start the same as any
    /// other out-of-range finite value) rather than being allowed to become an actual infinity.
    ///
    /// **Performance:** this call crosses into the browser and may trigger synchronous style or layout calculation.
    /// Profile before calling it inside a hot animation or pointer-move callback — the actual cost depends on the
    /// scene's complexity and whether the browser's geometry is already current.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if `distance` is `NaN` or infinite, if the browser rejects the call, or if this
    /// element does not implement `SVGGeometryElement` — see [`total_length`](Self::total_length) for which element
    /// types that excludes.
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
        if !distance.is_finite() {
            return Err(Error::Dom("getPointAtLength: distance must be finite".into()));
        }
        let geometry = self
            .inner
            .element
            .dyn_ref::<web_sys::SvgGeometryElement>()
            .ok_or_else(|| Error::Dom("getPointAtLength: element does not implement SVGGeometryElement".into()))?;
        // Saturate rather than cast directly: a finite `f64` beyond `f32`'s range would otherwise become an actual
        // `f32::INFINITY`/`f32::NEG_INFINITY`, which the browser's IDL `float` binding rejects outright instead of
        // clamping — see this method's own doc comment for the empirical confirmation.
        let distance_f32 = if distance > f64::from(f32::MAX) {
            f32::MAX
        } else if distance < f64::from(f32::MIN) {
            f32::MIN
        } else {
            distance as f32
        };
        geometry
            .get_point_at_length(distance_f32)
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
    /// **Performance:** this call crosses into the browser and may trigger synchronous style or layout calculation.
    /// Profile before calling it inside a hot animation or pointer-move callback — the actual cost depends on the
    /// scene's complexity and whether the browser's geometry is already current.
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
