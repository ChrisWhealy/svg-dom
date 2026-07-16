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
    /// # `transform="matrix(a, b, c, d, e, f)"`
    ///
    /// Sets the transform directly as a 2D affine matrix — see [`Matrix2D`] for what each field means and how they
    /// map onto the SVG function's arguments `a, b, c, d, e, f`.
    ///
    /// `matrix`'s fields are formatted at two different precisions, matching each field's typical role rather than
    /// treating all six as a single, undifferentiated list:
    /// - `h_scale`, `v_scale`, `h_skew`, `v_skew` (the linear/rotation/scale part) at three decimal places, the same
    ///   precision used by [`set_scale`](Self::set_scale), since they are typically small dimensionless ratios where
    ///   that precision matters
    /// - `h_trans`, `v_trans` (the translation part) at one decimal place, the same precision used by
    ///   [`set_translate`](Self::set_translate), since they are typically pixel-scale coordinates.
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
    /// # Arbitrary transform via `format_args!`
    ///
    /// Lower-level escape hatch for transform shapes the typed helpers above do not cover (skews, matrices, or several
    /// chained operations). It still formats, but writes into the reused `scratch` buffer rather than allocating a fresh
    /// `String` the way `format!` would.
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
