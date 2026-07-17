//! Allocation-reusing `transform` setters for high-frequency updates.
//!
//! Building a transform string with `format!` allocates a fresh heap `String` on every call. That is harmless for
//! one-off element creation, but transforms are among the most common *high-frequency* updates — dragging, sliders,
//! knobs, pan/zoom, follow-the-pointer cursors, resize/selection handles — where a handler may fire dozens or hundreds
//! of times per second. Each of those `format!`s allocates a string, formats into it, hands it to `set_attr`, and drops
//! it again.
//!
//! The browser still has to receive the new attribute value, so the DOM write cannot be avoided. What *can* be avoided
//! is the repeated Rust-side allocation: these helpers take a caller-owned `&mut String` scratch buffer, clear it, write
//! the new transform into it, and reuse the same backing allocation on the next call. No new allocation happens unless
//! the formatted text grows beyond the buffer's current capacity.
//!
//! The scratch buffer is deliberately **not** stored inside [`SvgNode`]: passive geometry nodes (the common case) would
//! then carry formatting state they never use. Keeping the buffer external keeps those nodes small and lets hot paths
//! explicitly opt in to allocation reuse.
//!
//! This complements [`AnimationLoop`](crate::AnimationLoop): that reduces per-frame allocation *inside animation
//! callbacks*, whereas event-driven drag/pointer handlers do not run through the animation loop and so need their own
//! reusable buffer.
//!
//! # Example
//!
//! ```rust,no_run
//! use svg_dom::{root::utils::{Point, Size}, SvgRoot};
//! let svg  = SvgRoot::attach("diagram")?;
//! let card = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
//!
//! // One buffer, reused for every move — no per-event heap allocation.
//! let mut transform_buf = String::new();
//! card.set_translate(&mut transform_buf, 12.0, 34.0)?;
//! card.set_translate(&mut transform_buf, 13.0, 35.0)?;
//! Ok::<(), svg_dom::Error>(())
//! ```

use std::fmt::Write;

use crate::{Error, SvgNode, root::utils::Matrix2D};

impl SvgNode {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # `transform="translate(x, y)"`
    ///
    /// Sets a translation transform, formatting `x` and `y` to one decimal place into the supplied scratch buffer.
    ///
    /// Reuse the same buffer across calls to avoid per-update heap allocation on hot paths such as dragging or
    /// follow-the-pointer movement.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let node = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    /// let mut buf = String::new();
    /// node.set_translate(&mut buf, 100.0, 50.0)?; // transform="translate(100.0, 50.0)"
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_translate(&self, scratch: &mut String, x: f64, y: f64) -> Result<(), Error> {
        scratch.clear();
        write!(scratch, "translate({x:.1}, {y:.1})")?;
        self.set_attr("transform", scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # `transform="rotate(angle)"`
    ///
    /// Sets a rotation (in degrees) about the element's origin, formatted to one decimal place into `scratch`.
    pub fn set_rotate(&self, scratch: &mut String, angle: f64) -> Result<(), Error> {
        scratch.clear();
        write!(scratch, "rotate({angle:.1})")?;
        self.set_attr("transform", scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # `transform="rotate(angle, cx, cy)"`
    ///
    /// Sets a rotation (in degrees) about the point `(cx, cy)`, all formatted to one decimal place into `scratch`.
    pub fn set_rotate_about(&self, scratch: &mut String, angle: f64, cx: f64, cy: f64) -> Result<(), Error> {
        scratch.clear();
        write!(scratch, "rotate({angle:.1}, {cx:.1}, {cy:.1})")?;
        self.set_attr("transform", scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # `transform="scale(s)"`
    ///
    /// Sets a uniform scale, formatted to three decimal places into `scratch`.
    pub fn set_scale(&self, scratch: &mut String, scale: f64) -> Result<(), Error> {
        scratch.clear();
        write!(scratch, "scale({scale:.3})")?;
        self.set_attr("transform", scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # `transform="scale(x, y)"`
    ///
    /// Sets a non-uniform scale, with each factor formatted to three decimal places into `scratch`.
    pub fn set_scale_xy(&self, scratch: &mut String, x: f64, y: f64) -> Result<(), Error> {
        scratch.clear();
        write!(scratch, "scale({x:.3}, {y:.3})")?;
        self.set_attr("transform", scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # `transform="translate(tx, ty) scale(s)"`
    ///
    /// Sets a combined translate-then-scale transform, the common shape for pan/zoom code. The translation is formatted
    /// to one decimal place and the scale to three, into `scratch`.
    pub fn set_translate_scale(&self, scratch: &mut String, tx: f64, ty: f64, scale: f64) -> Result<(), Error> {
        scratch.clear();
        write!(scratch, "translate({tx:.1}, {ty:.1}) scale({scale:.3})")?;
        self.set_attr("transform", scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # `transform="matrix(a, b, c, d, e, f)"`, quantised
    ///
    /// Sets the transform directly as a 2D affine matrix — see [`Matrix2D`] for what each field means and how they
    /// map onto the SVG function's arguments `a, b, c, d, e, f`.
    ///
    /// `matrix`'s fields are formatted at two different fixed precisions, matching each field's typical role rather
    /// than treating all six as a single, undifferentiated list:
    /// - `h_scale`, `v_scale`, `h_skew`, `v_skew` (the linear/rotation/scale part) at three decimal places, the same
    ///   precision used by [`set_scale`](Self::set_scale)
    /// - `h_trans`, `v_trans` (the translation part) at one decimal place, the same precision used by
    ///   [`set_translate`](Self::set_translate).
    ///
    /// # ⚠️ Caveat ⚠️
    ///
    /// The other named transform helpers ([`set_translate`](Self::set_translate), [`set_rotate`](Self::set_rotate),
    /// [`set_scale`](Self::set_scale), ...) each use a fixed precision appropriate for common interactive updates,
    /// which must be understood to be a sensible default, not a guarantee.
    ///
    /// A caller who genuinely needs different precision should use [`set_transform_fmt`](Self::set_transform_fmt).
    ///
    /// A matrix's linear coefficients need extra care beyond that, because they have a failure mode the other helpers
    /// structurally cannot avoid: their rounding error is multiplied by whatever coordinate the matrix uses for
    /// transforms.  So the error scales with the geometry rather than staying fixed the way a rounded translation or
    /// rotation angle does:
    ///
    /// - When the rounding threshold is set to 3dp, a rotation's sine term rounds to `0.000` below about `0.0286°`
    ///   (`sin(0.0286°) ≈ 0.0005`), so this rounding error can cause a very slow matrix-driven rotation to visibly
    ///   stick and then jump, rather than moving smoothly.
    ///
    /// - Each linear coefficient's rounding error (up to `0.0005`) is applied to whatever coordinate the matrix acts
    ///   upon, so at large coordinates the resulting positional error grows with them.  For example at `x = y =
    ///   10_000`, the error can exceed 10 user units.
    ///
    /// Prefer [`set_matrix_precise`](Self::set_matrix_precise) instead for coefficients obtained from a `DOMMatrix`,
    /// another library, or a coordinate-space conversion where the resulting matrix is intended to become this
    /// element's local transform — or for a slow/precise rotation, where `set_matrix`'s quantisation can round a small
    /// sine term in a way that introduces visual artefacts or makes the rotation disappear entirely.
    ///
    /// If the coefficients come from [`ctm`](Self::ctm) or [`screen_ctm`](Self::screen_ctm) rather than a
    /// local-space source, read that method's doc comment first — both are accumulated coordinate-conversion
    /// matrices, not generally this element's own local transform, so writing either back here unmodified would
    /// double-apply whatever ancestor (and, for `screen_ctm`, page) transform is already contributed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Matrix2D, Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let node = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    /// let mut buf = String::new();
    /// // A horizontal shear: each point's x shifts by 0.3 times its y — not expressible via translate/rotate/scale.
    /// node.set_matrix(
    ///     &mut buf,
    ///     Matrix2D { h_scale: 1.0, v_scale: 1.0, h_skew: 0.3, v_skew: 0.0, h_trans: 0.0, v_trans: 0.0 },
    /// )?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_matrix(&self, scratch: &mut String, m: Matrix2D) -> Result<(), Error> {
        scratch.clear();
        let Matrix2D {
            h_scale,
            v_scale,
            h_skew,
            v_skew,
            h_trans,
            v_trans,
        } = m;
        write!(
            scratch,
            "matrix({h_scale:.3}, {v_skew:.3}, {h_skew:.3}, {v_scale:.3}, {h_trans:.1}, {v_trans:.1})"
        )?;
        self.set_attr("transform", scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # `transform="matrix(a, b, c, d, e, f)"`, exact
    ///
    /// Sets the transform directly as a 2D affine matrix, exactly as [`set_matrix`](Self::set_matrix) does, but formats
    /// all six [`Matrix2D`] fields with Rust's shortest round-trip `Display` representation instead of `set_matrix`'s
    /// fixed three/one decimal places.
    ///
    /// Prefer this over `set_matrix` for coefficients obtained from a `DOMMatrix`, another library, or a
    /// coordinate-space conversion where the resulting matrix is intended to become this element's local transform
    /// — or for a slow/precise rotation where `set_matrix`'s quantisation can round a small sine term to zero thus
    /// introducing visual artefacts or making the rotation disappear entirely.
    ///
    /// See `set_matrix`'s own doc comment for the specific failure modes this avoids.
    ///
    /// Prefer `set_matrix` when its quantisation is acceptable (typically hand-authored values, a rotation angle, a
    /// pixel offset, or a scale factor where a difference of a few thousandths is insignificant) and limiting
    /// coefficient precision is itself desirable; for example, to keep the attribute value short and easy to read in
    /// devtools.
    ///
    /// This is not the same as a general size advantage: `set_matrix_precise` can produce the *shorter* string for
    /// round-number coefficients, since `set_matrix` always writes three or one decimal places even for `0` — an
    /// identity matrix is `matrix(1, 0, 0, 1, 0, 0)` via `set_matrix_precise` but
    /// `matrix(1.000, 0.000, 0.000, 1.000, 0.0, 0.0)` via `set_matrix`.
    ///
    /// Choose based on whether the original `f64` values need to survive serialisation exactly, not on an assumption
    /// about which output is shorter.
    ///
    /// If the coefficients come from [`ctm`](Self::ctm) or [`screen_ctm`](Self::screen_ctm) rather than a
    /// local-space source, read that method's doc comment first — both are accumulated coordinate-conversion
    /// matrices, not generally this element's own local transform, so writing either back here unmodified would
    /// double-apply whatever ancestor (and, for `screen_ctm`, page) transform is already contributed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Matrix2D, Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let node = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    /// let mut buf = String::new();
    /// // A near-zero rotation: set_matrix would round sin(0.01°) to 0.000 and lose it entirely.
    /// let angle = 0.01_f64.to_radians();
    /// node.set_matrix_precise(
    ///     &mut buf,
    ///     Matrix2D {
    ///         h_scale: angle.cos(),
    ///         v_skew: angle.sin(),
    ///         h_skew: -angle.sin(),
    ///         v_scale: angle.cos(),
    ///         h_trans: 0.0,
    ///         v_trans: 0.0,
    ///     },
    /// )?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_matrix_precise(&self, scratch: &mut String, m: Matrix2D) -> Result<(), Error> {
        scratch.clear();
        let Matrix2D {
            h_scale,
            v_scale,
            h_skew,
            v_skew,
            h_trans,
            v_trans,
        } = m;
        write!(
            scratch,
            "matrix({h_scale}, {v_skew}, {h_skew}, {v_scale}, {h_trans}, {v_trans})"
        )?;
        self.set_attr("transform", scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Arbitrary transform via `format_args!`
    ///
    /// Lower-level escape hatch for transform shapes that the typed helpers above cannot cover (e.g. skews, or matrices
    /// requiring custom precision, or several chained operations). It still formats, but writes into the reused
    /// `scratch` buffer rather than allocating a fresh `String` the way `format!` would.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let node = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    /// let mut buf = String::new();
    /// let (x, y, angle) = (10.0, 20.0, 45.0);
    /// node.set_transform_fmt(
    ///     &mut buf,
    ///     format_args!("translate({x:.1}, {y:.1}) rotate({angle:.1})"),
    /// )?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_transform_fmt(&self, scratch: &mut String, args: std::fmt::Arguments<'_>) -> Result<(), Error> {
        scratch.clear();
        scratch.write_fmt(args)?;
        self.set_attr("transform", scratch)
    }
}
