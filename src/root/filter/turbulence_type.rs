// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The noise function used by [`SvgFilter::turbulence`](super::SvgFilter::turbulence)/
/// [`turbulence_xy`](super::SvgFilter::turbulence_xy), selecting the SVG `type` attribute on `<feTurbulence>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurbulenceType {
    /// The absolute value of the Perlin-noise sum (SVG default) — higher-contrast, marbled/veined noise.
    Turbulence,
    /// The signed Perlin-noise sum, unmodified — softer, smoother, cloud-like noise.
    FractalNoise,
}

impl TurbulenceType {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Turbulence => "turbulence",
            Self::FractalNoise => "fractalNoise",
        }
    }
}
