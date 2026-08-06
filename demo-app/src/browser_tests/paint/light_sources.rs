//! Tests for `demo_light_sources`'s own four independent sliders, one per `LightSource` variant column.

use crate::browser_tests::{container, document};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `specular_lighting_with_light` returns both the `<feSpecularLighting>` primitive and its light-source child as
/// one `LightingNodes`, so `demo_light_sources` retains both for each of its four columns, the same retained-node
/// pattern `demo_lighting`'s own sliders use.
/// Source extraction cannot prove any of the four sliders actually reaches its own retained light-source node.
/// It cannot prove the Distant column's own `elevation` slider leaves `azimuth` untouched, at its own fixed
/// value that has no rendered effect for this column's own flat surface.
/// It cannot prove the Point column's own `z` slider leaves `x`/`y` untouched.
/// It cannot prove the open Spot column's own `x` slider moves `pointsAtX` by the same delta, translating the
/// beam rather than rotating it, while leaving `y`/`z` and `feSpecularLighting`'s own unrelated attributes
/// untouched.
/// It cannot prove the cone Spot column's own `limitingConeAngle` slider leaves that same light's own fixed
/// `specularExponent` untouched.
/// It cannot prove the four columns stay independent of one another.
/// Only a real browser can prove any of that.
#[wasm_bindgen_test]
fn demo_light_sources_sliders_update_their_own_light_independently() {
    container("demo-light-sources");
    crate::paint::demo_light_sources::demo().expect("demo_light_sources::demo should build without error");

    let root = document().get_element_by_id("demo-light-sources").expect("container exists");

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

    // --- the four retained primitives, at this demo's own initial defaults, all sharing the same fixed surface
    // parameters — none of the four sliders touches any of these ---
    let distant_primitive = find_el("#light-distant feSpecularLighting");
    let point_primitive = find_el("#light-point feSpecularLighting");
    let spot_open_primitive = find_el("#light-spot-open feSpecularLighting");
    let spot_cone_primitive = find_el("#light-spot-cone feSpecularLighting");
    for primitive in [&distant_primitive, &point_primitive, &spot_open_primitive, &spot_cone_primitive] {
        assert_eq!(primitive.get_attribute("surfaceScale").as_deref(), Some("2"));
        assert_eq!(primitive.get_attribute("specularConstant").as_deref(), Some("1"));
        assert_eq!(primitive.get_attribute("specularExponent").as_deref(), Some("8"));
        assert_eq!(primitive.get_attribute("in").as_deref(), Some("SourceAlpha"));
    }

    // --- every filter's own region is pinned to exactly its rect's own bounding box, removing the default 10%
    // margin's own dark rendering fringe past the rect's edge ---
    for filter_id in ["light-distant", "light-point", "light-spot-open", "light-spot-cone"] {
        let filter = find_el(&format!("#{filter_id}"));
        assert_eq!(filter.get_attribute("x").as_deref(), Some("0"));
        assert_eq!(filter.get_attribute("y").as_deref(), Some("0"));
        assert_eq!(filter.get_attribute("width").as_deref(), Some("1"));
        assert_eq!(filter.get_attribute("height").as_deref(), Some("1"));
    }

    // --- the four light sources, at this demo's own initial defaults ---
    let distant_light = find_el("#light-distant feDistantLight");
    assert_eq!(
        distant_light.get_attribute("azimuth").as_deref(),
        Some("235"),
        "fixed throughout -- rotating it has no visible effect on this column's own flat surface"
    );
    assert_eq!(distant_light.get_attribute("elevation").as_deref(), Some("55"));

    let point_light = find_el("#light-point fePointLight");
    assert_eq!(point_light.get_attribute("x").as_deref(), Some("250"));
    assert_eq!(point_light.get_attribute("y").as_deref(), Some("128"));
    assert_eq!(point_light.get_attribute("z").as_deref(), Some("60"));

    let spot_open_light = find_el("#light-spot-open feSpotLight");
    assert_eq!(spot_open_light.get_attribute("x").as_deref(), Some("440"));
    assert_eq!(spot_open_light.get_attribute("y").as_deref(), Some("98"));
    assert_eq!(spot_open_light.get_attribute("pointsAtX").as_deref(), Some("520"));
    assert_eq!(
        spot_open_light.get_attribute("specularExponent").as_deref(),
        Some("2"),
        "fixed throughout -- this attribute has no visible rendered effect in this sandbox's own Chrome, at any \
         value, for this light's own geometry"
    );
    assert!(
        spot_open_light.get_attribute("limitingConeAngle").is_none(),
        "the open Spot column's own light must carry no limitingConeAngle at all, the negative control against \
         the cone column"
    );

    let spot_cone_light = find_el("#light-spot-cone feSpotLight");
    assert_eq!(spot_cone_light.get_attribute("x").as_deref(), Some("630"));
    assert_eq!(spot_cone_light.get_attribute("specularExponent").as_deref(), Some("2"));
    assert_eq!(spot_cone_light.get_attribute("limitingConeAngle").as_deref(), Some("20"));

    // --- the four sliders, at this demo's own initial defaults ---
    let elevation_slider = find_slider("input[aria-label='distant light elevation']");
    assert_eq!(elevation_slider.get_attribute("min").as_deref(), Some("0"));
    assert_eq!(elevation_slider.get_attribute("max").as_deref(), Some("90"));
    assert_eq!(elevation_slider.value(), "55");
    assert_eq!(elevation_slider.get_attribute("aria-valuetext").as_deref(), Some("55 degrees"));

    let height_slider = find_slider("input[aria-label='point light height']");
    assert_eq!(height_slider.get_attribute("min").as_deref(), Some("20"));
    assert_eq!(height_slider.get_attribute("max").as_deref(), Some("200"));
    assert_eq!(height_slider.value(), "60");
    assert_eq!(height_slider.get_attribute("aria-valuetext").as_deref(), Some("60"));

    let spot_x_slider = find_slider("input[aria-label='spot light horizontal position']");
    assert_eq!(spot_x_slider.get_attribute("min").as_deref(), Some("400"));
    assert_eq!(spot_x_slider.get_attribute("max").as_deref(), Some("560"));
    assert_eq!(spot_x_slider.value(), "440");
    assert_eq!(spot_x_slider.get_attribute("aria-valuetext").as_deref(), Some("440"));

    let cone_angle_slider = find_slider("input[aria-label='spot light limiting cone angle']");
    assert_eq!(
        cone_angle_slider.get_attribute("min").as_deref(),
        Some("5"),
        "5, not 0 -- 0 renders as a fully open beam in this sandbox's own Chrome, not the near-invisible cutoff \
         a zero-width cone should be, so this slider's own range stays clear of that boundary"
    );
    assert_eq!(cone_angle_slider.get_attribute("max").as_deref(), Some("90"));
    assert_eq!(cone_angle_slider.value(), "20");
    assert_eq!(cone_angle_slider.get_attribute("aria-valuetext").as_deref(), Some("20 degrees"));

    // --- the four initial captions ---
    let distant_caption = find_text("Distant (uniform) elevation 55°");
    let point_caption = find_text("Point (hotspot) z 60");
    let spot_open_caption = find_text("Spot (no cone) x 440");
    let spot_cone_caption = find_text("Spot (cone) 20°");

    // --- moving elevation updates only the Distant light's own elevation, leaving its own fixed azimuth, its
    // own caption, and its own aria-valuetext ---
    dispatch_input(&elevation_slider, "15");
    assert_eq!(distant_light.get_attribute("elevation").as_deref(), Some("15"));
    assert_eq!(
        distant_light.get_attribute("azimuth").as_deref(),
        Some("235"),
        "moving elevation should not touch this light's own fixed azimuth"
    );
    assert_eq!(elevation_slider.get_attribute("aria-valuetext").as_deref(), Some("15 degrees"));
    assert_eq!(
        distant_caption.text_content().as_deref(),
        Some("Distant (uniform) elevation 15°")
    );
    assert_eq!(
        point_light.get_attribute("z").as_deref(),
        Some("60"),
        "moving elevation should not touch the Point column's own light"
    );

    // --- moving height (z) updates only the Point light's own z, leaving its own x/y untouched, along with its
    // own caption and aria-valuetext ---
    dispatch_input(&height_slider, "20");
    assert_eq!(
        point_light.get_attribute("x").as_deref(),
        Some("250"),
        "z alone should change, not x"
    );
    assert_eq!(
        point_light.get_attribute("y").as_deref(),
        Some("128"),
        "z alone should change, not y"
    );
    assert_eq!(point_light.get_attribute("z").as_deref(), Some("20"));
    assert_eq!(height_slider.get_attribute("aria-valuetext").as_deref(), Some("20"));
    assert_eq!(point_caption.text_content().as_deref(), Some("Point (hotspot) z 20"));
    assert_eq!(
        distant_light.get_attribute("elevation").as_deref(),
        Some("15"),
        "moving height should not touch the Distant column's own light, which stays at its own last value"
    );

    // --- moving the Spot (no cone) column's own position (x) slider writes the light's own x, and moves
    // pointsAtX by the same delta, translating the beam without rotating it. y/z, and feSpecularLighting's own
    // identically-scoped attributes, stay untouched ---
    dispatch_input(&spot_x_slider, "560");
    assert_eq!(spot_open_light.get_attribute("x").as_deref(), Some("560"));
    assert_eq!(
        spot_open_light.get_attribute("y").as_deref(),
        Some("98"),
        "x alone should change, not y"
    );
    assert_eq!(
        spot_open_light.get_attribute("pointsAtX").as_deref(),
        Some("640"),
        "pointsAtX must move by the same 80-unit offset as x, translating the beam rather than rotating it -- \
         leaving pointsAtX fixed at 520 while x passes it would swing the beam's own horizontal direction into \
         reverse"
    );
    assert_eq!(
        spot_open_primitive.get_attribute("surfaceScale").as_deref(),
        Some("2"),
        "the surface's own primitive must stay untouched by the light's own position slider"
    );
    assert!(
        spot_open_light.get_attribute("limitingConeAngle").is_none(),
        "the open Spot column's own light must still carry no limitingConeAngle after its own position moves"
    );
    assert_eq!(spot_x_slider.get_attribute("aria-valuetext").as_deref(), Some("560"));
    assert_eq!(spot_open_caption.text_content().as_deref(), Some("Spot (no cone) x 560"));

    // --- moving the Spot (with cone) column's own cone-angle slider writes the light's own limitingConeAngle,
    // leaving that same light's own specularExponent, and the open Spot column entirely, untouched ---
    dispatch_input(&cone_angle_slider, "60");
    assert_eq!(spot_cone_light.get_attribute("limitingConeAngle").as_deref(), Some("60"));
    assert_eq!(
        spot_cone_light.get_attribute("specularExponent").as_deref(),
        Some("2"),
        "moving cone angle should not touch this same light's own specularExponent"
    );
    assert_eq!(cone_angle_slider.get_attribute("aria-valuetext").as_deref(), Some("60 degrees"));
    assert_eq!(spot_cone_caption.text_content().as_deref(), Some("Spot (cone) 60°"));
    assert_eq!(
        spot_open_light.get_attribute("x").as_deref(),
        Some("560"),
        "moving the cone column's own slider should not touch the open Spot column, which stays at its own last \
         value"
    );
}
