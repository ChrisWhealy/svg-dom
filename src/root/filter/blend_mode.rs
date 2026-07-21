// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The blend mode for [`SvgFilter::blend`](super::SvgFilter::blend), controlling how the `in` and `in2` inputs
/// blend.
///
/// Selects one of the sixteen standard `<blend-mode>` keywords (`normal` plus fifteen separable/non-separable
/// modes) shared by CSS compositing and SVG `feBlend` — not the full CSS `mix-blend-mode` value set, which also
/// accepts two CSS-only, property-specific modes (`plus-lighter`/`plus-darker`) this enum does not offer.
///
/// ***IMPORTANT*** SVG filter primitives operate in the `linearRGB` colour space by default, unlike CSS
/// `mix-blend-mode` and most image editors, which operate in `sRGB`. The same [`BlendMode`] can therefore produce a
/// visibly different result here than the "same" mode elsewhere, even with identical input colours. Set
/// `color-interpolation-filters="sRGB"` on the `<filter>` element (via [`SvgFilter::set_attr`](super::SvgFilter::set_attr))
/// — or on an individual primitive's own [`SvgNode`](crate::SvgNode) via
/// [`set_attr`](crate::SvgNode::set_attr) to override it for just that one primitive — when an sRGB-space result is
/// required to match CSS or an image editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// `in` painted over `in2` with no blending (SVG default) — identical to
    /// [`CompositeOperator::Over`](super::CompositeOperator::Over).
    Normal,
    /// Multiplies channel values: black stays black, white leaves the other input unchanged. Always darkens or
    /// leaves unchanged, never lightens.
    Multiply,
    /// Multiplies the inverted channel values, then inverts the result — the inverse of [`Multiply`](Self::Multiply).
    /// Always lightens or leaves unchanged, never darkens.
    Screen,
    /// Keeps the darker of the two colours, per channel.
    Darken,
    /// Keeps the lighter of the two colours, per channel.
    Lighten,
    /// [`Multiply`](Self::Multiply) or [`Screen`](Self::Screen) depending on `in2`, increasing contrast.
    Overlay,
    /// Brightens `in2` to reflect `in`'s colour; has no effect where `in2` is white.
    ColorDodge,
    /// Darkens `in2` to reflect `in`'s colour; has no effect where `in2` is black.
    ColorBurn,
    /// Like [`Overlay`](Self::Overlay) but with the roles of `in`/`in2` swapped: `Multiply` where `in` is dark,
    /// `Screen` where `in` is light — the effect of shining a harsh spotlight through `in`.
    HardLight,
    /// A softer version of [`HardLight`](Self::HardLight), closer to a diffuse spotlight than a harsh one.
    SoftLight,
    /// The per-channel absolute difference between the two colours; identical colours become black.
    Difference,
    /// Like [`Difference`](Self::Difference), but lower-contrast — pairing with white inverts, pairing with black
    /// has no effect.
    Exclusion,
    /// Takes the hue of `in`, and the saturation and luminosity of `in2`.
    Hue,
    /// Takes the saturation of `in`, and the hue and luminosity of `in2`.
    Saturation,
    /// Takes the hue and saturation of `in` (i.e. its colour), and the luminosity of `in2`.
    Color,
    /// Takes the luminosity of `in`, and the hue and saturation of `in2`.
    Luminosity,
}

impl BlendMode {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Multiply => "multiply",
            Self::Screen => "screen",
            Self::Darken => "darken",
            Self::Lighten => "lighten",
            Self::Overlay => "overlay",
            Self::ColorDodge => "color-dodge",
            Self::ColorBurn => "color-burn",
            Self::HardLight => "hard-light",
            Self::SoftLight => "soft-light",
            Self::Difference => "difference",
            Self::Exclusion => "exclusion",
            Self::Hue => "hue",
            Self::Saturation => "saturation",
            Self::Color => "color",
            Self::Luminosity => "luminosity",
        }
    }
}
