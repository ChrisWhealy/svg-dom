use crate::{BAND, H, PAD_Y, W, caption, colours::*};
use svg_dom::{
    Error, SvgRoot,
    root::{
        filter::LightSource,
        utils::{Point, Size},
    },
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feDiffuseLighting / feSpecularLighting — Distant vs Point vs Spot (with/without limitingConeAngle)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-light-sources", Size::new(W, H))?;

    let rect_w = 160.0_f64;
    let rect_h = BAND - 30.0;
    let rect_y = PAD_Y + 15.0;
    let xs: [f64; 4] = [20.0, 210.0, 400.0, 590.0];

    // Every panel here lights a plain, flat steelblue rect with feSpecularLighting alone -- a flat surface has one
    // constant normal, so any position-dependence in the highlight comes entirely from the light source itself, not
    // from bump-map relief. This isolates exactly the LightSource-variant behaviour the doc comments describe.
    svg.build_defs(|d| {
        // Distant: direction is identical everywhere per the SVG spec, so a flat surface's highlight is uniform
        // across its whole area -- no hotspot, unlike the three sources to its right.
        d.build_filter("light-distant", |f| {
            f.specular_lighting(
                2.0,
                1.0,
                8.0,
                "white",
                LightSource::Distant {
                    azimuth: 235.0,
                    elevation: 55.0,
                },
            )?
            .set_attr("in", "SourceAlpha")?;
            Ok(())
        })?;

        // Point: direction varies per surface sample (see LightSource::Point's own doc comment), producing a
        // hotspot nearest the light's projected (x, y) rather than Distant's uniform sheen -- with no distance
        // attenuation on the light's colour itself, only its direction changes across the surface.
        d.build_filter("light-point", |f| {
            f.specular_lighting(
                2.0,
                1.0,
                8.0,
                "white",
                LightSource::Point {
                    x: xs[1] + 40.0,
                    y: rect_y + rect_h / 2.0,
                    z: 60.0,
                },
            )?
            .set_attr("in", "SourceAlpha")?;
            Ok(())
        })?;

        // Spot, no limiting_cone_angle: still directional (pow(-L.S, specular_exponent) falloff away from the aim
        // axis), but with no additional hard-edged cutoff -- compare its softer edge against the next panel's.
        d.build_filter("light-spot-open", |f| {
            f.specular_lighting(
                2.0,
                1.0,
                8.0,
                "white",
                LightSource::Spot {
                    x: xs[2] + 40.0,
                    y: rect_y + 20.0,
                    z: 80.0,
                    points_at_x: xs[2] + 120.0,
                    points_at_y: rect_y + rect_h - 10.0,
                    points_at_z: 0.0,
                    specular_exponent: 2.0,
                    limiting_cone_angle: None,
                },
            )?
            .set_attr("in", "SourceAlpha")?;
            Ok(())
        })?;

        // Spot, with limiting_cone_angle: the identical beam above, now additionally hard-clipped -- the sharper
        // circular edge here, versus the previous panel's falloff-only edge, is exactly the distinction
        // LightSource::Spot's own `limiting_cone_angle` doc comment describes.
        d.build_filter("light-spot-cone", |f| {
            f.specular_lighting(
                2.0,
                1.0,
                8.0,
                "white",
                LightSource::Spot {
                    x: xs[3] + 40.0,
                    y: rect_y + 20.0,
                    z: 80.0,
                    points_at_x: xs[3] + 120.0,
                    points_at_y: rect_y + rect_h - 10.0,
                    points_at_z: 0.0,
                    specular_exponent: 2.0,
                    limiting_cone_angle: Some(20.0),
                },
            )?
            .set_attr("in", "SourceAlpha")?;
            Ok(())
        })?;

        Ok(())
    })?;

    let distant = svg.rect(Point::new(xs[0], rect_y), Size::new(rect_w, rect_h))?;
    distant.set_fill(STEELBLUE)?;
    distant.set_filter("light-distant")?;
    caption(&svg, xs[0] + rect_w / 2.0, "Distant (uniform)")?;

    let point = svg.rect(Point::new(xs[1], rect_y), Size::new(rect_w, rect_h))?;
    point.set_fill(STEELBLUE)?;
    point.set_filter("light-point")?;
    caption(&svg, xs[1] + rect_w / 2.0, "Point (hotspot)")?;

    let spot_open = svg.rect(Point::new(xs[2], rect_y), Size::new(rect_w, rect_h))?;
    spot_open.set_fill(STEELBLUE)?;
    spot_open.set_filter("light-spot-open")?;
    caption(&svg, xs[2] + rect_w / 2.0, "Spot (no cone)")?;

    let spot_cone = svg.rect(Point::new(xs[3], rect_y), Size::new(rect_w, rect_h))?;
    spot_cone.set_fill(STEELBLUE)?;
    spot_cone.set_filter("light-spot-cone")?;
    caption(&svg, xs[3] + rect_w / 2.0, "Spot (limitingConeAngle)")?;

    Ok(())
}
