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
