use crate::{DemoClosure, W, colours::*, dom_err, keep_demo_closure};
use svg_dom::{
    Error, SvgRoot,
    root::{
        filter::LightSource,
        utils::{Point, Size},
    },
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// feDiffuseLighting / feSpecularLighting — Distant vs Point vs Spot (with/without limitingConeAngle)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Every panel here lights a plain, flat steelblue rect with feSpecularLighting alone. A flat surface has one
// constant normal, so any position-dependence in the highlight comes entirely from the light source itself, not
// from bump-map relief. This isolates exactly the LightSource-variant behaviour each column's own slider drives.
//
// Unlike `demo_lighting`'s own two sliders, which drive the same shared light across three columns, the four
// columns here are four genuinely different `LightSource` variants. Each gets its own single slider, driving the
// one parameter that most distinguishes its own column from the others:
//
// - Distant: `elevation`. `azimuth` was tried first, since it is `demo_lighting`'s own interactive parameter for
//   the same variant, but it has no visible effect here. A flat surface's own normal is constant, so a distant
//   light's N.L (and N.H, the specular halfway term) depends only on elevation: rotating azimuth around a fixed
//   vertical normal cannot change either dot product. `elevation` instead changes the whole surface's own
//   brightness, still with no hotspot, which is the visible half of "uniform sheen" this column can actually
//   demonstrate.
// - Point: `z`, its height above the surface. `LightSource::Point`'s own doc comment explains why: a larger `z`
//   makes the incident direction more uniform across the surface, a smaller one fans it out sharply near the
//   light, producing a tighter hotspot.
// - Spot (no cone): the light's own `x` position, sweeping the whole beam sideways. The light's own
//   `specular_exponent` was tried first, since it is the field `LightSource::Spot`'s own doc comment specifically
//   contrasts against `specular_lighting`'s identically-named parameter, but it has no visible effect in this
//   sandbox's own Chrome, checked across a wide range (0.01 through 10000, well past the 1.0-128.0 range the
//   surface's own specular exponent is conventionally clamped to). The specular hotspot's own position is driven
//   by the light's position, not by this exponent, so sweeping `x` instead reliably moves it.
// - Spot (with cone): `limiting_cone_angle`, the hard-edged cutoff this column adds on top of the same falloff
//   the previous column already shows.
//
// `specular_lighting_with_light` retains both the primitive and its light-source child as one `LightingNodes`.
// Every slider here only ever touches `.light`: each primitive's own `in` is set once at construction and never
// changes, so only `.primitive` from the plain `specular_lighting` would have been retained for that, and this
// demo has no further use for it once set.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

const RECT_W: f64 = 160.0;
const RECT_H: f64 = 100.0;
const LEFT_MARGIN: f64 = 20.0;
const ITEM_GAP: f64 = 30.0;

const COL1_X: f64 = LEFT_MARGIN; // Distant
const COL2_X: f64 = COL1_X + RECT_W + ITEM_GAP; // Point
const COL3_X: f64 = COL2_X + RECT_W + ITEM_GAP; // Spot, no cone
const COL4_X: f64 = COL3_X + RECT_W + ITEM_GAP; // Spot, with cone

const CONTROL_ROW_H: f64 = 62.0; // one HTML slider block: label + track + tick row + endpoint-value row
const CONTROL_GAP: f64 = 6.0; // gap between a slider block and the rect it drives
const CAPTION_OFFSET: f64 = 14.0; // caption sits this far below the rect's own bottom edge

const TOP: f64 = 10.0;
const RECT_Y: f64 = TOP + CONTROL_ROW_H + CONTROL_GAP;
const CAPTION_Y: f64 = RECT_Y + RECT_H + CAPTION_OFFSET;

const LIGHT_SOURCES_H: f64 = CAPTION_Y + 12.0;

const MIN_ELEVATION: i32 = 0;
const MAX_ELEVATION: i32 = 90;
const DEFAULT_ELEVATION: i32 = 55; // this demo's own original fixed elevation

const AZIMUTH: f64 = 235.0; // fixed throughout: rotating it has no visible effect on this column's own flat surface

const MIN_HEIGHT: i32 = 20;
const MAX_HEIGHT: i32 = 200;
const DEFAULT_HEIGHT: i32 = 60; // this demo's own original fixed z

// The rect this column's own light sweeps across spans COL3_X to COL3_X + RECT_W. Confirmed empirically: sweeping
// the light's own x this way, rather than its own points_at_x, moves the specular hotspot predictably from edge
// to edge, since the hotspot's own position is governed by the light's position, not by where it aims.
const MIN_SPOT_X: i32 = 400;
const MAX_SPOT_X: i32 = 560;
const DEFAULT_SPOT_X: i32 = 440; // this demo's own original fixed light x

const SPOT_SPECULAR_EXPONENT: f64 = 2.0; // fixed throughout, not one of these sliders: see the module doc comment

// `0.0` reads as a fully open beam in this sandbox's own Chrome, not the near-invisible cutoff `LightSource::Spot`'s
// own `limiting_cone_angle` doc comment describes for a zero-width cone. Confirmed empirically: `1.0` already
// renders as the expected tight, near-invisible clip, so this is a real rendering quirk specific to the literal
// `0.0` boundary, not a gradual falloff this slider's own range needs to reach. `5` keeps the slider's own tick
// ladder on round numbers while staying clear of it.
const MIN_CONE_ANGLE: i32 = 5;
const MAX_CONE_ANGLE: i32 = 90;
const DEFAULT_CONE_ANGLE: i32 = 20; // this demo's own original fixed value

const SPECULAR_CONSTANT: f64 = 1.0;
const SPECULAR_EXPONENT: f64 = 8.0; // the surface's own shininess, fixed throughout, not one of these sliders
const SURFACE_SCALE: f64 = 2.0;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
fn elevation_value_text(elevation: f64) -> String {
    format!("{elevation:.0} degrees")
}

fn distant_caption_text(elevation: f64) -> String {
    format!("Distant (uniform) elevation {elevation:.0}°")
}

fn height_value_text(z: f64) -> String {
    z.to_string()
}

fn point_caption_text(z: f64) -> String {
    format!("Point (hotspot) z {}", height_value_text(z))
}

fn spot_x_value_text(x: f64) -> String {
    x.to_string()
}

fn spot_open_caption_text(x: f64) -> String {
    format!("Spot (no cone) x {}", spot_x_value_text(x))
}

fn cone_angle_value_text(angle: f64) -> String {
    format!("{angle:.0} degrees")
}

fn spot_cone_caption_text(angle: f64) -> String {
    format!("Spot (cone) {angle:.0}°")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-light-sources", Size::new(W, LIGHT_SOURCES_H))?;
    let defs = svg.defs()?;

    let default_elevation = f64::from(DEFAULT_ELEVATION);
    let default_height = f64::from(DEFAULT_HEIGHT);
    let default_spot_x = f64::from(DEFAULT_SPOT_X);
    let default_cone_angle = f64::from(DEFAULT_CONE_ANGLE);

    // Every filter below is pinned to exactly its rect's own bounding box, the same exact_filter_region pattern
    // demo_fe_image uses. SourceAlpha is fully transparent past the rect's own edge, but a lit surface's own
    // normal there is still well-defined, so the SVG default filter region's own 10% margin paints a faint,
    // rectangular dark fringe past that edge. Pinning the region removes it, leaving each column's own rect edge
    // crisp instead of framed.
    //
    // Distant: direction is identical everywhere per the SVG spec, so a flat surface's highlight is uniform
    // across its whole area — no hotspot, unlike the three sources to its right. Raising elevation brightens
    // that uniform sheen; lowering it towards a grazing angle dims it, still with no hotspot anywhere.
    let distant_filter = defs.filter("light-distant")?;
    super::exact_filter_region(&distant_filter)?;
    let distant = distant_filter.specular_lighting_with_light(
        SURFACE_SCALE,
        SPECULAR_CONSTANT,
        SPECULAR_EXPONENT,
        "white",
        LightSource::Distant {
            azimuth: AZIMUTH,
            elevation: default_elevation,
        },
    )?;
    distant.primitive.set_attr("in", "SourceAlpha")?;

    let distant_rect = svg.rect(Point::new(COL1_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    distant_rect.set_fill(STEELBLUE)?;
    distant_rect.set_filter("light-distant")?;

    let distant_caption = svg.text(
        Point::new(COL1_X + RECT_W / 2.0, CAPTION_Y),
        &distant_caption_text(default_elevation),
    )?;
    distant_caption.set_fill(CAPTION)?;
    distant_caption.set_attr("font-size", "11")?;
    distant_caption.set_attr("text-anchor", "middle")?;

    // Point: direction varies per surface sample (see LightSource::Point's own doc comment), producing a hotspot
    // nearest the light's projected (x, y) rather than Distant's uniform sheen — with no distance attenuation on
    // the light's colour itself, only its direction changes across the surface. Raising z spreads that hotspot
    // out towards Distant's own uniform sheen; lowering it sharpens the hotspot further.
    let point_filter = defs.filter("light-point")?;
    super::exact_filter_region(&point_filter)?;
    let point = point_filter.specular_lighting_with_light(
        SURFACE_SCALE,
        SPECULAR_CONSTANT,
        SPECULAR_EXPONENT,
        "white",
        LightSource::Point {
            x: COL2_X + 40.0,
            y: RECT_Y + RECT_H / 2.0,
            z: default_height,
        },
    )?;
    point.primitive.set_attr("in", "SourceAlpha")?;

    let point_rect = svg.rect(Point::new(COL2_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    point_rect.set_fill(STEELBLUE)?;
    point_rect.set_filter("light-point")?;

    let point_caption = svg.text(
        Point::new(COL2_X + RECT_W / 2.0, CAPTION_Y),
        &point_caption_text(default_height),
    )?;
    point_caption.set_fill(CAPTION)?;
    point_caption.set_attr("font-size", "11")?;
    point_caption.set_attr("text-anchor", "middle")?;

    // Spot, no limiting_cone_angle: still directional (pow(-L.S, specular_exponent) falloff away from the aim
    // axis), but with no additional hard-edged cutoff — compare its softer edge against the next column's.
    // Sweeping the light's own x moves the specular hotspot across the surface, from edge to edge.
    let spot_open_filter = defs.filter("light-spot-open")?;
    super::exact_filter_region(&spot_open_filter)?;
    let spot_open = spot_open_filter.specular_lighting_with_light(
        SURFACE_SCALE,
        SPECULAR_CONSTANT,
        SPECULAR_EXPONENT,
        "white",
        LightSource::Spot {
            x: default_spot_x,
            y: RECT_Y + 20.0,
            z: 80.0,
            points_at_x: COL3_X + 120.0,
            points_at_y: RECT_Y + RECT_H - 10.0,
            points_at_z: 0.0,
            specular_exponent: SPOT_SPECULAR_EXPONENT,
            limiting_cone_angle: None,
        },
    )?;
    spot_open.primitive.set_attr("in", "SourceAlpha")?;

    let spot_open_rect = svg.rect(Point::new(COL3_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    spot_open_rect.set_fill(STEELBLUE)?;
    spot_open_rect.set_filter("light-spot-open")?;

    let spot_open_caption = svg.text(
        Point::new(COL3_X + RECT_W / 2.0, CAPTION_Y),
        &spot_open_caption_text(default_spot_x),
    )?;
    spot_open_caption.set_fill(CAPTION)?;
    spot_open_caption.set_attr("font-size", "11")?;
    spot_open_caption.set_attr("text-anchor", "middle")?;

    // Spot, with limiting_cone_angle: the identical beam above, now additionally hard-clipped — the sharper
    // circular edge here, versus the previous column's falloff-only edge, is exactly the distinction
    // LightSource::Spot's own limiting_cone_angle doc comment describes. Widening the angle relaxes that cutoff;
    // narrowing it clips the beam down towards its own aim axis.
    let spot_cone_filter = defs.filter("light-spot-cone")?;
    super::exact_filter_region(&spot_cone_filter)?;
    let spot_cone = spot_cone_filter.specular_lighting_with_light(
        SURFACE_SCALE,
        SPECULAR_CONSTANT,
        SPECULAR_EXPONENT,
        "white",
        LightSource::Spot {
            x: COL4_X + 40.0,
            y: RECT_Y + 20.0,
            z: 80.0,
            points_at_x: COL4_X + 120.0,
            points_at_y: RECT_Y + RECT_H - 10.0,
            points_at_z: 0.0,
            specular_exponent: SPOT_SPECULAR_EXPONENT,
            limiting_cone_angle: Some(default_cone_angle),
        },
    )?;
    spot_cone.primitive.set_attr("in", "SourceAlpha")?;

    let spot_cone_rect = svg.rect(Point::new(COL4_X, RECT_Y), Size::new(RECT_W, RECT_H))?;
    spot_cone_rect.set_fill(STEELBLUE)?;
    spot_cone_rect.set_filter("light-spot-cone")?;

    let spot_cone_caption = svg.text(
        Point::new(COL4_X + RECT_W / 2.0, CAPTION_Y),
        &spot_cone_caption_text(default_cone_angle),
    )?;
    spot_cone_caption.set_fill(CAPTION)?;
    spot_cone_caption.set_attr("font-size", "11")?;
    spot_cone_caption.set_attr("text-anchor", "middle")?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    let elevation_slider = super::build_h_slider(
        &svg,
        Point::new(COL1_X, TOP),
        Size::new(RECT_W, CONTROL_ROW_H),
        ("elevation", "distant light elevation"),
        (MIN_ELEVATION, MAX_ELEVATION, DEFAULT_ELEVATION),
        15, // a tick lands on 0°/15°/30°/45°/60°/75°/90°
        &["0°", "90°"],
    )?
    .input;
    elevation_slider
        .set_attribute("aria-valuetext", &elevation_value_text(default_elevation))
        .map_err(dom_err)?;

    let height_slider = super::build_h_slider(
        &svg,
        Point::new(COL2_X, TOP),
        Size::new(RECT_W, CONTROL_ROW_H),
        ("height (z)", "point light height"),
        (MIN_HEIGHT, MAX_HEIGHT, DEFAULT_HEIGHT),
        20, // a tick lands exactly on every multiple of 20 from 20 through 200
        &["20", "200"],
    )?
    .input;
    height_slider
        .set_attribute("aria-valuetext", &height_value_text(default_height))
        .map_err(dom_err)?;

    let spot_x_slider = super::build_h_slider(
        &svg,
        Point::new(COL3_X, TOP),
        Size::new(RECT_W, CONTROL_ROW_H),
        ("position (x)", "spot light horizontal position"),
        (MIN_SPOT_X, MAX_SPOT_X, DEFAULT_SPOT_X),
        40, // a tick lands exactly on every multiple of 40 from 400 through 560
        &["400", "560"],
    )?
    .input;
    spot_x_slider
        .set_attribute("aria-valuetext", &spot_x_value_text(default_spot_x))
        .map_err(dom_err)?;

    let cone_angle_slider = super::build_h_slider(
        &svg,
        Point::new(COL4_X, TOP),
        Size::new(RECT_W, CONTROL_ROW_H),
        ("cone angle", "spot light limiting cone angle"),
        (MIN_CONE_ANGLE, MAX_CONE_ANGLE, DEFAULT_CONE_ANGLE),
        15, // a tick lands on 5°/20°/35°/50°/65°/80°, plus a final tick at 90°
        &["5°", "90°"],
    )?
    .input;
    cone_angle_slider
        .set_attribute("aria-valuetext", &cone_angle_value_text(default_cone_angle))
        .map_err(dom_err)?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    {
        let slider = elevation_slider.clone();
        let light = distant.light.clone();
        let caption = distant_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let elevation = slider.value_as_number();
            let _ = light.set_attr("elevation", &elevation.to_string());
            let _ = slider.set_attribute("aria-valuetext", &elevation_value_text(elevation));
            caption.set_text(&distant_caption_text(elevation));
        });
        elevation_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    {
        let slider = height_slider.clone();
        let light = point.light.clone();
        let caption = point_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let z = slider.value_as_number();
            let text = height_value_text(z);
            let _ = light.set_attr("z", &text);
            let _ = slider.set_attribute("aria-valuetext", &text);
            caption.set_text(&point_caption_text(z));
        });
        height_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    {
        let slider = spot_x_slider.clone();
        let light = spot_open.light.clone();
        let caption = spot_open_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let x = slider.value_as_number();
            let text = spot_x_value_text(x);
            let _ = light.set_attr("x", &text);
            let _ = slider.set_attribute("aria-valuetext", &text);
            caption.set_text(&spot_open_caption_text(x));
        });
        spot_x_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    {
        let slider = cone_angle_slider.clone();
        let light = spot_cone.light.clone();
        let caption = spot_cone_caption.clone();
        let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let angle = slider.value_as_number();
            let _ = light.set_attr("limitingConeAngle", &angle.to_string());
            let _ = slider.set_attribute("aria-valuetext", &cone_angle_value_text(angle));
            caption.set_text(&spot_cone_caption_text(angle));
        });
        cone_angle_slider
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_input);
    }

    Ok(())
}
