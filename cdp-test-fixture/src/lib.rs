//! WASM fixture for `cdp-integration-test`'s Chrome-DevTools-Protocol integration tests
//! (`tests/accessibility_tree.rs`, `tests/filter_blend_render.rs`, and `tests/turbulence_scale_zero_render.rs`).
//!
//! Builds a handful of SVG elements exercising real `svg-dom` API calls whose correctness cannot be verified from
//! the DOM alone: `set_title`/`set_desc` against real accessible-name/description computation rules (ARIA
//! precedence, blank-value rejection), `SvgFilter::blend`'s alpha-preserving tint chain against real rendered
//! pixels, and `SvgFilter::displacement_map`'s `scale` argument against real rendered pixels too. All three need a
//! real Chrome instance to observe — the first via the Accessibility CDP domain, the other two via actual
//! rasterised output — neither of which `wasm-bindgen-test`'s WebDriver-run browser tests have access to.
//!
//! Every shape-based accessibility scenario receives an explicit `role="img"` so Chrome always creates an
//! accessibility-tree node for it, regardless of any SVG-specific pruning heuristics that might otherwise apply to
//! a plain, otherwise-unremarkable shape. The `<a>` scenario is the exception: it deliberately relies on its native
//! link semantics rather than this workaround — see its own comment below.

use svg_dom::{
    Error, SvgRoot,
    root::{
        filter::{BlendMode, Channel, CompositeOperator, TurbulenceType},
        utils::Point,
    },
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    build().map_err(|e| JsValue::from_str(&e.to_string()))
}

fn build() -> Result<(), Error> {
    let svg = SvgRoot::create_in("stage", svg_dom::root::utils::Size::new(540.0, 200.0))?;

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

    // Signals to the driving test (polling via `wait_for_element`) that the fixture has finished building.
    let ready = svg.rect(Point::new(0.0, 0.0), svg_dom::root::utils::Size::new(1.0, 1.0))?;
    ready.as_element().set_id("fixture-ready");

    Ok(())
}

fn js_to_dom_err(e: JsValue) -> Error {
    Error::Dom(format!("{e:?}"))
}
