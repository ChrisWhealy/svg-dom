// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A colour channel used by a `<feFuncX>` child, built via
/// [`SvgFilter::component_transfer`](super::SvgFilter::component_transfer).
/// Also used by `<feDisplacementMap>`'s `xChannelSelector`/`yChannelSelector` attributes; see
/// [`SvgFilter::displacement_map`](super::SvgFilter::displacement_map).
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
    /// started fully transparent. If the function maps `0.0` to an output above `0.0`, every previously transparent
    /// pixel becomes visible too. This includes pixels that were never part of the shape.
    ///
    /// This could happen for example with:
    ///
    /// * [`TransferFunction::Linear`](super::TransferFunction::Linear) with a positive `intercept`
    /// * [`TransferFunction::Gamma`](super::TransferFunction::Gamma) with a positive `offset`
    /// * [`TransferFunction::Table`](super::TransferFunction::Table) or
    ///   [`TransferFunction::Discrete`](super::TransferFunction::Discrete) whose first entry is above `0.0`
    ///
    /// When `in` is `SourceGraphic`, the default for the first primitive, the primitive subregion is the whole
    /// filter region. This appears as a rectangular halo or background fill across that entire region.
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

    /// The single-letter SVG keyword used by `<feDisplacementMap>`'s `xChannelSelector`/`yChannelSelector`
    /// attributes (see [`SvgFilter::displacement_map`](super::SvgFilter::displacement_map)).
    /// This names the same four channels as this enum's own variants, just written as a bare letter rather
    /// than an element tag.
    pub fn selector_str(self) -> &'static str {
        match self {
            Self::Red => "R",
            Self::Green => "G",
            Self::Blue => "B",
            Self::Alpha => "A",
        }
    }
}
