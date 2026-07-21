// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The remap applied by one `<feFuncX>` child of
/// [`SvgFilter::component_transfer`](super::SvgFilter::component_transfer), selecting both the SVG `type` attribute
/// and the attributes that go with it — the same one-enum-covers-a-`type`-dependent-attribute-shape
/// [`ColorMatrixType`](super::ColorMatrixType) already uses for `<feColorMatrix>`.
///
/// Deliberately does not derive `Copy`, for the same reason [`ColorMatrixType`](super::ColorMatrixType) does not:
/// [`Table`](Self::Table) and [`Discrete`](Self::Discrete) each carry a `Vec<f64>`, and making that implicitly
/// copyable would encourage silent full-`Vec` clones at call sites when only a move or borrow was needed.
#[derive(Debug, Clone, PartialEq)]
pub enum TransferFunction {
    /// No change to this channel — the SVG default for any channel that gets no `<feFuncX>` child at all. Only
    /// worth passing explicitly if the element itself needs to be present for some other reason.
    Identity,
    /// A piecewise-linear lookup table: the channel's `0.0`–`1.0` value selects between consecutive entries by
    /// linear interpolation.
    ///
    /// The SVG spec defines `n+1` values as `n` interpolation regions; zero entries is explicitly defined as
    /// equivalent to [`Identity`](Self::Identity), but a *single* entry leaves `n = 0` with no region for the
    /// interpolation formula to apply to, so its behaviour is unspecified rather than "a constant function" — see
    /// [`Error::InvalidTransferTable`](crate::Error::InvalidTransferTable), which
    /// [`component_transfer`](super::SvgFilter::component_transfer) returns for exactly this case.
    ///
    /// For a portable constant transfer function, supply the same value twice instead: `Table(vec![0.5, 0.5])`.
    Table(Vec<f64>),
    /// A stepped lookup table: the channel's value selects one entry outright, per the SVG "discrete" stepping
    /// formula, rather than interpolating between two neighbours the way [`Table`](Self::Table) does. Produces a
    /// posterised/quantised look.
    Discrete(Vec<f64>),
    /// A linear remap: `slope * C + intercept`, applied to the channel's `0.0`–`1.0` value `C`.
    Linear {
        /// Multiplies the channel value.
        slope: f64,
        /// Added after the multiply.
        intercept: f64,
    },
    /// A gamma remap: `amplitude * C^exponent + offset`, applied to the channel's `0.0`–`1.0` value `C`. The
    /// standard way to gamma-correct or contrast-adjust a channel — `exponent < 1.0` brightens midtones,
    /// `exponent > 1.0` darkens them.
    Gamma {
        /// Scales the result of the exponentiation.
        amplitude: f64,
        /// The power `C` is raised to.
        exponent: f64,
        /// Added after scaling.
        offset: f64,
    },
}

impl TransferFunction {
    pub(super) fn type_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Table(_) => "table",
            Self::Discrete(_) => "discrete",
            Self::Linear { .. } => "linear",
            Self::Gamma { .. } => "gamma",
        }
    }
}
