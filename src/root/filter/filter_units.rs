// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls which coordinate space the filter region (`x`, `y`, `width`, `height`) or a primitive's own coordinate
/// attributes are expressed in.
///
/// Used for both the `filterUnits` and `primitiveUnits` attributes.
/// Passed to [`SvgFilter::set_filter_units`](super::SvgFilter::set_filter_units) and
/// [`SvgFilter::set_primitive_units`](super::SvgFilter::set_primitive_units).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterUnits {
    /// Values are expressed in the same coordinate space as the element that references the filter.
    /// SVG default for `primitiveUnits`.
    UserSpaceOnUse,
    /// Values are expressed as fractions of the referencing element's bounding box — `(0, 0)` maps to the top-left
    /// corner and `(1, 1)` maps to the bottom-right corner.
    /// SVG default for `filterUnits`.
    ObjectBoundingBox,
}

impl FilterUnits {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::UserSpaceOnUse => "userSpaceOnUse",
            Self::ObjectBoundingBox => "objectBoundingBox",
        }
    }
}
