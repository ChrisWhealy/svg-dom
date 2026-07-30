use svg_dom::{Error, SvgFilter};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// One file per demo, each exporting its own `fn demo()` — the same split `texts/mod.rs` uses, and for the same
// reason: `demo_gallery!`'s `$path:path` grammar accepts any nesting depth, so `paint::demo_filter::demo` works
// exactly like `texts::demo_text::demo`, and DEMO_SOURCE_FILES (lib.rs) keys each file by its own module path so
// two files can each define `fn demo()` without colliding on lookup.
//
// Unlike `structure`, this module does have shared helpers: `widen_filter_region`/`exact_filter_region` below are
// used by more than one demo file (`demo_filter`, `demo_turbulence`, `demo_morphology`, `demo_fe_image`,
// `demo_fe_tile`), so they live here rather than being duplicated per file. Plain private `fn`s, not `pub(crate)`:
// nothing outside `paint` needs them, and a private item defined here is already visible to every descendant
// module (i.e. every file below), so no `pub` qualifier is needed for `super::widen_filter_region(f)` to resolve.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) mod demo_blend;
pub(crate) mod demo_clip_path;
pub(crate) mod demo_color_matrix;
pub(crate) mod demo_component_transfer;
pub(crate) mod demo_convolve_matrix;
pub(crate) mod demo_fe_image;
pub(crate) mod demo_fe_tile;
pub(crate) mod demo_filter;
pub(crate) mod demo_light_sources;
pub(crate) mod demo_lighting;
pub(crate) mod demo_linear_gradient;
pub(crate) mod demo_mask;
pub(crate) mod demo_morphology;
pub(crate) mod demo_pattern;
pub(crate) mod demo_radial_gradient;
pub(crate) mod demo_turbulence;

// Widens a filter's region from the SVG default (-10%/-10%/120%/120% of the referencing element's bounding box)
// to -50%/-50%/200%/200%, via the typed set_x/set_y/set_width/set_height setters rather than the generic
// set_attrs escape hatch. Shared by every build_filter closure that needs the same wider region to avoid visibly
// clipping their blur or offset shadow.
fn widen_filter_region(f: &SvgFilter) -> Result<(), Error> {
    f.set_x(-0.5)?;
    f.set_y(-0.5)?;
    f.set_width(2.0)?;
    f.set_height(2.0)?;
    Ok(())
}

// Narrows a filter's region from the SVG default (-10%/-10%/120%/120% of the referencing element's bounding box)
// down to exactly 0%/0%/100%/100% — i.e. the referencing element's own bounding box, with no margin. feImage's
// content fills its primitive subregion, which defaults to the *filter region* rather than the referencing
// element's box, so the default 120% padding makes an imported image render visibly larger than the same image
// placed directly via a plain <image> element. Unlike widen_filter_region's blur/offset use case, feImage has
// nothing that spills past its own source image's edges, so it needs no such margin.
fn exact_filter_region(f: &SvgFilter) -> Result<(), Error> {
    f.set_x(0.0)?;
    f.set_y(0.0)?;
    f.set_width(1.0)?;
    f.set_height(1.0)?;
    Ok(())
}
