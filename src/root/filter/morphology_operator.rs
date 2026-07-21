// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Defines the direction in which [`SvgFilter::morphology`](super::SvgFilter::morphology) and
/// [`morphology_xy`](super::SvgFilter::morphology_xy) grow or shrink the input's opaque regions, selecting the SVG
/// `operator` attribute on `<feMorphology>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphologyOperator {
    /// Shrinks/thins opaque regions (SVG default) — the standard way to thin an outline, or to shrink a mask slightly
    /// inward before reusing it elsewhere.
    Erode,
    /// Grows/thickens opaque regions — the standard way to bolden a thin outline, or fatten a mask before using it as,
    /// for example, a blur source that should not leave a visible gap at the original edge.
    Dilate,
}

impl MorphologyOperator {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Erode => "erode",
            Self::Dilate => "dilate",
        }
    }
}
