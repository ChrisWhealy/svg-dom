// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Which colour channel a `<feFuncX>` child (built via
/// [`SvgFilter::component_transfer`](super::SvgFilter::component_transfer)) applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// `<feFuncR>` — the red channel.
    Red,
    /// `<feFuncG>` — the green channel.
    Green,
    /// `<feFuncB>` — the blue channel.
    Blue,
    /// `<feFuncA>` — the alpha channel. Remapping this alone is the standard way to fade or clip transparency
    /// without touching colour at all.
    ///
    /// ***⚠️ A function with `f(0) > 0` can paint a background across the whole filter region*** —
    /// [`component_transfer`](super::SvgFilter::component_transfer) runs on every pixel, including ones that
    /// started fully transparent. If the function mapped to this channel gives `0.0` an output above `0.0` every
    /// previously-transparent pixel becomes visible too — not just the ones that were already part of the shape.
    ///
    /// This could happen for example with:
    ///
    /// * [`TransferFunction::Linear`](super::TransferFunction::Linear) with a positive `intercept`
    /// * [`TransferFunction::Gamma`](super::TransferFunction::Gamma) with a positive `offset`
    /// * [`TransferFunction::Table`](super::TransferFunction::Table) or
    ///   [`TransferFunction::Discrete`](super::TransferFunction::Discrete) whose first entry is above `0.0`
    ///
    /// When `in` is `SourceGraphic` (the default for the first primitive), the primitive subregion is the whole filter
    /// region, so this shows up as a rectangular halo or background fill across that entire region.
    ///
    /// Do not give this channel a function with `f(0) > 0` unless a background fill across the whole region is the
    /// intended effect.
    Alpha,
}

impl Channel {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Self::Red => "feFuncR",
            Self::Green => "feFuncG",
            Self::Blue => "feFuncB",
            Self::Alpha => "feFuncA",
        }
    }
}
