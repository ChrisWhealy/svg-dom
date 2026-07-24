mod blend;
mod color_matrix;
mod component_transfer;
mod composite;
mod convolve_matrix;
mod displacement_map;
mod drop_shadow;
mod flood;
mod gaussian_blur;
mod image;
mod merge;
mod morphology;
mod offset;
mod tile;
mod turbulence;

use std::fmt;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Formats a slice of `f64` space-separated, straight into the caller's `fmt::Formatter` — used to write `tableValues`
/// ([`component_transfer`](super::SvgFilter::component_transfer)) and `kernelMatrix`
/// ([`convolve_matrix`](super::SvgFilter::convolve_matrix) or
/// [`convolve_matrix_xy`](super::SvgFilter::convolve_matrix_xy)) through
/// [`SvgAttrs::display_element`](crate::root::attrs::SvgAttrs::display_element)'s scratch buffer with no intermediate
/// `String`/`Vec` allocation — the same technique this crate's internal `write_points` helper uses for the `points`
/// attribute.
pub(super) struct SpaceSeparated<'a>(pub(super) &'a [f64]);

impl fmt::Display for SpaceSeparated<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{v}")?;
        }
        Ok(())
    }
}
