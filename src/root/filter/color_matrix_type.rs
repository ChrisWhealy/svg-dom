// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The colour transform applied by [`SvgFilter::color_matrix`](super::SvgFilter::color_matrix), selecting both the
/// SVG `type` attribute and the shape of the `values` attribute that goes with it.
///
/// This enum deliberately does not implement `Copy` (unlike [`CompositeOperator`](super::CompositeOperator), which
/// is small enough that copying is free): the [`Matrix`](Self::Matrix) variant carries 160 bytes of `f64`s, and
/// making that implicitly copyable would encourage silent full-array copies at call sites when only a move or
/// borrow was needed.
#[derive(Debug, Clone, PartialEq)]
pub enum ColorMatrixType {
    /// A full 4x5 colour transform matrix, applied to each pixel's `[R, G, B, A]` as `M · [R, G, B, A, 1]ᵀ`.
    ///
    /// Deliberately a fixed-size `[f64; 20]` rather than a `Vec<f64>` or `&[f64]`: the SVG `values` attribute for
    /// this type is defined as exactly 20 numbers, no more and no fewer, so a matrix with the wrong element count
    /// cannot be constructed at all, rather than failing at the DOM boundary or silently truncating/padding.
    Matrix([f64; 20]),
    /// Adjusts colour saturation. `1.0` is the identity (no change); `0.0` produces greyscale.
    Saturate(f64),
    /// Rotates hue by the given angle in degrees around the colour circle.
    HueRotate(f64),
    /// Converts to greyscale using each pixel's luminance as its resulting alpha (zeroing RGB) — derives a mask
    /// from perceived brightness rather than the alpha channel
    /// [`gaussian_blur`](super::SvgFilter::gaussian_blur) and friends use for a shadow silhouette.
    LuminanceToAlpha,
}

impl ColorMatrixType {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            Self::Matrix(_) => "matrix",
            Self::Saturate(_) => "saturate",
            Self::HueRotate(_) => "hueRotate",
            Self::LuminanceToAlpha => "luminanceToAlpha",
        }
    }
}
