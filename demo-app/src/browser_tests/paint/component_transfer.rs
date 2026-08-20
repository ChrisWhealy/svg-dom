//! Tests for `demo_component_transfer`'s own three sliders: gamma exponent, discrete step count, and alpha slope.

use crate::browser_tests::{container, document};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `component_transfer` returns only the `<feComponentTransfer>` container's own `SvgNode`.
/// It does not return a handle to any `<feFuncX>` child.
/// `gaussian_blur`, `drop_shadow`, and `blend` each return their own primitive node instead.
/// So `demo_component_transfer`'s own three sliders reach their target elements by CSS selector, not by a
/// retained handle.
///
/// Source extraction cannot prove that this wiring works.
/// It cannot prove the gamma and alpha sliders' own scaled values reach the correct fractional attribute.
/// It cannot prove the discrete slider's own `tableValues` stays evenly spaced at every step count.
/// It cannot prove that text matches a fresh build's own text.
/// It cannot prove the three controls stay independent of each other and of the original rectangle.
/// Only a real browser can prove any of that.
#[wasm_bindgen_test]
fn demo_component_transfer_sliders_update_gamma_discrete_and_alpha_independently() -> Result<(), String> {
    container("demo-component-transfer");
    crate::paint::demo_component_transfer::demo()
        .map_err(|e| format!("demo_component_transfer::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-component-transfer")
        .ok_or_else(|| "container exists".to_owned())?;

    let find_el = |selector: &str| -> Result<web_sys::Element, String> {
        root.query_selector(selector)
            .map_err(|e| format!("invalid selector {selector:?}: {e:?}"))?
            .ok_or_else(|| format!("no element matching {selector:?}"))
    };

    let find_all = |selector: &str| -> Result<Vec<web_sys::Element>, String> {
        let list = root
            .query_selector_all(selector)
            .map_err(|e| format!("query elements: {e:?}"))?;
        (0..list.length())
            .map(|i| {
                list.item(i)
                    .ok_or_else(|| "item".to_owned())?
                    .dyn_into::<web_sys::Element>()
                    .map_err(|_| "expected an Element".to_owned())
            })
            .collect()
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

    // --- gamma: three <feFuncR/G/B>, all sharing the slider's own exponent, at this demo's own default ---
    let gamma_funcs = find_all(
        "#component-transfer-gamma feFuncR, #component-transfer-gamma feFuncG, #component-transfer-gamma feFuncB",
    )?;
    if gamma_funcs.len() != 3 {
        return Err(format!("gamma drives all three colour channels, got {}", gamma_funcs.len()));
    }
    for func in &gamma_funcs {
        if func.get_attribute("type").as_deref() != Some("gamma") {
            return Err(format!("expected type \"gamma\", got {:?}", func.get_attribute("type")));
        }
        if func.get_attribute("exponent").as_deref() != Some("2.2") {
            return Err(format!(
                "2.2 is this demo's own initial default, got {:?}",
                func.get_attribute("exponent")
            ));
        }
        if func.get_attribute("amplitude").as_deref() != Some("1") {
            return Err(format!("expected amplitude \"1\", got {:?}", func.get_attribute("amplitude")));
        }
        if func.get_attribute("offset").as_deref() != Some("0") {
            return Err(format!("expected offset \"0\", got {:?}", func.get_attribute("offset")));
        }
    }

    let gamma_slider = find_slider("input[aria-label='component transfer gamma exponent']")?;
    if gamma_slider.get_attribute("min").as_deref() != Some("2") {
        return Err(format!(
            "2 (0.2) is this slider's own documented minimum, got {:?}",
            gamma_slider.get_attribute("min")
        ));
    }
    if gamma_slider.get_attribute("max").as_deref() != Some("50") {
        return Err(format!(
            "50 (5.0) is this slider's own documented maximum, got {:?}",
            gamma_slider.get_attribute("max")
        ));
    }
    if gamma_slider.value() != "22" {
        return Err(format!("expected gamma_slider value \"22\", got {:?}", gamma_slider.value()));
    }
    if gamma_slider.get_attribute("aria-valuetext").as_deref() != Some("2.2") {
        return Err(format!(
            "expected aria-valuetext \"2.2\", got {:?}",
            gamma_slider.get_attribute("aria-valuetext")
        ));
    }

    let gamma_caption = find_text("Gamma(2.2)")?;

    // --- discrete: three <feFuncR/G/B>, all sharing the slider's own step count, at this demo's own default ---
    let discrete_funcs = find_all(
        "#component-transfer-discrete feFuncR, #component-transfer-discrete feFuncG, \
         #component-transfer-discrete feFuncB",
    )?;
    if discrete_funcs.len() != 3 {
        return Err(format!(
            "discrete drives all three colour channels, got {}",
            discrete_funcs.len()
        ));
    }
    for func in &discrete_funcs {
        if func.get_attribute("type").as_deref() != Some("discrete") {
            return Err(format!("expected type \"discrete\", got {:?}", func.get_attribute("type")));
        }
        if func.get_attribute("tableValues").as_deref() != Some("0 0.333 0.667 1") {
            return Err(format!(
                "4 evenly spaced steps is this demo's own initial default, got {:?}",
                func.get_attribute("tableValues")
            ));
        }
    }

    let discrete_slider = find_slider("input[aria-label='component transfer discrete step count']")?;
    if discrete_slider.get_attribute("min").as_deref() != Some("2") {
        return Err(format!(
            "2 is this slider's own documented minimum, got {:?}",
            discrete_slider.get_attribute("min")
        ));
    }
    if discrete_slider.get_attribute("max").as_deref() != Some("8") {
        return Err(format!(
            "8 is this slider's own documented maximum, got {:?}",
            discrete_slider.get_attribute("max")
        ));
    }
    if discrete_slider.value() != "4" {
        return Err(format!(
            "expected discrete_slider value \"4\", got {:?}",
            discrete_slider.value()
        ));
    }

    let discrete_caption = find_text("Discrete(4-step)")?;

    // --- alpha: a single <feFuncA>, untouched by either channel above, at this demo's own default ---
    let alpha_func = find_el("#component-transfer-alpha feFuncA")?;
    if alpha_func.get_attribute("type").as_deref() != Some("linear") {
        return Err(format!("expected type \"linear\", got {:?}", alpha_func.get_attribute("type")));
    }
    if alpha_func.get_attribute("slope").as_deref() != Some("0.4") {
        return Err(format!(
            "0.4 is this demo's own initial default, got {:?}",
            alpha_func.get_attribute("slope")
        ));
    }
    if alpha_func.get_attribute("intercept").as_deref() != Some("0") {
        return Err(format!(
            "intercept stays fixed at 0.0. Only slope is exposed to the slider, got {:?}",
            alpha_func.get_attribute("intercept")
        ));
    }

    let alpha_slider = find_slider("input[aria-label='component transfer alpha slope']")?;
    if alpha_slider.get_attribute("min").as_deref() != Some("0") {
        return Err(format!(
            "expected alpha_slider min \"0\", got {:?}",
            alpha_slider.get_attribute("min")
        ));
    }
    if alpha_slider.get_attribute("max").as_deref() != Some("100") {
        return Err(format!(
            "expected alpha_slider max \"100\", got {:?}",
            alpha_slider.get_attribute("max")
        ));
    }
    if alpha_slider.value() != "40" {
        return Err(format!("expected alpha_slider value \"40\", got {:?}", alpha_slider.value()));
    }
    if alpha_slider.get_attribute("aria-valuetext").as_deref() != Some("0.4") {
        return Err(format!(
            "expected aria-valuetext \"0.4\", got {:?}",
            alpha_slider.get_attribute("aria-valuetext")
        ));
    }

    let alpha_caption = find_text("Alpha Linear(0.4)")?;

    // --- moving gamma to its documented minimum and maximum updates only the gamma channels and caption ---
    dispatch_input(&gamma_slider, "2")?; // 0.2
    for func in &gamma_funcs {
        if func.get_attribute("exponent").as_deref() != Some("0.2") {
            return Err(format!("expected exponent \"0.2\", got {:?}", func.get_attribute("exponent")));
        }
    }
    if gamma_slider.get_attribute("aria-valuetext").as_deref() != Some("0.2") {
        return Err(format!(
            "expected aria-valuetext \"0.2\", got {:?}",
            gamma_slider.get_attribute("aria-valuetext")
        ));
    }
    if gamma_caption.text_content().as_deref() != Some("Gamma(0.2)") {
        return Err(format!(
            "expected caption \"Gamma(0.2)\", got {:?}",
            gamma_caption.text_content()
        ));
    }

    dispatch_input(&gamma_slider, "50")?; // 5.0
    for func in &gamma_funcs {
        if func.get_attribute("exponent").as_deref() != Some("5") {
            return Err(format!(
                "5.0 prints as a bare \"5\". This matches component_transfer's own construction-time Display \
                 formatting. Got {:?}",
                func.get_attribute("exponent")
            ));
        }
    }
    if gamma_slider.get_attribute("aria-valuetext").as_deref() != Some("5.0") {
        return Err(format!(
            "expected aria-valuetext \"5.0\", got {:?}",
            gamma_slider.get_attribute("aria-valuetext")
        ));
    }
    if gamma_caption.text_content().as_deref() != Some("Gamma(5.0)") {
        return Err(format!(
            "expected caption \"Gamma(5.0)\", got {:?}",
            gamma_caption.text_content()
        ));
    }

    for func in &discrete_funcs {
        if func.get_attribute("tableValues").as_deref() != Some("0 0.333 0.667 1") {
            return Err(format!(
                "moving the gamma slider should not touch the discrete channels, got {:?}",
                func.get_attribute("tableValues")
            ));
        }
    }
    if alpha_func.get_attribute("slope").as_deref() != Some("0.4") {
        return Err(format!(
            "moving the gamma slider should not touch the alpha channel, got {:?}",
            alpha_func.get_attribute("slope")
        ));
    }

    // --- moving discrete to its documented minimum and maximum updates only the discrete channels and caption ---
    dispatch_input(&discrete_slider, "2")?;
    for func in &discrete_funcs {
        if func.get_attribute("tableValues").as_deref() != Some("0 1") {
            return Err(format!(
                "expected tableValues \"0 1\", got {:?}",
                func.get_attribute("tableValues")
            ));
        }
    }
    if discrete_caption.text_content().as_deref() != Some("Discrete(2-step)") {
        return Err(format!(
            "expected caption \"Discrete(2-step)\", got {:?}",
            discrete_caption.text_content()
        ));
    }

    dispatch_input(&discrete_slider, "8")?;
    for func in &discrete_funcs {
        if func.get_attribute("tableValues").as_deref() != Some("0 0.143 0.286 0.429 0.571 0.714 0.857 1") {
            return Err(format!(
                "8 evenly spaced steps, each rounded to 3 decimal places, got {:?}",
                func.get_attribute("tableValues")
            ));
        }
    }
    if discrete_caption.text_content().as_deref() != Some("Discrete(8-step)") {
        return Err(format!(
            "expected caption \"Discrete(8-step)\", got {:?}",
            discrete_caption.text_content()
        ));
    }

    for func in &gamma_funcs {
        if func.get_attribute("exponent").as_deref() != Some("5") {
            return Err(format!(
                "moving the discrete slider should not touch the gamma channels. They stay at their own last \
                 value. Got {:?}",
                func.get_attribute("exponent")
            ));
        }
    }
    if alpha_func.get_attribute("slope").as_deref() != Some("0.4") {
        return Err(format!(
            "moving the discrete slider should not touch the alpha channel, got {:?}",
            alpha_func.get_attribute("slope")
        ));
    }

    // --- moving alpha to its documented minimum and maximum updates only the alpha channel and caption ---
    dispatch_input(&alpha_slider, "0")?;
    if alpha_func.get_attribute("slope").as_deref() != Some("0") {
        return Err(format!("expected slope \"0\", got {:?}", alpha_func.get_attribute("slope")));
    }
    if alpha_slider.get_attribute("aria-valuetext").as_deref() != Some("0.0") {
        return Err(format!(
            "expected aria-valuetext \"0.0\", got {:?}",
            alpha_slider.get_attribute("aria-valuetext")
        ));
    }
    if alpha_caption.text_content().as_deref() != Some("Alpha Linear(0.0)") {
        return Err(format!(
            "expected caption \"Alpha Linear(0.0)\", got {:?}",
            alpha_caption.text_content()
        ));
    }

    dispatch_input(&alpha_slider, "100")?;
    if alpha_func.get_attribute("slope").as_deref() != Some("1") {
        return Err(format!("expected slope \"1\", got {:?}", alpha_func.get_attribute("slope")));
    }
    if alpha_slider.get_attribute("aria-valuetext").as_deref() != Some("1.0") {
        return Err(format!(
            "expected aria-valuetext \"1.0\", got {:?}",
            alpha_slider.get_attribute("aria-valuetext")
        ));
    }
    if alpha_caption.text_content().as_deref() != Some("Alpha Linear(1.0)") {
        return Err(format!(
            "expected caption \"Alpha Linear(1.0)\", got {:?}",
            alpha_caption.text_content()
        ));
    }

    for func in &gamma_funcs {
        if func.get_attribute("exponent").as_deref() != Some("5") {
            return Err(format!(
                "moving the alpha slider should not touch the gamma channels. They stay at their own last value. \
                 Got {:?}",
                func.get_attribute("exponent")
            ));
        }
    }
    for func in &discrete_funcs {
        if func.get_attribute("tableValues").as_deref() != Some("0 0.143 0.286 0.429 0.571 0.714 0.857 1") {
            return Err(format!(
                "moving the alpha slider should not touch the discrete channels. They stay at their own last \
                 value. Got {:?}",
                func.get_attribute("tableValues")
            ));
        }
    }

    // --- the original rectangle stays a plain, unfiltered comparison, untouched by every slider above ---
    let rects = root.query_selector_all("rect").map_err(|e| format!("query rects: {e:?}"))?;
    if rects.length() != 4 {
        return Err(format!("one original rectangle and one per slider, got {}", rects.length()));
    }
    let original = rects
        .item(0)
        .ok_or_else(|| "first rect".to_owned())?
        .dyn_into::<web_sys::Element>()
        .map_err(|_| "expected an Element".to_owned())?;
    if original.get_attribute("filter").is_some() {
        return Err("the original rectangle carries no filter".to_owned());
    }
    Ok(())
}
