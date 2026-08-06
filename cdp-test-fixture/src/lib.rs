//! WASM fixture for `cdp-integration-test`'s Chrome-DevTools-Protocol integration tests
//! (`tests/accessibility_tree.rs`, `tests/filter_blend_render.rs`, `tests/turbulence_scale_zero_render.rs`,
//! `tests/lighting_render.rs`, and `tests/light_sources_render.rs`).
//!
//! Builds a handful of SVG elements exercising real `svg-dom` API calls whose correctness cannot be verified from
//! the DOM alone: `set_title`/`set_desc` against real accessible-name/description computation rules (ARIA
//! precedence, blank-value rejection), `SvgFilter::blend`'s alpha-preserving tint chain against real rendered
//! pixels, `SvgFilter::displacement_map`'s `scale` argument against real rendered pixels too,
//! `SvgFilter::diffuse_lighting`'s own `surfaceScale`/light source `azimuth` against real rendered pixels as well,
//! and `SvgFilter::specular_lighting`'s own response to each `LightSource` variant's own distinguishing parameter
//! against real rendered pixels too. All five need a real Chrome instance to observe — the first via the
//! Accessibility CDP domain, the other four via actual rasterised output — neither of which
//! `wasm-bindgen-test`'s WebDriver-run browser tests have access to.
//!
//! Every shape-based accessibility scenario receives an explicit `role="img"` so Chrome always creates an
//! accessibility-tree node for it, regardless of any SVG-specific pruning heuristics that might otherwise apply to
//! a plain, otherwise-unremarkable shape. The `<a>` scenario is the exception: it deliberately relies on its native
//! link semantics rather than this workaround — see its own comment below.

use svg_dom::{
    Error, SvgRoot,
    root::{
        filter::{BlendMode, Channel, CompositeOperator, LightSource, TurbulenceType},
        utils::Point,
    },
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    build().map_err(|e| JsValue::from_str(&e.to_string()))
}

fn build() -> Result<(), Error> {
    let svg = SvgRoot::create_in("stage", svg_dom::root::utils::Size::new(1750.0, 360.0))?;

    // 1. title-only naming: no ARIA attributes, so the <title> child supplies the accessible name.
    let s1 = svg.circle(Point::new(10.0, 10.0), 5.0)?;
    s1.as_element().set_id("s1");
    s1.as_element().set_attribute("role", "img").map_err(js_to_dom_err)?;
    s1.set_title("Save file")?;

    // 2. description exposure: a name (via <title>) plus a <desc> — the <desc> supplies the accessible description.
    let s2 = svg.circle(Point::new(30.0, 10.0), 5.0)?;
    s2.as_element().set_id("s2");
    s2.as_element().set_attribute("role", "img").map_err(js_to_dom_err)?;
    s2.set_title("Icon")?;
    s2.set_desc("Writes the current document to disk.")?;

    // 3. aria-label overrides <title> in accessible-name computation.
    let s3 = svg.circle(Point::new(50.0, 10.0), 5.0)?;
    s3.as_element().set_id("s3");
    s3.as_element().set_attribute("role", "img").map_err(js_to_dom_err)?;
    s3.set_title("Should be overridden by aria-label")?;
    s3.as_element()
        .set_attribute("aria-label", "Override name")
        .map_err(js_to_dom_err)?;

    // 4. aria-describedby overrides <desc> in accessible-description computation. The referenced element supplies
    // the description text; its own visibility is irrelevant to this fixture.
    let s4 = svg.circle(Point::new(70.0, 10.0), 5.0)?;
    s4.as_element().set_id("s4");
    s4.as_element().set_attribute("role", "img").map_err(js_to_dom_err)?;
    s4.set_title("Icon")?;
    s4.set_desc("Should be overridden by aria-describedby")?;
    s4.as_element()
        .set_attribute("aria-describedby", "s4-desc")
        .map_err(js_to_dom_err)?;
    let s4_desc_source = svg.text(Point::new(70.0, 30.0), "Override description")?;
    s4_desc_source.as_element().set_id("s4-desc");

    // 5. blank/whitespace-only values are rejected outright, so this element must end up with no accessible name at
    // all — the rejection must never silently attach a blank <title>.
    let s5 = svg.circle(Point::new(90.0, 10.0), 5.0)?;
    s5.as_element().set_id("s5");
    s5.as_element().set_attribute("role", "img").map_err(js_to_dom_err)?;
    let rejected = s5.set_title("   ");
    if rejected.is_ok() {
        return Err(Error::Dom("fixture invariant violated: blank <title> was accepted".into()));
    }

    // 6. aria-labelledby has higher precedence than aria-label, which in turn has higher precedence than <title> —
    // give this element all three, plus an aria-label, so the test proves aria-labelledby wins over *both* of the
    // others, not just over <title> the way scenario 3 does.
    let s6 = svg.circle(Point::new(110.0, 10.0), 5.0)?;
    s6.as_element().set_id("s6");
    s6.as_element().set_attribute("role", "img").map_err(js_to_dom_err)?;
    s6.set_title("Should be overridden by aria-labelledby")?;
    s6.as_element()
        .set_attribute("aria-label", "Should also be overridden by aria-labelledby")
        .map_err(js_to_dom_err)?;
    s6.as_element()
        .set_attribute("aria-labelledby", "s6-label")
        .map_err(js_to_dom_err)?;
    let s6_label_source = svg.text(Point::new(110.0, 30.0), "Labelledby override name")?;
    s6_label_source.as_element().set_id("s6-label");

    // 7. `<a>` around visible text is exposed as a named, accessible link — SVG maps `<a>` to the ARIA "link" role
    // automatically, unlike the plain shapes above, which need the explicit role="img" workaround to force a node
    // to exist at all. No aria-label here: the accessible name must come from the linked text content itself, the
    // same way it would for an HTML <a>.
    let s7 = svg.anchor("https://example.com")?;
    s7.as_element().set_id("s7");
    let s7_label = svg.text(Point::new(130.0, 10.0), "Read the docs")?;
    s7.append(&s7_label)?;

    // blend-circle: the real alpha-preserving tint chain from `SvgFilter::blend`'s own doc example
    // (flood -> blend -> composite(In)), applied to a circle rather than a rectangle so its bounding box has
    // genuinely transparent corners for `tests/filter_blend_render.rs` to sample. A solid white fill is
    // deliberate: white is Multiply's identity element, so a correctly alpha-preserving chain paints the flood
    // colour into the circle completely unchanged, giving that test an *exact* expected RGB inside the circle
    // rather than an approximate one, while the corner of the bounding box must stay fully transparent
    // (alpha 0) — that second assertion is what the pre-fix flood+blend chain (without the final composite)
    // would fail, since the opaque flood used to leak straight through it.
    let defs = svg.defs()?;
    let blend_filter = defs.build_filter("blend-tint", |f| {
        f.flood("#f0883e", 1.0)?.set_attr("result", "tint")?;
        f.blend("tint", BlendMode::Multiply)?
            .set_attrs([("in", "SourceGraphic"), ("result", "tinted")])?;
        f.composite("SourceGraphic", CompositeOperator::In)?.set_attr("in", "tinted")?;
        Ok(())
    })?;
    let blend_circle = svg.circle(Point::new(150.0, 120.0), 50.0)?;
    blend_circle.as_element().set_id("blend-circle");
    blend_circle.set_fill("white")?;
    blend_circle.set_filter_ref(&blend_filter)?;

    // turbulence-reference / turbulence-scale-zero: for `tests/turbulence_scale_zero_render.rs`, which checks the
    // demo gallery's own turbulence panel's prominent claim that scale 0 restores a perfect geometric circle
    // (`demo/panels/panel-turbulence.html`, `demo-app/src/paint/demo_turbulence.rs`) against real rendered pixels,
    // not just the `scale="0"` attribute a structural DOM test can already see. The two circles use the same
    // radius and fill and are sampled at corresponding offsets around their own respective centres; only the
    // second one passes through `turbulence` -> `displacement_map`, with `scale` fixed at `0.0`. If scale 0
    // really does produce "no displacement at all"
    // (`SvgFilter::displacement_map`'s own doc comment), the two circles' own edges should rasterise identically,
    // within antialiasing rounding — a non-zero `scale`'s organic, hand-drawn edge would not.
    //
    // The filter's own region is pinned to exactly the circle's own bounding box (0%/0%/100%/100%, the same
    // `exact_filter_region` pattern `demo-app`'s own `feImage` demo uses), rather than left at SVG's own default
    // 10%-margin region. That default margin is what `tests/turbulence_scale_zero_render.rs`'s own module doc
    // comment warns about: in this sandbox's headless, software-rendered Chrome, a filtered element's own
    // fractional-margin region gets rasterised into an intermediate buffer whose own compositing back onto the
    // page introduces a real, several-pixel positional error, even at `scale` `0.0` — unrelated to any real
    // displacement, but large enough to make an unpinned region's own comparison against the unfiltered reference
    // circle fail unpredictably. Pinning the region to a plain 100% box removed that error outright.
    let turbulence_reference = svg.circle(Point::new(260.0, 120.0), 40.0)?;
    turbulence_reference.as_element().set_id("turbulence-reference");
    turbulence_reference.set_fill("steelblue")?;

    let turbulence_scale_zero_filter = defs.filter("turbulence-scale-zero-filter")?;
    turbulence_scale_zero_filter.set_x(0.0)?;
    turbulence_scale_zero_filter.set_y(0.0)?;
    turbulence_scale_zero_filter.set_width(1.0)?;
    turbulence_scale_zero_filter.set_height(1.0)?;
    turbulence_scale_zero_filter
        .turbulence(0.02, 3, 5.0, TurbulenceType::FractalNoise)?
        .set_attr("result", "noise")?;
    turbulence_scale_zero_filter
        .displacement_map("noise", 0.0, Channel::Red, Channel::Green)?
        .set_attr("in", "SourceGraphic")?;

    let turbulence_scale_zero = svg.circle(Point::new(360.0, 120.0), 40.0)?;
    turbulence_scale_zero.as_element().set_id("turbulence-scale-zero");
    turbulence_scale_zero.set_fill("steelblue")?;
    turbulence_scale_zero.set_filter_ref(&turbulence_scale_zero_filter)?;

    // turbulence-scale-sixty: the positive control for the comparison above. On its own, "scale zero rasterises
    // like the unfiltered reference" is a one-sided claim — it is equally consistent with a correctly-working
    // filter chain and with a browser that silently ignored the filter, or fell back to unfiltered SourceGraphic,
    // since both would also rasterise like the reference. This third circle uses the same filter chain, but with
    // `scale` fixed at `60.0` (`demo_turbulence.rs`'s own documented maximum) rather than `0.0`, so
    // `tests/turbulence_scale_zero_render.rs` can also assert the opposite: that its own fixture and sampling
    // method *do* detect a real, substantial displacement when one is actually present, not just that they fail
    // to detect an absent one.
    //
    // Unlike `turbulence-scale-zero`'s own filter, this one keeps the wider region `demo_turbulence.rs`'s real,
    // interactive circle uses (`widen_filter_region`: -50%/-50%/200%/200%), rather than the region pinned exactly
    // to the bounding box. A genuine `scale / 2` = 30px displacement needs room to sample source pixels from
    // outside the bare bounding box; the zero-displacement case never reads past its own edge, which is why that
    // filter can stay pinned tight without clipping anything real.
    let turbulence_scale_sixty_filter = defs.filter("turbulence-scale-sixty-filter")?;
    turbulence_scale_sixty_filter.set_x(-0.5)?;
    turbulence_scale_sixty_filter.set_y(-0.5)?;
    turbulence_scale_sixty_filter.set_width(2.0)?;
    turbulence_scale_sixty_filter.set_height(2.0)?;
    turbulence_scale_sixty_filter
        .turbulence(0.02, 3, 5.0, TurbulenceType::FractalNoise)?
        .set_attr("result", "noise")?;
    turbulence_scale_sixty_filter
        .displacement_map("noise", 60.0, Channel::Red, Channel::Green)?
        .set_attr("in", "SourceGraphic")?;

    let turbulence_scale_sixty = svg.circle(Point::new(460.0, 120.0), 40.0)?;
    turbulence_scale_sixty.as_element().set_id("turbulence-scale-sixty");
    turbulence_scale_sixty.set_fill("steelblue")?;
    turbulence_scale_sixty.set_filter_ref(&turbulence_scale_sixty_filter)?;

    // lighting-reference / lighting-azimuth-90 / lighting-scale-zero: for `tests/lighting_render.rs`, which checks
    // `demo_lighting.rs`'s own surfaceScale and azimuth sliders (`demo-app/src/paint/demo_lighting.rs`) against
    // real rendered pixels, not just the attribute values a structural DOM test
    // (`demo-app/src/browser_tests/paint/lighting.rs`) can already see. All three run the exact same
    // feDiffuseLighting recipe that demo's own "diffuse-only" column uses, reading SourceAlpha, at that demo's
    // own default surfaceScale (6) and azimuth (235deg) unless noted otherwise below.
    //
    // Each filter's own region is pinned to exactly its circle's own bounding box, the same
    // turbulence-scale-zero pattern above, so every circle rasterises at an identical position — feDiffuseLighting
    // never displaces content the way displacement_map does, but a consistent pinned region still keeps this
    // fixture's own three circles free of the unpinned-region compositing drift that pattern's own comment
    // describes.
    let lighting_reference_filter = defs.filter("lighting-reference-filter")?;
    lighting_reference_filter.set_x(0.0)?;
    lighting_reference_filter.set_y(0.0)?;
    lighting_reference_filter.set_width(1.0)?;
    lighting_reference_filter.set_height(1.0)?;
    lighting_reference_filter
        .diffuse_lighting(
            6.0,
            1.0,
            "white",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )?
        .set_attr("in", "SourceAlpha")?;

    let lighting_reference = svg.circle(Point::new(580.0, 120.0), 40.0)?;
    lighting_reference.as_element().set_id("lighting-reference");
    lighting_reference.set_fill("steelblue")?;
    lighting_reference.set_filter_ref(&lighting_reference_filter)?;

    // The azimuth positive control: the same recipe as `lighting-reference`, with only `azimuth` changed (90deg
    // instead of 235deg). If the demo's own azimuth slider really does turn the rendered light, not just the
    // `<feDistantLight>` attribute a DOM test can already see, this circle's own rim should rasterise visibly
    // differently from `lighting-reference` at matching angles.
    let lighting_azimuth_90_filter = defs.filter("lighting-azimuth-90-filter")?;
    lighting_azimuth_90_filter.set_x(0.0)?;
    lighting_azimuth_90_filter.set_y(0.0)?;
    lighting_azimuth_90_filter.set_width(1.0)?;
    lighting_azimuth_90_filter.set_height(1.0)?;
    lighting_azimuth_90_filter
        .diffuse_lighting(6.0, 1.0, "white", LightSource::Distant { azimuth: 90.0, elevation: 55.0 })?
        .set_attr("in", "SourceAlpha")?;

    let lighting_azimuth_90 = svg.circle(Point::new(680.0, 120.0), 40.0)?;
    lighting_azimuth_90.as_element().set_id("lighting-azimuth-90");
    lighting_azimuth_90.set_fill("steelblue")?;
    lighting_azimuth_90.set_filter_ref(&lighting_azimuth_90_filter)?;

    // The surfaceScale negative control: the same recipe as `lighting-reference`, with only `surfaceScale`
    // changed (0 instead of 6). `panel-lighting.html` claims that this flattens the bump map entirely, so the
    // whole circle should light uniformly, with no rim-to-rim variation left for `lighting-reference`'s own
    // non-zero surfaceScale to show.
    let lighting_scale_zero_filter = defs.filter("lighting-scale-zero-filter")?;
    lighting_scale_zero_filter.set_x(0.0)?;
    lighting_scale_zero_filter.set_y(0.0)?;
    lighting_scale_zero_filter.set_width(1.0)?;
    lighting_scale_zero_filter.set_height(1.0)?;
    lighting_scale_zero_filter
        .diffuse_lighting(
            0.0,
            1.0,
            "white",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )?
        .set_attr("in", "SourceAlpha")?;

    let lighting_scale_zero = svg.circle(Point::new(780.0, 120.0), 40.0)?;
    lighting_scale_zero.as_element().set_id("lighting-scale-zero");
    lighting_scale_zero.set_fill("steelblue")?;
    lighting_scale_zero.set_filter_ref(&lighting_scale_zero_filter)?;

    // ls-distant-low / ls-distant-high / ls-point-low-z / ls-point-high-z / ls-spot-left / ls-spot-right /
    // ls-cone-zero / ls-cone-narrow / ls-cone-wide: for `tests/light_sources_render.rs`, which checks
    // `demo_light_sources.rs`'s own four sliders (`demo-app/src/paint/demo_light_sources.rs`) against real
    // rendered pixels, not just the attribute values a structural DOM test
    // (`demo-app/src/browser_tests/paint/light_sources.rs`) can already see. Each pair runs that demo's own exact
    // `feSpecularLighting` recipe on a plain, flat rect (surfaceScale 2, specularConstant 1, surface's own
    // specularExponent 8, reading SourceAlpha), fixed at two different slider positions rather than one shared
    // default, so the test can compare their own rendered pixels directly. Every filter's own region is pinned to
    // exactly its rect's own bounding box, the same pattern `demo_light_sources.rs` itself uses to remove the
    // default region's own dark rendering fringe past the rect's edge.
    const LS_RECT_W: f64 = 160.0;
    const LS_RECT_H: f64 = 100.0;
    const LS_ROW_Y: f64 = 220.0;
    const LS_DISTANT_LOW_X: f64 = 20.0;
    const LS_DISTANT_HIGH_X: f64 = 210.0;
    const LS_POINT_LOW_X: f64 = 400.0;
    const LS_POINT_HIGH_X: f64 = 590.0;
    const LS_SPOT_LEFT_X: f64 = 780.0;
    const LS_SPOT_RIGHT_X: f64 = 970.0;
    const LS_CONE_ZERO_X: f64 = 1160.0;
    const LS_CONE_NARROW_X: f64 = 1350.0;
    const LS_CONE_WIDE_X: f64 = 1540.0;

    let ls_specular_lighting = |defs: &svg_dom::SvgDefs, id: &str, light: LightSource| -> Result<(), Error> {
        let filter = defs.filter(id)?;
        filter.set_x(0.0)?;
        filter.set_y(0.0)?;
        filter.set_width(1.0)?;
        filter.set_height(1.0)?;
        filter
            .specular_lighting(2.0, 1.0, 8.0, "white", light)?
            .set_attr("in", "SourceAlpha")?;
        Ok(())
    };

    // Distant: elevation 15deg (grazing) vs 85deg (near-overhead). A flat surface's own diffuse/specular response
    // to a distant light depends only on elevation, never azimuth — panel-light-sources.html's own claim, checked
    // here as average luminance across several sample points, not just a single centre pixel.
    ls_specular_lighting(
        &defs,
        "ls-distant-low-filter",
        LightSource::Distant {
            azimuth: 235.0,
            elevation: 15.0,
        },
    )?;
    let ls_distant_low = svg.rect(
        Point::new(LS_DISTANT_LOW_X, LS_ROW_Y),
        svg_dom::root::utils::Size::new(LS_RECT_W, LS_RECT_H),
    )?;
    ls_distant_low.as_element().set_id("ls-distant-low");
    ls_distant_low.set_fill("steelblue")?;
    ls_distant_low.set_filter("ls-distant-low-filter")?;

    ls_specular_lighting(
        &defs,
        "ls-distant-high-filter",
        LightSource::Distant {
            azimuth: 235.0,
            elevation: 85.0,
        },
    )?;
    let ls_distant_high = svg.rect(
        Point::new(LS_DISTANT_HIGH_X, LS_ROW_Y),
        svg_dom::root::utils::Size::new(LS_RECT_W, LS_RECT_H),
    )?;
    ls_distant_high.as_element().set_id("ls-distant-high");
    ls_distant_high.set_fill("steelblue")?;
    ls_distant_high.set_filter("ls-distant-high-filter")?;

    // Point: z 20 (low, sharp hotspot) vs 180 (high, spread towards a Distant-like uniform sheen) — the light's
    // own x/y sit at each rect's own centre, matching `demo_light_sources.rs`'s own construction.
    ls_specular_lighting(
        &defs,
        "ls-point-low-z-filter",
        LightSource::Point {
            x: LS_POINT_LOW_X + LS_RECT_W / 2.0,
            y: LS_ROW_Y + LS_RECT_H / 2.0,
            z: 20.0,
        },
    )?;
    let ls_point_low_z = svg.rect(
        Point::new(LS_POINT_LOW_X, LS_ROW_Y),
        svg_dom::root::utils::Size::new(LS_RECT_W, LS_RECT_H),
    )?;
    ls_point_low_z.as_element().set_id("ls-point-low-z");
    ls_point_low_z.set_fill("steelblue")?;
    ls_point_low_z.set_filter("ls-point-low-z-filter")?;

    ls_specular_lighting(
        &defs,
        "ls-point-high-z-filter",
        LightSource::Point {
            x: LS_POINT_HIGH_X + LS_RECT_W / 2.0,
            y: LS_ROW_Y + LS_RECT_H / 2.0,
            z: 180.0,
        },
    )?;
    let ls_point_high_z = svg.rect(
        Point::new(LS_POINT_HIGH_X, LS_ROW_Y),
        svg_dom::root::utils::Size::new(LS_RECT_W, LS_RECT_H),
    )?;
    ls_point_high_z.as_element().set_id("ls-point-high-z");
    ls_point_high_z.set_fill("steelblue")?;
    ls_point_high_z.set_filter("ls-point-high-z-filter")?;

    // Spot (no cone): light x at each rect's own left edge (matching demo_light_sources.rs's own slider minimum,
    // 400) or right edge (matching its own maximum, 560), with pointsAtX trailing by the same 80-unit offset
    // SPOT_OPEN_AIM_OFFSET uses in that file — translating the whole beam sideways rather than rotating it, the
    // same fix that file's own module doc comment explains. The left rect's own aim point still lands inside its
    // own bounds (80 units in from a 160-unit-wide rect); only the right rect's own aim point falls outside them
    // (240 units in), the same asymmetry the real demo has at its own two slider extremes.
    ls_specular_lighting(
        &defs,
        "ls-spot-left-filter",
        LightSource::Spot {
            x: LS_SPOT_LEFT_X,
            y: LS_ROW_Y + 20.0,
            z: 80.0,
            points_at_x: LS_SPOT_LEFT_X + 80.0,
            points_at_y: LS_ROW_Y + LS_RECT_H - 10.0,
            points_at_z: 0.0,
            specular_exponent: 2.0,
            limiting_cone_angle: None,
        },
    )?;
    let ls_spot_left = svg.rect(
        Point::new(LS_SPOT_LEFT_X, LS_ROW_Y),
        svg_dom::root::utils::Size::new(LS_RECT_W, LS_RECT_H),
    )?;
    ls_spot_left.as_element().set_id("ls-spot-left");
    ls_spot_left.set_fill("steelblue")?;
    ls_spot_left.set_filter("ls-spot-left-filter")?;

    ls_specular_lighting(
        &defs,
        "ls-spot-right-filter",
        LightSource::Spot {
            x: LS_SPOT_RIGHT_X + LS_RECT_W,
            y: LS_ROW_Y + 20.0,
            z: 80.0,
            points_at_x: LS_SPOT_RIGHT_X + LS_RECT_W + 80.0,
            points_at_y: LS_ROW_Y + LS_RECT_H - 10.0,
            points_at_z: 0.0,
            specular_exponent: 2.0,
            limiting_cone_angle: None,
        },
    )?;
    let ls_spot_right = svg.rect(
        Point::new(LS_SPOT_RIGHT_X, LS_ROW_Y),
        svg_dom::root::utils::Size::new(LS_RECT_W, LS_RECT_H),
    )?;
    ls_spot_right.as_element().set_id("ls-spot-right");
    ls_spot_right.set_fill("steelblue")?;
    ls_spot_right.set_filter("ls-spot-right-filter")?;

    // Spot (with cone): limitingConeAngle 0deg vs 5deg (this demo's own slider minimum) vs 90deg (its own
    // maximum), same light position/aim otherwise. `demo_light_sources.rs`'s own module doc comment and
    // `panel-light-sources.html` both document that 0deg renders as a fully open beam in this sandbox's own
    // Chrome, not the near-invisible cutoff the spec describes, which is why the slider's own range starts at 5
    // instead. The 0deg rect exists to check that specific claim directly, rather than trusting the slider's own
    // chosen minimum on the strength of the 5deg-vs-90deg comparison alone — that pair proves 5deg is usefully
    // narrow, not that 0deg is anomalous.
    ls_specular_lighting(
        &defs,
        "ls-cone-zero-filter",
        LightSource::Spot {
            x: LS_CONE_ZERO_X + 40.0,
            y: LS_ROW_Y + 20.0,
            z: 80.0,
            points_at_x: LS_CONE_ZERO_X + 120.0,
            points_at_y: LS_ROW_Y + LS_RECT_H - 10.0,
            points_at_z: 0.0,
            specular_exponent: 2.0,
            limiting_cone_angle: Some(0.0),
        },
    )?;
    let ls_cone_zero = svg.rect(
        Point::new(LS_CONE_ZERO_X, LS_ROW_Y),
        svg_dom::root::utils::Size::new(LS_RECT_W, LS_RECT_H),
    )?;
    ls_cone_zero.as_element().set_id("ls-cone-zero");
    ls_cone_zero.set_fill("steelblue")?;
    ls_cone_zero.set_filter("ls-cone-zero-filter")?;

    ls_specular_lighting(
        &defs,
        "ls-cone-narrow-filter",
        LightSource::Spot {
            x: LS_CONE_NARROW_X + 40.0,
            y: LS_ROW_Y + 20.0,
            z: 80.0,
            points_at_x: LS_CONE_NARROW_X + 120.0,
            points_at_y: LS_ROW_Y + LS_RECT_H - 10.0,
            points_at_z: 0.0,
            specular_exponent: 2.0,
            limiting_cone_angle: Some(5.0),
        },
    )?;
    let ls_cone_narrow = svg.rect(
        Point::new(LS_CONE_NARROW_X, LS_ROW_Y),
        svg_dom::root::utils::Size::new(LS_RECT_W, LS_RECT_H),
    )?;
    ls_cone_narrow.as_element().set_id("ls-cone-narrow");
    ls_cone_narrow.set_fill("steelblue")?;
    ls_cone_narrow.set_filter("ls-cone-narrow-filter")?;

    ls_specular_lighting(
        &defs,
        "ls-cone-wide-filter",
        LightSource::Spot {
            x: LS_CONE_WIDE_X + 40.0,
            y: LS_ROW_Y + 20.0,
            z: 80.0,
            points_at_x: LS_CONE_WIDE_X + 120.0,
            points_at_y: LS_ROW_Y + LS_RECT_H - 10.0,
            points_at_z: 0.0,
            specular_exponent: 2.0,
            limiting_cone_angle: Some(90.0),
        },
    )?;
    let ls_cone_wide = svg.rect(
        Point::new(LS_CONE_WIDE_X, LS_ROW_Y),
        svg_dom::root::utils::Size::new(LS_RECT_W, LS_RECT_H),
    )?;
    ls_cone_wide.as_element().set_id("ls-cone-wide");
    ls_cone_wide.set_fill("steelblue")?;
    ls_cone_wide.set_filter("ls-cone-wide-filter")?;

    // Signals to the driving test (polling via `wait_for_element`) that the fixture has finished building.
    let ready = svg.rect(Point::new(0.0, 0.0), svg_dom::root::utils::Size::new(1.0, 1.0))?;
    ready.as_element().set_id("fixture-ready");

    Ok(())
}

fn js_to_dom_err(e: JsValue) -> Error {
    Error::Dom(format!("{e:?}"))
}
