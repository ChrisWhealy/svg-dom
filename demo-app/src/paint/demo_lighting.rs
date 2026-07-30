use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot, TextAnchor,
    root::{
        filter::{CompositeOperator, LightSource},
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feDiffuseLighting / feSpecularLighting — matte + shiny bump-mapped lighting, and the combined bevel recipe
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-lighting", Size::new(W, H))?;

    // Every panel below lights the same upper-left distant light, so the emboss direction reads consistently
    // across all three lit columns.
    let light = LightSource::Distant {
        azimuth: 235.0,
        elevation: 55.0,
    };

    svg.build_defs(|d| {
        // feDiffuseLighting alone: bump-maps SourceAlpha and lights it with a matte (Lambertian) model. The result
        // is fully opaque (A = 1.0 everywhere, per the SVG spec), so with no further compositing this renders as a
        // flat lit plate with the text embossed into it, not a transparent highlight over the canvas.
        d.build_filter("diffuse-only", |f| {
            f.diffuse_lighting(6.0, 1.0, "white", light)?.set_attr("in", "SourceAlpha")?;
            Ok(())
        })?;

        // feSpecularLighting alone: the shiny (Blinn–Phong) counterpart. Unlike feDiffuseLighting, its alpha is the
        // maximum of its own lit R/G/B, so it renders as a highlight-only glint against the dark canvas
        // background, transparent everywhere the highlight itself is zero.
        d.build_filter("specular-only", |f| {
            f.specular_lighting(6.0, 1.0, 20.0, "white", light)?
                .set_attr("in", "SourceAlpha")?;
            Ok(())
        })?;

        // The combined bevel recipe from SvgFilter::diffuse_lighting's/specular_lighting's own doc comments:
        // multiply the opaque diffuse-lit surface back over the original graphic (composite Arithmetic, k1: 1.0),
        // then add the non-opaque specular highlight on top (composite Arithmetic, k2/k3: 1.0). Using merge instead
        // of the first composite step would paint the opaque lit plate over everything, hiding the original
        // steelblue fill entirely.
        d.build_filter("bevel-highlight", |f| {
            f.diffuse_lighting(6.0, 1.0, "white", light)?
                .set_attrs([("in", "SourceAlpha"), ("result", "lit")])?;
            f.composite("lit", CompositeOperator::Arithmetic)?.set_attrs([
                ("in", "SourceGraphic"),
                ("result", "beveled"),
                ("k1", "1"),
                ("k2", "0"),
                ("k3", "0"),
                ("k4", "0"),
            ])?;
            f.specular_lighting(6.0, 1.0, 20.0, "white", light)?
                .set_attrs([("in", "SourceAlpha"), ("result", "highlight")])?;
            f.composite("highlight", CompositeOperator::Arithmetic)?.set_attrs([
                ("in", "beveled"),
                ("k1", "0"),
                ("k2", "1"),
                ("k3", "1"),
                ("k4", "0"),
            ])?;
            Ok(())
        })?;

        Ok(())
    })?;

    let mid_y = PAD_Y + BAND / 2.0;
    let rect_w = 160.0_f64;
    let xs: [f64; 4] = [20.0, 210.0, 400.0, 590.0];

    let original = svg.text(Point::new(xs[0] + rect_w / 2.0, mid_y + 10.0), "LIGHT")?;
    original.set_fill(STEELBLUE)?;
    original.set_font_size(34.0)?;
    original.set_text_anchor(TextAnchor::Middle)?;
    original.set_attr("font-weight", "bold")?;
    caption(&svg, xs[0] + rect_w / 2.0, "original")?;

    let diffuse = svg.text(Point::new(xs[1] + rect_w / 2.0, mid_y + 10.0), "LIGHT")?;
    diffuse.set_fill(STEELBLUE)?;
    diffuse.set_font_size(34.0)?;
    diffuse.set_text_anchor(TextAnchor::Middle)?;
    diffuse.set_attr("font-weight", "bold")?;
    diffuse.set_filter("diffuse-only")?;
    caption(&svg, xs[1] + rect_w / 2.0, "feDiffuseLighting (opaque)")?;

    let specular = svg.text(Point::new(xs[2] + rect_w / 2.0, mid_y + 10.0), "LIGHT")?;
    specular.set_fill(STEELBLUE)?;
    specular.set_font_size(34.0)?;
    specular.set_text_anchor(TextAnchor::Middle)?;
    specular.set_attr("font-weight", "bold")?;
    specular.set_filter("specular-only")?;
    caption(&svg, xs[2] + rect_w / 2.0, "feSpecularLighting (highlight)")?;

    let beveled = svg.text(Point::new(xs[3] + rect_w / 2.0, mid_y + 10.0), "LIGHT")?;
    beveled.set_fill(STEELBLUE)?;
    beveled.set_font_size(34.0)?;
    beveled.set_text_anchor(TextAnchor::Middle)?;
    beveled.set_attr("font-weight", "bold")?;
    beveled.set_filter("bevel-highlight")?;
    caption(&svg, xs[3] + rect_w / 2.0, "combined bevel")?;

    Ok(())
}
