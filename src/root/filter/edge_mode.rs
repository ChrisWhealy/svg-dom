// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Selects how [`SvgFilter::convolve_matrix`](super::SvgFilter::convolve_matrix) and
/// [`convolve_matrix_xy`](super::SvgFilter::convolve_matrix_xy) extend the input image with virtual pixel values
/// beyond its edge.
/// The kernel needs these values when it overhangs the border.
/// This is the SVG `edgeMode` attribute on `<feConvolveMatrix>`.
///
/// `<feGaussianBlur>` also has an `edgeMode` attribute sharing this same three-keyword vocabulary.
/// Its SVG default (`None`) differs from `<feConvolveMatrix>`'s (`Duplicate`).
/// This enum only fixes which values are legal, not which one any particular element starts with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeMode {
    /// This is the SVG default for `<feConvolveMatrix>`.  It extends the input beyond its border by repeating the
    /// colour values found at the edge.
    ///
    /// The usual choice: a sharpen or edge-detect kernel reads a plausible continuation of the image rather than a
    /// hard, artificial boundary.
    Duplicate,
    /// Extends the input by wrapping around to the colour values on the opposite edge, as if the image is tiled.
    /// Suited to a kernel applied to an already seamlessly tileable input, for example a `feTurbulence` pattern
    /// with `stitchTiles="stitch"`.
    /// On ordinary artwork it reads as an odd seam, since the far edge is rarely a natural continuation of the
    /// near one.
    Wrap,
    /// Extends the input with fully transparent black (`R = G = B = A = 0`) beyond its border.
    /// Produces a visible darkened or faded fringe near the edges for any kernel that would otherwise sample
    /// duplicated or wrapped colour there.
    /// The convolution now blends real pixels with true zeros instead.
    None,
}

impl EdgeMode {
    /// Returns the exact keyword this mode's own `edgeMode` attribute needs.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Duplicate => "duplicate",
            Self::Wrap => "wrap",
            Self::None => "none",
        }
    }
}
