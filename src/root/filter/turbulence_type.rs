// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The noise function used by [`SvgFilter::turbulence`](super::SvgFilter::turbulence)/
/// [`turbulence_xy`](super::SvgFilter::turbulence_xy), selecting the SVG `type` attribute on `<feTurbulence>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurbulenceType {
    /// The sum of the absolute values of the Perlin-noise contribution from each octave (SVG default) —
    /// higher-contrast, marbled/veined noise. Taking the absolute value *per octave*, before summing, is not the
    /// same as summing first and taking the absolute value of the total: the per-octave version cannot have
    /// positive and negative octaves cancel each other out, which is what gives this variant its higher contrast
    /// relative to [`FractalNoise`](Self::FractalNoise).
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
