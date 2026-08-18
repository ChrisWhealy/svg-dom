// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The remap applied by one `<feFuncX>` child of
/// [`SvgFilter::component_transfer`](super::SvgFilter::component_transfer).
/// Selects both the SVG `type` attribute and the attributes that go with it.
/// This is the same one-enum-covers-a-`type`-dependent-attribute-shape pattern
/// [`ColorMatrixType`](super::ColorMatrixType) already uses for `<feColorMatrix>`.
///
/// Deliberately does not derive `Copy`, for the same reason [`ColorMatrixType`](super::ColorMatrixType) does not.
/// [`Table`](Self::Table) and [`Discrete`](Self::Discrete) each carry a `Vec<f64>`.
/// Making that implicitly copyable would encourage silent full-`Vec` clones at call sites that only needed a move
/// or a borrow.
#[derive(Debug, Clone, PartialEq)]
pub enum TransferFunction {
    /// No change to this channel — the SVG default for any channel that gets no `<feFuncX>` child at all. Only
    /// worth passing explicitly if the element itself needs to be present for some other reason.
    Identity,
    /// A piecewise-linear lookup table: the channel's `0.0`–`1.0` value selects between consecutive entries by
    /// linear interpolation.
    ///
    /// The SVG spec defines `n+1` values as `n` interpolation regions.
    /// Zero entries is explicitly defined as equivalent to [`Identity`](Self::Identity).
    /// A *single* entry leaves `n = 0`, with no region for the interpolation formula to apply to.
    /// Its behaviour is therefore unspecified, rather than "a constant function".
    /// [`component_transfer`](super::SvgFilter::component_transfer) returns
    /// [`Error::InvalidTransferFunction`](crate::Error::InvalidTransferFunction) for exactly this case.
    ///
    /// For a portable constant transfer function, supply the same value twice instead: `Table(vec![0.5, 0.5])`.
    Table(Vec<f64>),
    /// A stepped lookup table: the channel's value selects one entry outright, per the SVG "discrete" stepping
    /// formula, rather than interpolating between two neighbours the way [`Table`](Self::Table) does. Produces a
    /// posterised/quantised look.
    ///
    /// Unlike [`Table`](Self::Table), a *single* value here is well-defined: every input maps to that one entry, a
    /// constant function.
    /// An *empty* list is not well-defined, though.
    /// The stepping formula divides by the value count and indexes into the list with the result.
    /// Both of these are undefined for zero values.
    /// The spec also gives the empty list no identity fallback, unlike `Table`.
    /// At least one value is required, or
    /// [`component_transfer`](super::SvgFilter::component_transfer) returns
    /// [`Error::InvalidTransferFunction`](crate::Error::InvalidTransferFunction).
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
