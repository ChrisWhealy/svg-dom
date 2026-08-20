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
/// beam rather than rotating it, while leaving `y`/`z`/`pointsAtY`/`pointsAtZ` and `feSpecularLighting`'s own
/// unrelated attributes untouched.
/// It cannot prove the cone Spot column's own `limitingConeAngle` slider leaves that same light's own fixed
/// `specularExponent` untouched.
/// It cannot prove the four columns stay independent of one another.
/// Only a real browser can prove any of that.
#[wasm_bindgen_test]
fn demo_light_sources_sliders_update_their_own_light_independently() -> Result<(), String> {
    container("demo-light-sources");
    crate::paint::demo_light_sources::demo()
        .map_err(|e| format!("demo_light_sources::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-light-sources")
        .ok_or_else(|| "container exists".to_owned())?;

    let find_el = |selector: &str| -> Result<web_sys::Element, String> {
        root.query_selector(selector)
            .map_err(|e| format!("invalid selector {selector:?}: {e:?}"))?
            .ok_or_else(|| format!("no element matching {selector:?}"))
    };

    let find_text = |content: &str| -> Result<web_sys::Element, String> {
        let texts = root
            .query_selector_all("text")
            .map_err(|e| format!("query text elements: {e:?}"))?;
        for i in 0..texts.length() {
            let el = texts
                .item(i)
                .ok_or_else(|| "text item".to_owned())?
                .dyn_into::<web_sys::Element>()
                .map_err(|_| "expected an Element".to_owned())?;
            if el.text_content().as_deref() == Some(content) {
                return Ok(el);
            }
        }
        Err(format!("no <text> element with content {content:?}"))
    };

    let find_slider = |aria_label_selector: &str| -> Result<web_sys::HtmlInputElement, String> {
        root.query_selector(aria_label_selector)
            .map_err(|e| format!("query slider: {e:?}"))?
            .ok_or_else(|| format!("no slider matching {aria_label_selector:?}"))?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| "slider is an HtmlInputElement".to_owned())
    };

    let dispatch_input = |slider: &web_sys::HtmlInputElement, value: &str| -> Result<(), String> {
        slider.set_value(value);
        let event = web_sys::Event::new("input").map_err(|e| format!("create input event: {e:?}"))?;
        slider.dispatch_event(&event).map_err(|e| format!("dispatch input: {e:?}"))?;
        Ok(())
    };

    // --- the four retained primitives, at this demo's own initial defaults, all sharing the same fixed surface
    // parameters — none of the four sliders touches any of these ---
    let distant_primitive = find_el("#light-distant feSpecularLighting")?;
    let point_primitive = find_el("#light-point feSpecularLighting")?;
    let spot_open_primitive = find_el("#light-spot-open feSpecularLighting")?;
    let spot_cone_primitive = find_el("#light-spot-cone feSpecularLighting")?;
    for primitive in [&distant_primitive, &point_primitive, &spot_open_primitive, &spot_cone_primitive] {
        if primitive.get_attribute("surfaceScale").as_deref() != Some("2") {
            return Err(format!(
                "expected surfaceScale \"2\", got {:?}",
                primitive.get_attribute("surfaceScale")
            ));
        }
        if primitive.get_attribute("specularConstant").as_deref() != Some("1") {
            return Err(format!(
                "expected specularConstant \"1\", got {:?}",
                primitive.get_attribute("specularConstant")
            ));
        }
        if primitive.get_attribute("specularExponent").as_deref() != Some("8") {
            return Err(format!(
                "expected specularExponent \"8\", got {:?}",
                primitive.get_attribute("specularExponent")
            ));
        }
        if primitive.get_attribute("in").as_deref() != Some("SourceAlpha") {
            return Err(format!("expected in \"SourceAlpha\", got {:?}", primitive.get_attribute("in")));
        }
    }

    // --- every filter's own region is pinned to exactly its rect's own bounding box, removing the default 10%
    // margin's own dark rendering fringe past the rect's edge ---
    for filter_id in ["light-distant", "light-point", "light-spot-open", "light-spot-cone"] {
        let filter = find_el(&format!("#{filter_id}"))?;
        if filter.get_attribute("x").as_deref() != Some("0") {
            return Err(format!("expected x \"0\" on {filter_id}, got {:?}", filter.get_attribute("x")));
        }
        if filter.get_attribute("y").as_deref() != Some("0") {
            return Err(format!("expected y \"0\" on {filter_id}, got {:?}", filter.get_attribute("y")));
        }
        if filter.get_attribute("width").as_deref() != Some("1") {
            return Err(format!(
                "expected width \"1\" on {filter_id}, got {:?}",
                filter.get_attribute("width")
            ));
        }
        if filter.get_attribute("height").as_deref() != Some("1") {
            return Err(format!(
                "expected height \"1\" on {filter_id}, got {:?}",
                filter.get_attribute("height")
            ));
        }
    }

    // --- the four light sources, at this demo's own initial defaults ---
    let distant_light = find_el("#light-distant feDistantLight")?;
    if distant_light.get_attribute("azimuth").as_deref() != Some("235") {
        return Err(format!(
            "fixed throughout -- rotating it has no visible effect on this column's own flat surface, got {:?}",
            distant_light.get_attribute("azimuth")
        ));
    }
    if distant_light.get_attribute("elevation").as_deref() != Some("55") {
        return Err(format!(
            "expected elevation \"55\", got {:?}",
            distant_light.get_attribute("elevation")
        ));
    }

    let point_light = find_el("#light-point fePointLight")?;
    if point_light.get_attribute("x").as_deref() != Some("250") {
        return Err(format!("expected x \"250\", got {:?}", point_light.get_attribute("x")));
    }
    if point_light.get_attribute("y").as_deref() != Some("128") {
        return Err(format!("expected y \"128\", got {:?}", point_light.get_attribute("y")));
    }
    if point_light.get_attribute("z").as_deref() != Some("60") {
        return Err(format!("expected z \"60\", got {:?}", point_light.get_attribute("z")));
    }

    let spot_open_light = find_el("#light-spot-open feSpotLight")?;
    if spot_open_light.get_attribute("x").as_deref() != Some("440") {
        return Err(format!("expected x \"440\", got {:?}", spot_open_light.get_attribute("x")));
    }
    if spot_open_light.get_attribute("y").as_deref() != Some("98") {
        return Err(format!("expected y \"98\", got {:?}", spot_open_light.get_attribute("y")));
    }
    if spot_open_light.get_attribute("z").as_deref() != Some("80") {
        return Err(format!("expected z \"80\", got {:?}", spot_open_light.get_attribute("z")));
    }
    if spot_open_light.get_attribute("pointsAtX").as_deref() != Some("520") {
        return Err(format!(
            "expected pointsAtX \"520\", got {:?}",
            spot_open_light.get_attribute("pointsAtX")
        ));
    }
    if spot_open_light.get_attribute("pointsAtY").as_deref() != Some("168") {
        return Err(format!(
            "expected pointsAtY \"168\", got {:?}",
            spot_open_light.get_attribute("pointsAtY")
        ));
    }
    if spot_open_light.get_attribute("pointsAtZ").as_deref() != Some("0") {
        return Err(format!(
            "expected pointsAtZ \"0\", got {:?}",
            spot_open_light.get_attribute("pointsAtZ")
        ));
    }
    if spot_open_light.get_attribute("specularExponent").as_deref() != Some("2") {
        return Err(format!(
            "fixed throughout -- this attribute has no visible rendered effect in this sandbox's own Chrome, at \
             any value, for this light's own geometry, got {:?}",
            spot_open_light.get_attribute("specularExponent")
        ));
    }
    if spot_open_light.get_attribute("limitingConeAngle").is_some() {
        return Err(
            "the open Spot column's own light must carry no limitingConeAngle at all, the negative control \
             against the cone column"
                .to_owned(),
        );
    }

    let spot_cone_light = find_el("#light-spot-cone feSpotLight")?;
    if spot_cone_light.get_attribute("x").as_deref() != Some("630") {
        return Err(format!("expected x \"630\", got {:?}", spot_cone_light.get_attribute("x")));
    }
    if spot_cone_light.get_attribute("specularExponent").as_deref() != Some("2") {
        return Err(format!(
            "expected specularExponent \"2\", got {:?}",
            spot_cone_light.get_attribute("specularExponent")
        ));
    }
    if spot_cone_light.get_attribute("limitingConeAngle").as_deref() != Some("20") {
        return Err(format!(
            "expected limitingConeAngle \"20\", got {:?}",
            spot_cone_light.get_attribute("limitingConeAngle")
        ));
    }

    // --- the four sliders, at this demo's own initial defaults ---
    let elevation_slider = find_slider("input[aria-label='distant light elevation']")?;
    if elevation_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!("expected min \"0\", got {:?}", elevation_slider.get_attribute("min")));
    }
    if elevation_slider.get_attribute("max").as_deref() != Some("90") {
        return Err(format!("expected max \"90\", got {:?}", elevation_slider.get_attribute("max")));
    }
    if elevation_slider.value() != "55" {
        return Err(format!("expected value \"55\", got {:?}", elevation_slider.value()));
    }
    if elevation_slider.get_attribute("aria-valuetext").as_deref() != Some("55 degrees") {
        return Err(format!(
            "expected aria-valuetext \"55 degrees\", got {:?}",
            elevation_slider.get_attribute("aria-valuetext")
        ));
    }

    let height_slider = find_slider("input[aria-label='point light height']")?;
    if height_slider.get_attribute("min").as_deref() != Some("20") {
        return Err(format!("expected min \"20\", got {:?}", height_slider.get_attribute("min")));
    }
    if height_slider.get_attribute("max").as_deref() != Some("200") {
        return Err(format!("expected max \"200\", got {:?}", height_slider.get_attribute("max")));
    }
    if height_slider.value() != "60" {
        return Err(format!("expected value \"60\", got {:?}", height_slider.value()));
    }
    if height_slider.get_attribute("aria-valuetext").as_deref() != Some("60") {
        return Err(format!(
            "expected aria-valuetext \"60\", got {:?}",
            height_slider.get_attribute("aria-valuetext")
        ));
    }

    let spot_x_slider = find_slider("input[aria-label='spot light horizontal position']")?;
    if spot_x_slider.get_attribute("min").as_deref() != Some("400") {
        return Err(format!("expected min \"400\", got {:?}", spot_x_slider.get_attribute("min")));
    }
    if spot_x_slider.get_attribute("max").as_deref() != Some("560") {
        return Err(format!("expected max \"560\", got {:?}", spot_x_slider.get_attribute("max")));
    }
    if spot_x_slider.value() != "440" {
        return Err(format!("expected value \"440\", got {:?}", spot_x_slider.value()));
    }
    if spot_x_slider.get_attribute("aria-valuetext").as_deref() != Some("440") {
        return Err(format!(
            "expected aria-valuetext \"440\", got {:?}",
            spot_x_slider.get_attribute("aria-valuetext")
        ));
    }

    let cone_angle_slider = find_slider("input[aria-label='spot light limiting cone angle']")?;
    if cone_angle_slider.get_attribute("min").as_deref() != Some("5") {
        return Err(format!(
            "5, not 0 -- 0 renders as a fully open beam in this sandbox's own Chrome, not the near-invisible \
             cutoff a zero-width cone should be, so this slider's own range stays clear of that boundary, got {:?}",
            cone_angle_slider.get_attribute("min")
        ));
    }
    if cone_angle_slider.get_attribute("max").as_deref() != Some("90") {
        return Err(format!("expected max \"90\", got {:?}", cone_angle_slider.get_attribute("max")));
    }
    if cone_angle_slider.value() != "20" {
        return Err(format!("expected value \"20\", got {:?}", cone_angle_slider.value()));
    }
    if cone_angle_slider.get_attribute("aria-valuetext").as_deref() != Some("20 degrees") {
        return Err(format!(
            "expected aria-valuetext \"20 degrees\", got {:?}",
            cone_angle_slider.get_attribute("aria-valuetext")
        ));
    }

    // --- the four initial captions ---
    let distant_caption = find_text("Distant (uniform) elevation 55°")?;
    let point_caption = find_text("Point (hotspot) z 60")?;
    let spot_open_caption = find_text("Spot (no cone) x 440")?;
    let spot_cone_caption = find_text("Spot (cone) 20°")?;

    // --- moving elevation updates only the Distant light's own elevation, leaving its own fixed azimuth, its
    // own caption, and its own aria-valuetext ---
    dispatch_input(&elevation_slider, "15")?;
    if distant_light.get_attribute("elevation").as_deref() != Some("15") {
        return Err(format!(
            "expected elevation \"15\", got {:?}",
            distant_light.get_attribute("elevation")
        ));
    }
    if distant_light.get_attribute("azimuth").as_deref() != Some("235") {
        return Err(format!(
            "moving elevation should not touch this light's own fixed azimuth, got {:?}",
            distant_light.get_attribute("azimuth")
        ));
    }
    if elevation_slider.get_attribute("aria-valuetext").as_deref() != Some("15 degrees") {
        return Err(format!(
            "expected aria-valuetext \"15 degrees\", got {:?}",
            elevation_slider.get_attribute("aria-valuetext")
        ));
    }
    if distant_caption.text_content().as_deref() != Some("Distant (uniform) elevation 15°") {
        return Err(format!(
            "expected caption \"Distant (uniform) elevation 15°\", got {:?}",
            distant_caption.text_content()
        ));
    }
    if point_light.get_attribute("z").as_deref() != Some("60") {
        return Err(format!(
            "moving elevation should not touch the Point column's own light, got {:?}",
            point_light.get_attribute("z")
        ));
    }

    // --- moving height (z) updates only the Point light's own z, leaving its own x/y untouched, along with its
    // own caption and aria-valuetext ---
    dispatch_input(&height_slider, "20")?;
    if point_light.get_attribute("x").as_deref() != Some("250") {
        return Err(format!(
            "z alone should change, not x, got {:?}",
            point_light.get_attribute("x")
        ));
    }
    if point_light.get_attribute("y").as_deref() != Some("128") {
        return Err(format!(
            "z alone should change, not y, got {:?}",
            point_light.get_attribute("y")
        ));
    }
    if point_light.get_attribute("z").as_deref() != Some("20") {
        return Err(format!("expected z \"20\", got {:?}", point_light.get_attribute("z")));
    }
    if height_slider.get_attribute("aria-valuetext").as_deref() != Some("20") {
        return Err(format!(
            "expected aria-valuetext \"20\", got {:?}",
            height_slider.get_attribute("aria-valuetext")
        ));
    }
    if point_caption.text_content().as_deref() != Some("Point (hotspot) z 20") {
        return Err(format!(
            "expected caption \"Point (hotspot) z 20\", got {:?}",
            point_caption.text_content()
        ));
    }
    if distant_light.get_attribute("elevation").as_deref() != Some("15") {
        return Err(format!(
            "moving height should not touch the Distant column's own light, which stays at its own last value, \
             got {:?}",
            distant_light.get_attribute("elevation")
        ));
    }

    // --- moving the Spot (no cone) column's own position (x) slider writes the light's own x, and moves
    // pointsAtX by the same delta, translating the beam without rotating it. y/z, pointsAtY/pointsAtZ, and
    // feSpecularLighting's own identically-scoped attributes, stay untouched ---
    dispatch_input(&spot_x_slider, "560")?;
    if spot_open_light.get_attribute("x").as_deref() != Some("560") {
        return Err(format!("expected x \"560\", got {:?}", spot_open_light.get_attribute("x")));
    }
    if spot_open_light.get_attribute("y").as_deref() != Some("98") {
        return Err(format!(
            "x alone should change, not y, got {:?}",
            spot_open_light.get_attribute("y")
        ));
    }
    if spot_open_light.get_attribute("z").as_deref() != Some("80") {
        return Err(format!(
            "x alone should change, not z, got {:?}",
            spot_open_light.get_attribute("z")
        ));
    }
    if spot_open_light.get_attribute("pointsAtX").as_deref() != Some("640") {
        return Err(format!(
            "pointsAtX must move by the same 80-unit offset as x, translating the beam rather than rotating it -- \
             leaving pointsAtX fixed at 520 while x passes it would swing the beam's own horizontal direction \
             into reverse, got {:?}",
            spot_open_light.get_attribute("pointsAtX")
        ));
    }
    if spot_open_light.get_attribute("pointsAtY").as_deref() != Some("168") {
        return Err(format!(
            "x alone should change, not pointsAtY, got {:?}",
            spot_open_light.get_attribute("pointsAtY")
        ));
    }
    if spot_open_light.get_attribute("pointsAtZ").as_deref() != Some("0") {
        return Err(format!(
            "x alone should change, not pointsAtZ, got {:?}",
            spot_open_light.get_attribute("pointsAtZ")
        ));
    }
    if spot_open_primitive.get_attribute("surfaceScale").as_deref() != Some("2") {
        return Err(format!(
            "the surface's own primitive must stay untouched by the light's own position slider, got {:?}",
            spot_open_primitive.get_attribute("surfaceScale")
        ));
    }
    if spot_open_light.get_attribute("limitingConeAngle").is_some() {
        return Err(
            "the open Spot column's own light must still carry no limitingConeAngle after its own position moves"
                .to_owned(),
        );
    }
    if spot_x_slider.get_attribute("aria-valuetext").as_deref() != Some("560") {
        return Err(format!(
            "expected aria-valuetext \"560\", got {:?}",
            spot_x_slider.get_attribute("aria-valuetext")
        ));
    }
    if spot_open_caption.text_content().as_deref() != Some("Spot (no cone) x 560") {
        return Err(format!(
            "expected caption \"Spot (no cone) x 560\", got {:?}",
            spot_open_caption.text_content()
        ));
    }

    // --- moving the Spot (with cone) column's own cone-angle slider writes the light's own limitingConeAngle,
    // leaving that same light's own specularExponent, and the open Spot column entirely, untouched ---
    dispatch_input(&cone_angle_slider, "60")?;
    if spot_cone_light.get_attribute("limitingConeAngle").as_deref() != Some("60") {
        return Err(format!(
            "expected limitingConeAngle \"60\", got {:?}",
            spot_cone_light.get_attribute("limitingConeAngle")
        ));
    }
    if spot_cone_light.get_attribute("specularExponent").as_deref() != Some("2") {
        return Err(format!(
            "moving cone angle should not touch this same light's own specularExponent, got {:?}",
            spot_cone_light.get_attribute("specularExponent")
        ));
    }
    if cone_angle_slider.get_attribute("aria-valuetext").as_deref() != Some("60 degrees") {
        return Err(format!(
            "expected aria-valuetext \"60 degrees\", got {:?}",
            cone_angle_slider.get_attribute("aria-valuetext")
        ));
    }
    if spot_cone_caption.text_content().as_deref() != Some("Spot (cone) 60°") {
        return Err(format!(
            "expected caption \"Spot (cone) 60°\", got {:?}",
            spot_cone_caption.text_content()
        ));
    }
    if spot_open_light.get_attribute("x").as_deref() != Some("560") {
        return Err(format!(
            "moving the cone column's own slider should not touch the open Spot column, which stays at its own \
             last value, got {:?}",
            spot_open_light.get_attribute("x")
        ));
    }
    Ok(())
}
