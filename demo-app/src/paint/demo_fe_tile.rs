use crate::{BAND, H, PAD_Y, W, caption};
use svg_dom::{
    Error, SvgRoot,
    root::{
        filter::TurbulenceType,
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feTile — repeats an upstream primitive's own (narrowed) subregion across the whole filter region
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-fe-tile", Size::new(W, H))?;

    let box_w = 120.0_f64;
    let box_h = 80.0_f64;
    let tile_size = 20.0_f64;
    let y0 = PAD_Y + (BAND - box_h) / 2.0;
    let xs: [f64; 3] = [100.0, 340.0, 580.0];

    // `x`, `y`, `width`, `height` on a primitive default to `primitiveUnits="userSpaceOnUse"` — that is, the plain,
    // absolute document coordinates, not a fraction of the referencing rect's own bounding box the way
    // exact_filter_region's objectBoundingBox values are.
    //
    // Each turbulence subregion below is therefore pinned to its own panel's actual canvas position (xs[i], y0),
    // not to (0, 0), so the 20×20 tile lands inside that panel's box rather than off to the left of it.
    svg.build_defs(|d| {
        // Narrow the noise to a 20×20 corner, but do not repeat it: everything outside that corner stays transparent,
        // showing exactly the rectangle `tile` would otherwise repeat in the next two panels.
        d.build_filter("tile-source", |f| {
            super::exact_filter_region(f)?;
            f.turbulence(0.9, 2, 7.0, TurbulenceType::Turbulence)?.set_attrs([
                ("x", xs[0].to_string()),
                ("y", y0.to_string()),
                ("width", tile_size.to_string()),
                ("height", tile_size.to_string()),
            ])?;
            Ok(())
        })?;

        // The same narrowed noise, now repeated by `tile` across the whole (exact_filter_region-pinned) filter region.
        // Plain feTurbulence noise is not periodic at an arbitrary width/height, so the seams between adjacent copies
        // are visible — compare against the next panel.
        d.build_filter("tile-seams", |f| {
            super::exact_filter_region(f)?;
            f.turbulence(0.9, 2, 7.0, TurbulenceType::Turbulence)?.set_attrs([
                ("x", xs[1].to_string()),
                ("y", y0.to_string()),
                ("width", tile_size.to_string()),
                ("height", tile_size.to_string()),
            ])?;
            f.tile()?;
            Ok(())
        })?;

        // `stitchTiles="stitch"` (see `turbulence`'s own doc comment) adjusts baseFrequency slightly so the noise
        // is exactly periodic across its own subregion — the seams `tile` would otherwise leave visible disappear.
        d.build_filter("tile-stitched", |f| {
            super::exact_filter_region(f)?;
            f.turbulence(0.9, 2, 7.0, TurbulenceType::Turbulence)?.set_attrs([
                ("x", xs[2].to_string()),
                ("y", y0.to_string()),
                ("width", tile_size.to_string()),
                ("height", tile_size.to_string()),
                ("stitchTiles", "stitch".to_string()),
            ])?;
            f.tile()?;
            Ok(())
        })?;

        Ok(())
    })?;

    let source = svg.rect(Point::new(xs[0], y0), Size::new(box_w, box_h))?;
    source.set_filter("tile-source")?;
    caption(&svg, xs[0] + box_w / 2.0, "narrow tile (unrepeated)")?;

    let seams = svg.rect(Point::new(xs[1], y0), Size::new(box_w, box_h))?;
    seams.set_filter("tile-seams")?;
    caption(&svg, xs[1] + box_w / 2.0, "feTile (visible seams)")?;

    let stitched = svg.rect(Point::new(xs[2], y0), Size::new(box_w, box_h))?;
    stitched.set_filter("tile-stitched")?;
    caption(&svg, xs[2] + box_w / 2.0, "feTile + stitchTiles")?;

    Ok(())
}
