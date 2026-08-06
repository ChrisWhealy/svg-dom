//! Tests for `demo_lighting`'s own surfaceScale and azimuth sliders.

use crate::browser_tests::{container, document};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `diffuse_lighting`/`specular_lighting` each return their own primitive's `SvgNode` directly, so
/// `demo_lighting` retains all four (diffuse-only, specular-only, and the combined bevel's own pair) the same
/// way `demo_morphology`'s sliders do.
/// Source extraction cannot prove either slider actually reaches all four retained nodes live.
/// It cannot prove the azimuth slider reaches the four `<feDistantLight>` children too, which the library never
/// returns a handle for at all.
/// It cannot prove the combined bevel's own two light sources, one per lighting primitive, stay correctly
/// disambiguated from one another.
/// It cannot prove `elevation` and the diffuse/specular constants stay untouched by either slider.
/// It cannot prove the shared "surfaceScale N · azimuth N°" readout keeps both values in agreement after only
/// one of the two sliders moves.
/// Only a real browser can prove any of that.
#[wasm_bindgen_test]
fn demo_lighting_sliders_update_surface_scale_and_azimuth_together() {
    container("demo-lighting");
    crate::paint::demo_lighting::demo().expect("demo_lighting::demo should build without error");

    let root = document().get_element_by_id("demo-lighting").expect("container exists");

    let find_el = |selector: &str| -> web_sys::Element {
        root.query_selector(selector)
            .unwrap_or_else(|_| panic!("invalid selector {selector:?}"))
            .unwrap_or_else(|| panic!("no element matching {selector:?}"))
    };

    let find_text = |content: &str| -> web_sys::Element {
        let texts = root.query_selector_all("text").expect("query text elements");
        for i in 0..texts.length() {
            let el = texts
                .item(i)
                .expect("text item")
                .dyn_into::<web_sys::Element>()
                .expect("Element");
            if el.text_content().as_deref() == Some(content) {
                return el;
            }
        }
        panic!("no <text> element with content {content:?}");
    };

    let find_slider = |aria_label_selector: &str| -> web_sys::HtmlInputElement {
        root.query_selector(aria_label_selector)
            .expect("query slider")
            .unwrap_or_else(|| panic!("no slider matching {aria_label_selector:?}"))
            .dyn_into::<web_sys::HtmlInputElement>()
            .expect("slider is an HtmlInputElement")
    };

    let dispatch_input = |slider: &web_sys::HtmlInputElement, value: &str| {
        slider.set_value(value);
        let event = web_sys::Event::new("input").expect("create input event");
        slider.dispatch_event(&event).expect("dispatch input");
    };

    // --- the four retained lighting primitives, at this demo's own initial defaults ---
    let diffuse_only = find_el("#diffuse-only feDiffuseLighting");
    let specular_only = find_el("#specular-only feSpecularLighting");
    let bevel_diffuse = find_el("#bevel-highlight feDiffuseLighting");
    let bevel_specular = find_el("#bevel-highlight feSpecularLighting");

    for node in [&diffuse_only, &specular_only, &bevel_diffuse, &bevel_specular] {
        assert_eq!(
            node.get_attribute("surfaceScale").as_deref(),
            Some("6"),
            "6 is this demo's own initial default surfaceScale"
        );
    }
    assert_eq!(diffuse_only.get_attribute("in").as_deref(), Some("SourceAlpha"));
    assert_eq!(specular_only.get_attribute("in").as_deref(), Some("SourceAlpha"));

    // --- the combined bevel's own filter graph: diffuse multiplies over SourceGraphic first, then specular adds
    // on top. A regression swapping either composite's own coefficients, or either lighting primitive's own
    // `result` name, would silently break the recipe without changing either primitive's own surfaceScale. ---
    assert_eq!(bevel_diffuse.get_attribute("in").as_deref(), Some("SourceAlpha"));
    assert_eq!(bevel_diffuse.get_attribute("result").as_deref(), Some("lit"));
    assert_eq!(bevel_specular.get_attribute("in").as_deref(), Some("SourceAlpha"));
    assert_eq!(bevel_specular.get_attribute("result").as_deref(), Some("highlight"));

    let bevel_composites = root
        .query_selector_all("#bevel-highlight feComposite")
        .expect("query feComposite");
    assert_eq!(bevel_composites.length(), 2, "one composite per lighting primitive");
    let multiply = bevel_composites
        .item(0)
        .expect("first composite")
        .dyn_into::<web_sys::Element>()
        .expect("Element");
    assert_eq!(multiply.get_attribute("in").as_deref(), Some("SourceGraphic"));
    assert_eq!(multiply.get_attribute("in2").as_deref(), Some("lit"));
    assert_eq!(multiply.get_attribute("result").as_deref(), Some("beveled"));
    assert_eq!(multiply.get_attribute("k1").as_deref(), Some("1"));
    let add = bevel_composites
        .item(1)
        .expect("second composite")
        .dyn_into::<web_sys::Element>()
        .expect("Element");
    assert_eq!(add.get_attribute("in").as_deref(), Some("beveled"));
    assert_eq!(add.get_attribute("in2").as_deref(), Some("highlight"));
    assert_eq!(add.get_attribute("k2").as_deref(), Some("1"));
    assert_eq!(add.get_attribute("k3").as_deref(), Some("1"));

    // --- the four light sources, one per lighting primitive, disambiguated within the combined bevel filter by
    // their own parent primitive's tag name ---
    let diffuse_only_light = find_el("#diffuse-only feDistantLight");
    let specular_only_light = find_el("#specular-only feDistantLight");
    let bevel_diffuse_light = find_el("#bevel-highlight feDiffuseLighting feDistantLight");
    let bevel_specular_light = find_el("#bevel-highlight feSpecularLighting feDistantLight");

    for light in [
        &diffuse_only_light,
        &specular_only_light,
        &bevel_diffuse_light,
        &bevel_specular_light,
    ] {
        assert_eq!(
            light.get_attribute("azimuth").as_deref(),
            Some("235"),
            "235 is this demo's own initial default azimuth"
        );
        assert_eq!(
            light.get_attribute("elevation").as_deref(),
            Some("55"),
            "elevation is fixed, not one of this demo's own interactive controls"
        );
    }

    // --- both sliders, at this demo's own initial defaults ---
    let scale_slider = find_slider("input[aria-label='lighting surface scale']");
    assert_eq!(scale_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(scale_slider.get_attribute("max").as_deref(), Some("20"));
    assert_eq!(scale_slider.value(), "6");
    assert_eq!(scale_slider.get_attribute("aria-valuetext").as_deref(), Some("6"));

    let azimuth_slider = find_slider("input[aria-label='lighting azimuth']");
    assert_eq!(azimuth_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(azimuth_slider.get_attribute("max").as_deref(), Some("360"));
    assert_eq!(azimuth_slider.value(), "235");
    assert_eq!(
        azimuth_slider.get_attribute("aria-valuetext").as_deref(),
        Some("235 degrees"),
        "the raw slider value alone does not carry its own unit"
    );

    let values_caption = find_text("surfaceScale 6 · azimuth 235°");

    // --- moving surfaceScale updates all four primitives, their own caption, and its own aria-valuetext, but
    // touches none of the four light sources' own azimuth or elevation ---
    dispatch_input(&scale_slider, "13");
    for node in [&diffuse_only, &specular_only, &bevel_diffuse, &bevel_specular] {
        assert_eq!(node.get_attribute("surfaceScale").as_deref(), Some("13"));
    }
    assert_eq!(scale_slider.get_attribute("aria-valuetext").as_deref(), Some("13"));
    assert_eq!(values_caption.text_content().as_deref(), Some("surfaceScale 13 · azimuth 235°"));
    for light in [
        &diffuse_only_light,
        &specular_only_light,
        &bevel_diffuse_light,
        &bevel_specular_light,
    ] {
        assert_eq!(
            light.get_attribute("azimuth").as_deref(),
            Some("235"),
            "moving surfaceScale should not touch azimuth"
        );
        assert_eq!(light.get_attribute("elevation").as_deref(), Some("55"));
    }

    // --- moving azimuth updates all four light sources, their own caption, and its own aria-valuetext, but
    // touches none of the four primitives' own surfaceScale. The shared caption also keeps surfaceScale's own
    // last value (13), not the demo's own original default (6), proving each slider's own handler reads the
    // other slider's own current value rather than a stale constant. ---
    dispatch_input(&azimuth_slider, "90");
    for light in [
        &diffuse_only_light,
        &specular_only_light,
        &bevel_diffuse_light,
        &bevel_specular_light,
    ] {
        assert_eq!(light.get_attribute("azimuth").as_deref(), Some("90"));
        assert_eq!(light.get_attribute("elevation").as_deref(), Some("55"));
    }
    assert_eq!(azimuth_slider.get_attribute("aria-valuetext").as_deref(), Some("90 degrees"));
    assert_eq!(values_caption.text_content().as_deref(), Some("surfaceScale 13 · azimuth 90°"));
    for node in [&diffuse_only, &specular_only, &bevel_diffuse, &bevel_specular] {
        assert_eq!(
            node.get_attribute("surfaceScale").as_deref(),
            Some("13"),
            "moving azimuth should not touch surfaceScale"
        );
    }

    // --- the diffuse/specular constants stay fixed throughout, since neither slider drives them ---
    assert_eq!(diffuse_only.get_attribute("diffuseConstant").as_deref(), Some("1"));
    assert_eq!(specular_only.get_attribute("specularConstant").as_deref(), Some("1"));
    assert_eq!(specular_only.get_attribute("specularExponent").as_deref(), Some("20"));
}
