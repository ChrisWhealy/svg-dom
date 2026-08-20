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
fn demo_lighting_sliders_update_surface_scale_and_azimuth_together() -> Result<(), String> {
    container("demo-lighting");
    crate::paint::demo_lighting::demo()
        .map_err(|e| format!("demo_lighting::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-lighting")
        .ok_or_else(|| "container exists".to_owned())?;

    let find_el = |selector: &str| -> Result<web_sys::Element, String> {
        root.query_selector(selector)
            .map_err(|e| format!("invalid selector {selector:?}: {e:?}"))?
            .ok_or_else(|| format!("no element matching {selector:?}"))
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

    // --- the four retained lighting primitives, at this demo's own initial defaults ---
    let diffuse_only = find_el("#diffuse-only feDiffuseLighting")?;
    let specular_only = find_el("#specular-only feSpecularLighting")?;
    let bevel_diffuse = find_el("#bevel-highlight feDiffuseLighting")?;
    let bevel_specular = find_el("#bevel-highlight feSpecularLighting")?;

    for node in [&diffuse_only, &specular_only, &bevel_diffuse, &bevel_specular] {
        if node.get_attribute("surfaceScale").as_deref() != Some("6") {
            return Err(format!(
                "6 is this demo's own initial default surfaceScale, got {:?}",
                node.get_attribute("surfaceScale")
            ));
        }
    }
    if diffuse_only.get_attribute("in").as_deref() != Some("SourceAlpha") {
        return Err(format!(
            "expected in=\"SourceAlpha\", got {:?}",
            diffuse_only.get_attribute("in")
        ));
    }
    if specular_only.get_attribute("in").as_deref() != Some("SourceAlpha") {
        return Err(format!(
            "expected in=\"SourceAlpha\", got {:?}",
            specular_only.get_attribute("in")
        ));
    }

    // --- the combined bevel's own filter graph: diffuse multiplies over SourceGraphic first, then specular adds
    // on top. A regression swapping either composite's own coefficients, or either lighting primitive's own
    // `result` name, would silently break the recipe without changing either primitive's own surfaceScale. ---
    if bevel_diffuse.get_attribute("in").as_deref() != Some("SourceAlpha") {
        return Err(format!(
            "expected bevel_diffuse in=\"SourceAlpha\", got {:?}",
            bevel_diffuse.get_attribute("in")
        ));
    }
    if bevel_diffuse.get_attribute("result").as_deref() != Some("lit") {
        return Err(format!(
            "expected bevel_diffuse result=\"lit\", got {:?}",
            bevel_diffuse.get_attribute("result")
        ));
    }
    if bevel_specular.get_attribute("in").as_deref() != Some("SourceAlpha") {
        return Err(format!(
            "expected bevel_specular in=\"SourceAlpha\", got {:?}",
            bevel_specular.get_attribute("in")
        ));
    }
    if bevel_specular.get_attribute("result").as_deref() != Some("highlight") {
        return Err(format!(
            "expected bevel_specular result=\"highlight\", got {:?}",
            bevel_specular.get_attribute("result")
        ));
    }

    let bevel_composites = root
        .query_selector_all("#bevel-highlight feComposite")
        .map_err(|e| format!("query feComposite: {e:?}"))?;
    if bevel_composites.length() != 2 {
        return Err(format!(
            "one composite per lighting primitive, got {}",
            bevel_composites.length()
        ));
    }
    let multiply = bevel_composites
        .item(0)
        .ok_or_else(|| "first composite".to_owned())?
        .dyn_into::<web_sys::Element>()
        .map_err(|_| "expected an Element".to_owned())?;
    if multiply.get_attribute("in").as_deref() != Some("SourceGraphic") {
        return Err(format!(
            "expected multiply in=\"SourceGraphic\", got {:?}",
            multiply.get_attribute("in")
        ));
    }
    if multiply.get_attribute("in2").as_deref() != Some("lit") {
        return Err(format!(
            "expected multiply in2=\"lit\", got {:?}",
            multiply.get_attribute("in2")
        ));
    }
    if multiply.get_attribute("result").as_deref() != Some("beveled") {
        return Err(format!(
            "expected multiply result=\"beveled\", got {:?}",
            multiply.get_attribute("result")
        ));
    }
    if multiply.get_attribute("k1").as_deref() != Some("1") {
        return Err(format!("expected multiply k1=\"1\", got {:?}", multiply.get_attribute("k1")));
    }
    let add = bevel_composites
        .item(1)
        .ok_or_else(|| "second composite".to_owned())?
        .dyn_into::<web_sys::Element>()
        .map_err(|_| "expected an Element".to_owned())?;
    if add.get_attribute("in").as_deref() != Some("beveled") {
        return Err(format!("expected add in=\"beveled\", got {:?}", add.get_attribute("in")));
    }
    if add.get_attribute("in2").as_deref() != Some("highlight") {
        return Err(format!("expected add in2=\"highlight\", got {:?}", add.get_attribute("in2")));
    }
    if add.get_attribute("k2").as_deref() != Some("1") {
        return Err(format!("expected add k2=\"1\", got {:?}", add.get_attribute("k2")));
    }
    if add.get_attribute("k3").as_deref() != Some("1") {
        return Err(format!("expected add k3=\"1\", got {:?}", add.get_attribute("k3")));
    }

    // --- the four light sources, one per lighting primitive, disambiguated within the combined bevel filter by
    // their own parent primitive's tag name ---
    let diffuse_only_light = find_el("#diffuse-only feDistantLight")?;
    let specular_only_light = find_el("#specular-only feDistantLight")?;
    let bevel_diffuse_light = find_el("#bevel-highlight feDiffuseLighting feDistantLight")?;
    let bevel_specular_light = find_el("#bevel-highlight feSpecularLighting feDistantLight")?;

    for light in [
        &diffuse_only_light,
        &specular_only_light,
        &bevel_diffuse_light,
        &bevel_specular_light,
    ] {
        if light.get_attribute("azimuth").as_deref() != Some("235") {
            return Err(format!(
                "235 is this demo's own initial default azimuth, got {:?}",
                light.get_attribute("azimuth")
            ));
        }
        if light.get_attribute("elevation").as_deref() != Some("55") {
            return Err(format!(
                "elevation is fixed, not one of this demo's own interactive controls, got {:?}",
                light.get_attribute("elevation")
            ));
        }
    }

    // --- both sliders, at this demo's own initial defaults ---
    let scale_slider = find_slider("input[aria-label='lighting surface scale']")?;
    if scale_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!(
            "expected scale_slider min=\"0\", got {:?}",
            scale_slider.get_attribute("min")
        ));
    }
    if scale_slider.get_attribute("max").as_deref() != Some("20") {
        return Err(format!(
            "expected scale_slider max=\"20\", got {:?}",
            scale_slider.get_attribute("max")
        ));
    }
    if scale_slider.value() != "6" {
        return Err(format!("expected scale_slider value \"6\", got {:?}", scale_slider.value()));
    }
    if scale_slider.get_attribute("aria-valuetext").as_deref() != Some("6") {
        return Err(format!(
            "expected scale_slider aria-valuetext \"6\", got {:?}",
            scale_slider.get_attribute("aria-valuetext")
        ));
    }

    let azimuth_slider = find_slider("input[aria-label='lighting azimuth']")?;
    if azimuth_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!(
            "expected azimuth_slider min=\"0\", got {:?}",
            azimuth_slider.get_attribute("min")
        ));
    }
    if azimuth_slider.get_attribute("max").as_deref() != Some("360") {
        return Err(format!(
            "expected azimuth_slider max=\"360\", got {:?}",
            azimuth_slider.get_attribute("max")
        ));
    }
    if azimuth_slider.value() != "235" {
        return Err(format!(
            "expected azimuth_slider value \"235\", got {:?}",
            azimuth_slider.value()
        ));
    }
    if azimuth_slider.get_attribute("aria-valuetext").as_deref() != Some("235 degrees") {
        return Err(format!(
            "the raw slider value alone does not carry its own unit, got {:?}",
            azimuth_slider.get_attribute("aria-valuetext")
        ));
    }

    let values_caption = {
        let texts = root
            .query_selector_all("text")
            .map_err(|e| format!("query text elements: {e:?}"))?;
        let mut found = None;
        for i in 0..texts.length() {
            let el = texts
                .item(i)
                .ok_or_else(|| "text item".to_owned())?
                .dyn_into::<web_sys::Element>()
                .map_err(|_| "expected an Element".to_owned())?;
            if el.text_content().as_deref() == Some("surfaceScale 6 · azimuth 235°") {
                found = Some(el);
                break;
            }
        }
        found.ok_or_else(|| "no <text> element with content \"surfaceScale 6 · azimuth 235°\"".to_owned())?
    };

    // --- moving surfaceScale updates all four primitives, their own caption, and its own aria-valuetext, but
    // touches none of the four light sources' own azimuth or elevation ---
    dispatch_input(&scale_slider, "13")?;
    for node in [&diffuse_only, &specular_only, &bevel_diffuse, &bevel_specular] {
        if node.get_attribute("surfaceScale").as_deref() != Some("13") {
            return Err(format!(
                "expected surfaceScale \"13\", got {:?}",
                node.get_attribute("surfaceScale")
            ));
        }
    }
    if scale_slider.get_attribute("aria-valuetext").as_deref() != Some("13") {
        return Err(format!(
            "expected scale_slider aria-valuetext \"13\", got {:?}",
            scale_slider.get_attribute("aria-valuetext")
        ));
    }
    if values_caption.text_content().as_deref() != Some("surfaceScale 13 · azimuth 235°") {
        return Err(format!(
            "expected caption \"surfaceScale 13 · azimuth 235°\", got {:?}",
            values_caption.text_content()
        ));
    }
    for light in [
        &diffuse_only_light,
        &specular_only_light,
        &bevel_diffuse_light,
        &bevel_specular_light,
    ] {
        if light.get_attribute("azimuth").as_deref() != Some("235") {
            return Err(format!(
                "moving surfaceScale should not touch azimuth, got {:?}",
                light.get_attribute("azimuth")
            ));
        }
        if light.get_attribute("elevation").as_deref() != Some("55") {
            return Err(format!("expected elevation \"55\", got {:?}", light.get_attribute("elevation")));
        }
    }

    // --- moving azimuth updates all four light sources, their own caption, and its own aria-valuetext, but
    // touches none of the four primitives' own surfaceScale. The shared caption also keeps surfaceScale's own
    // last value (13), not the demo's own original default (6), proving each slider's own handler reads the
    // other slider's own current value rather than a stale constant. ---
    dispatch_input(&azimuth_slider, "90")?;
    for light in [
        &diffuse_only_light,
        &specular_only_light,
        &bevel_diffuse_light,
        &bevel_specular_light,
    ] {
        if light.get_attribute("azimuth").as_deref() != Some("90") {
            return Err(format!("expected azimuth \"90\", got {:?}", light.get_attribute("azimuth")));
        }
        if light.get_attribute("elevation").as_deref() != Some("55") {
            return Err(format!("expected elevation \"55\", got {:?}", light.get_attribute("elevation")));
        }
    }
    if azimuth_slider.get_attribute("aria-valuetext").as_deref() != Some("90 degrees") {
        return Err(format!(
            "expected azimuth_slider aria-valuetext \"90 degrees\", got {:?}",
            azimuth_slider.get_attribute("aria-valuetext")
        ));
    }
    if values_caption.text_content().as_deref() != Some("surfaceScale 13 · azimuth 90°") {
        return Err(format!(
            "expected caption \"surfaceScale 13 · azimuth 90°\", got {:?}",
            values_caption.text_content()
        ));
    }
    for node in [&diffuse_only, &specular_only, &bevel_diffuse, &bevel_specular] {
        if node.get_attribute("surfaceScale").as_deref() != Some("13") {
            return Err(format!(
                "moving azimuth should not touch surfaceScale, got {:?}",
                node.get_attribute("surfaceScale")
            ));
        }
    }

    // --- the diffuse/specular constants stay fixed throughout, since neither slider drives them ---
    if diffuse_only.get_attribute("diffuseConstant").as_deref() != Some("1") {
        return Err(format!(
            "expected diffuseConstant \"1\", got {:?}",
            diffuse_only.get_attribute("diffuseConstant")
        ));
    }
    if specular_only.get_attribute("specularConstant").as_deref() != Some("1") {
        return Err(format!(
            "expected specularConstant \"1\", got {:?}",
            specular_only.get_attribute("specularConstant")
        ));
    }
    if specular_only.get_attribute("specularExponent").as_deref() != Some("20") {
        return Err(format!(
            "expected specularExponent \"20\", got {:?}",
            specular_only.get_attribute("specularExponent")
        ));
    }
    Ok(())
}
