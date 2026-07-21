// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The Porter-Duff (or Photoshop-style) compositing operator for [`SvgFilter::composite`](super::SvgFilter::composite),
/// controlling how the `in` and `in2` inputs combine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeOperator {
    /// `in` painted over `in2` (SVG default). Neither input is clipped to the other's shape.
    Over,
    /// Only the part of `in` that overlaps `in2` is kept — the standard way to tint a mask, e.g. compositing a
    /// flood colour "in" a blurred alpha silhouette to colour a drop shadow.
    In,
    /// Only the part of `in` that does *not* overlap `in2` is kept.
    Out,
    /// The part of `in` that overlaps `in2`, painted on top of `in2`.
    Atop,
    /// The non-overlapping parts of both inputs; the overlap is removed from both.
    Xor,
    /// The arithmetic sum of both inputs, clamped to fully opaque — brightens rather than blends.
    Lighter,
    /// A per-pixel weighted sum `k1*i1*i2 + k2*i1 + k3*i2 + k4`, controlled by the `k1`–`k4` attributes (not
    /// wrapped by a named parameter here; set them via the returned [`SvgNode`](crate::SvgNode)'s
    /// [`set_attr`](crate::SvgNode::set_attr) — this is the one operator
    /// [`composite`](super::SvgFilter::composite) does not fully configure on its own).
    ///
    /// ***⚠️ `k1`–`k4` arguments all default to `0`*** — [`composite`](super::SvgFilter::composite) does not write
    /// them, and the SVG initial value for each is `0`. Selecting `Arithmetic` and stopping there evaluates to
    /// `0*i1*i2 + 0*i1 + 0*i2 + 0` for every pixel, i.e. **transparent black**, not a blend of the two inputs.
    ///
    /// Always set all four coefficients you need immediately after calling `composite` with this operator:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::CompositeOperator};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("blend")?;
    /// flt.gaussian_blur(6.0)?.set_attrs([("in", "SourceGraphic"), ("result", "blur")])?;
    ///
    /// // A straightforward 50/50 blend of the sharp source and its blurred copy: k2 = k3 = 0.5, k1 = k4 = 0.
    /// flt.composite("blur", CompositeOperator::Arithmetic)?.set_attrs([
    ///     ("in", "SourceGraphic"),
    ///     ("k1", "0"), ("k2", "0.5"), ("k3", "0.5"), ("k4", "0"),
    /// ])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    Arithmetic,
}

impl CompositeOperator {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Over => "over",
            Self::In => "in",
            Self::Out => "out",
            Self::Atop => "atop",
            Self::Xor => "xor",
            Self::Lighter => "lighter",
            Self::Arithmetic => "arithmetic",
        }
    }
}
