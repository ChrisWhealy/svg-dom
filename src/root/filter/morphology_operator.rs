// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Defines the direction [`SvgFilter::morphology`](super::SvgFilter::morphology)/
/// [`morphology_xy`](super::SvgFilter::morphology_xy) take a component-wise minimum or maximum in, selecting the
/// SVG `operator` attribute on `<feMorphology>`.
///
/// Both variants apply across all four premultiplied R/G/B/A channels of whatever `in` actually is.
/// The "opaque regions"/silhouette framing below describes the common case of passing `SourceAlpha`, where alpha is
/// the only channel with anything to shrink or grow.
/// See [`morphology`](super::SvgFilter::morphology)'s own doc comment for what happens against `SourceGraphic`
/// instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphologyOperator {
    /// Takes the per-pixel minimum over `radius` (SVG default).
    /// Against `SourceAlpha`, this shrinks or thins opaque regions inward.
    /// It is the standard way to thin an outline, or to shrink a mask slightly inward before reusing it elsewhere.
    Erode,
    /// Takes the per-pixel maximum over `radius`.
    /// Against `SourceAlpha`, this grows or thickens opaque regions outward.
    /// It is the standard way to bolden a thin outline, or to fatten a mask before use.
    /// For example, use it before a blur so the source leaves no visible gap at the original edge.
    Dilate,
}

impl MorphologyOperator {
    /// Returns the exact keyword this operator's own `operator` attribute on `<feMorphology>` needs.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Erode => "erode",
            Self::Dilate => "dilate",
        }
    }
}
