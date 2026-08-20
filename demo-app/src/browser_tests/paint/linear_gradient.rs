//! Tests for `demo_linear_gradient`'s own five sliders.

use crate::browser_tests::{container, document};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `demo_linear_gradient` writes to raw `<stop>` and gradient elements via `select_el`, not through a typed
/// `SvgLinearGradient` handle.
/// Source extraction cannot prove those CSS selectors still resolve.
/// It also cannot prove the two spectrum sliders still keep their middle stops ordered.
/// Only a real browser can prove either.
///
/// The spectrum's mutual constraint gets two explicit boundary checks, not just one.
/// Each slider's own `min`/`max` attribute enforces the constraint, not a JavaScript clamp reacting after the
/// fact. This test checks those attributes directly, not only the values they produce.
/// The second boundary check pushes stop 3 against stop 2's own updated value, not its original one, to prove
/// the live attribute tracks the other slider, not a value fixed at construction time.
///
/// Every slider's `aria-valuetext` is also checked before any interaction.
/// Each one must already report its real starting value there, not only after the first input event.
#[wasm_bindgen_test]
fn demo_linear_gradient_sliders_update_stops_and_respect_ordering() -> Result<(), String> {
    container("demo-linear-gradient");
    crate::paint::demo_linear_gradient::demo()
        .map_err(|e| format!("demo_linear_gradient::demo should build without error: {e:?}"))?;

    let root = document()
        .get_element_by_id("demo-linear-gradient")
        .ok_or_else(|| "container exists".to_owned())?;

    let dispatch_input = |slider: &web_sys::HtmlInputElement, value: &str| -> Result<(), String> {
        slider.set_value(value);
        let event = web_sys::Event::new("input").map_err(|e| format!("create input event: {e:?}"))?;
        slider.dispatch_event(&event).map_err(|e| format!("dispatch input: {e:?}"))?;
        Ok(())
    };

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

    // --- horizontal: the slider shifts #demo-lg-h's second stop ---
    let h_stop = find_el("#demo-lg-h stop:nth-child(2)")?;
    if h_stop.get_attribute("offset").as_deref() != Some("1") {
        return Err(format!("expected offset \"1\", got {:?}", h_stop.get_attribute("offset")));
    }

    let h_slider = find_slider("input[aria-label='horizontal gradient stop 2']")?;
    // Set at construction time, before any interaction, so a screen reader announces the real starting value.
    if h_slider.get_attribute("aria-valuetext").as_deref() != Some("100%") {
        return Err(format!(
            "expected aria-valuetext \"100%\", got {:?}",
            h_slider.get_attribute("aria-valuetext")
        ));
    }
    dispatch_input(&h_slider, "40")?;
    if h_stop.get_attribute("offset").as_deref() != Some("0.40") {
        return Err(format!(
            "moving the horizontal slider should update the second stop's offset, got {:?}",
            h_stop.get_attribute("offset")
        ));
    }
    if h_slider.get_attribute("aria-valuetext").as_deref() != Some("40%") {
        return Err(format!(
            "expected aria-valuetext \"40%\", got {:?}",
            h_slider.get_attribute("aria-valuetext")
        ));
    }

    // --- vertical: the slider shifts #demo-lg-v's second stop ---
    let v_stop = find_el("#demo-lg-v stop:nth-child(2)")?;
    if v_stop.get_attribute("offset").as_deref() != Some("1") {
        return Err(format!("expected offset \"1\", got {:?}", v_stop.get_attribute("offset")));
    }

    let v_slider = find_slider("input[aria-label=\"shift the vertical gradient's second stop\"]")?;
    if v_slider.get_attribute("aria-valuetext").as_deref() != Some("100%") {
        return Err(format!(
            "expected aria-valuetext \"100%\", got {:?}",
            v_slider.get_attribute("aria-valuetext")
        ));
    }
    if v_slider.get_attribute("aria-orientation").as_deref() != Some("vertical") {
        return Err(format!(
            "a rotated <input type=range> stays a horizontal slider to assistive technology without this, got {:?}",
            v_slider.get_attribute("aria-orientation")
        ));
    }
    dispatch_input(&v_slider, "25")?;
    if v_stop.get_attribute("offset").as_deref() != Some("0.25") {
        return Err(format!(
            "moving the vertical slider should update the second stop's offset, got {:?}",
            v_stop.get_attribute("offset")
        ));
    }
    if v_slider.get_attribute("aria-valuetext").as_deref() != Some("25%") {
        return Err(format!(
            "expected aria-valuetext \"25%\", got {:?}",
            v_slider.get_attribute("aria-valuetext")
        ));
    }

    // The vertical slider's own keydown handler remaps ArrowUp/ArrowDown to match the visual "up is smaller"
    // scale, the opposite of a native horizontal range input's own default ArrowUp-increments behaviour. A
    // synthetic keydown dispatch never triggers a browser's native default action in the first place, so this
    // exercises only the demo's own handler, not any native fallback behaviour.
    let dispatch_keydown = |slider: &web_sys::HtmlInputElement, key: &str| -> Result<(), String> {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(key);
        let event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
            .map_err(|e| format!("create keydown event: {e:?}"))?;
        slider.dispatch_event(&event).map_err(|e| format!("dispatch keydown: {e:?}"))?;
        Ok(())
    };

    dispatch_keydown(&v_slider, "ArrowUp")?;
    if v_slider.value() != "24" {
        return Err(format!(
            "ArrowUp should decrement, matching the visual up-is-smaller scale, got {:?}",
            v_slider.value()
        ));
    }
    if v_stop.get_attribute("offset").as_deref() != Some("0.24") {
        return Err(format!("expected offset \"0.24\", got {:?}", v_stop.get_attribute("offset")));
    }
    if v_slider.get_attribute("aria-valuetext").as_deref() != Some("24%") {
        return Err(format!(
            "expected aria-valuetext \"24%\", got {:?}",
            v_slider.get_attribute("aria-valuetext")
        ));
    }

    dispatch_keydown(&v_slider, "ArrowDown")?;
    if v_slider.value() != "25" {
        return Err(format!(
            "ArrowDown should increment, matching the visual down-is-larger scale, got {:?}",
            v_slider.value()
        ));
    }
    if v_stop.get_attribute("offset").as_deref() != Some("0.25") {
        return Err(format!("expected offset \"0.25\", got {:?}", v_stop.get_attribute("offset")));
    }
    if v_slider.get_attribute("aria-valuetext").as_deref() != Some("25%") {
        return Err(format!(
            "expected aria-valuetext \"25%\", got {:?}",
            v_slider.get_attribute("aria-valuetext")
        ));
    }

    // --- diagonal: the slider rotates #demo-lg-d and updates the visible readout ---
    let d_gradient = find_el("#demo-lg-d")?;
    if d_gradient.get_attribute("gradientTransform").as_deref() != Some("rotate(45, 0.5, 0.5)") {
        return Err(format!(
            "expected gradientTransform \"rotate(45, 0.5, 0.5)\", got {:?}",
            d_gradient.get_attribute("gradientTransform")
        ));
    }

    // The readout text starts at "rotate 45°", the demo's own initial caption. No id distinguishes it from any
    // other <text>, so it is found by that starting content, the same way other tests in this file find theirs.
    let rotate_readout = {
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
            if el.text_content().as_deref() == Some("rotate 45°") {
                found = Some(el);
                break;
            }
        }
        found.ok_or_else(|| "no <text> element with initial content \"rotate 45°\"".to_owned())?
    };

    let rotate_slider = find_slider("input[aria-label='diagonal gradient rotation']")?;
    // The slider's min, max, and value all share one coordinate system: the total angle applied to the gradient.
    // Its raw value starts at 45, matching the rendered gradient and the visible readout, not a relative
    // displacement that would need translating before it means anything.
    if rotate_slider.min() != "-45" {
        return Err(format!("expected min \"-45\", got {:?}", rotate_slider.min()));
    }
    if rotate_slider.max() != "135" {
        return Err(format!("expected max \"135\", got {:?}", rotate_slider.max()));
    }
    if rotate_slider.value() != "45" {
        return Err(format!("expected value \"45\", got {:?}", rotate_slider.value()));
    }
    if rotate_slider.get_attribute("aria-valuetext").as_deref() != Some("rotate 45°") {
        return Err(format!(
            "expected aria-valuetext \"rotate 45°\", got {:?}",
            rotate_slider.get_attribute("aria-valuetext")
        ));
    }
    dispatch_input(&rotate_slider, "15")?;
    if d_gradient.get_attribute("gradientTransform").as_deref() != Some("rotate(15, 0.5, 0.5)") {
        return Err(format!(
            "the slider's own value should apply directly as the gradient's rotation angle, got {:?}",
            d_gradient.get_attribute("gradientTransform")
        ));
    }
    if rotate_readout.text_content().as_deref() != Some("rotate 15°") {
        return Err(format!(
            "expected readout \"rotate 15°\", got {:?}",
            rotate_readout.text_content()
        ));
    }
    if rotate_slider.get_attribute("aria-valuetext").as_deref() != Some("rotate 15°") {
        return Err(format!(
            "expected aria-valuetext \"rotate 15°\", got {:?}",
            rotate_slider.get_attribute("aria-valuetext")
        ));
    }

    // --- 4-stop spectrum: the two middle stops stay ordered ---
    let s2_stop = find_el("#demo-lg-s stop:nth-child(2)")?;
    let s3_stop = find_el("#demo-lg-s stop:nth-child(3)")?;
    if s2_stop.get_attribute("offset").as_deref() != Some("0.35") {
        return Err(format!("expected offset \"0.35\", got {:?}", s2_stop.get_attribute("offset")));
    }
    if s3_stop.get_attribute("offset").as_deref() != Some("0.65") {
        return Err(format!("expected offset \"0.65\", got {:?}", s3_stop.get_attribute("offset")));
    }

    let s2_slider = find_slider("input[aria-label='spectrum gradient stop 2']")?;
    let s3_slider = find_slider("input[aria-label='spectrum gradient stop 3']")?;
    if s2_slider.get_attribute("aria-valuetext").as_deref() != Some("35%") {
        return Err(format!(
            "expected aria-valuetext \"35%\", got {:?}",
            s2_slider.get_attribute("aria-valuetext")
        ));
    }
    if s3_slider.get_attribute("aria-valuetext").as_deref() != Some("65%") {
        return Err(format!(
            "expected aria-valuetext \"65%\", got {:?}",
            s3_slider.get_attribute("aria-valuetext")
        ));
    }

    // Each slider's own min/max attribute already exposes the live constraint at construction, before either
    // slider has fired an input event of its own. Stop 2's absolute min (1) and stop 3's absolute max (99) never
    // change; only the shared boundary between the two stops does.
    if s2_slider.min() != "1" {
        return Err(format!(
            "stop 2's absolute lower bound never changes, got {:?}",
            s2_slider.min()
        ));
    }
    if s2_slider.max() != "64" {
        return Err(format!(
            "stop 2's live upper bound should track stop 3's value minus one, got {:?}",
            s2_slider.max()
        ));
    }
    if s3_slider.min() != "36" {
        return Err(format!(
            "stop 3's live lower bound should track stop 2's value plus one, got {:?}",
            s3_slider.min()
        ));
    }
    if s3_slider.max() != "99" {
        return Err(format!(
            "stop 3's absolute upper bound never changes, got {:?}",
            s3_slider.max()
        ));
    }

    // The visible endpoint labels and tick marks beside each slider must describe the same live range its own
    // min/max attribute does, not the absolute range it was first built with — otherwise the rightmost tick and
    // the "64%" position on screen would silently disagree with what the thumb can actually reach.
    let endpoint_texts = |slider: &web_sys::HtmlInputElement| -> Result<(String, String), String> {
        let container = slider
            .closest(".demo-slider-container")
            .map_err(|e| format!("query closest container: {e:?}"))?
            .ok_or_else(|| "slider has a .demo-slider-container ancestor".to_owned())?;
        let labels = container
            .query_selector(".demo-endpoint-labels")
            .map_err(|e| format!("query endpoint labels: {e:?}"))?
            .ok_or_else(|| "endpoint-labels row present".to_owned())?;
        let spans = labels
            .query_selector_all("span")
            .map_err(|e| format!("query endpoint spans: {e:?}"))?;
        let lo = spans
            .item(0)
            .ok_or_else(|| "lo span".to_owned())?
            .text_content()
            .unwrap_or_default();
        let hi = spans
            .item(1)
            .ok_or_else(|| "hi span".to_owned())?
            .text_content()
            .unwrap_or_default();
        Ok((lo, hi))
    };
    let tick_count = |slider: &web_sys::HtmlInputElement| -> Result<u32, String> {
        let container = slider
            .closest(".demo-slider-container")
            .map_err(|e| format!("query closest container: {e:?}"))?
            .ok_or_else(|| "slider has a .demo-slider-container ancestor".to_owned())?;
        let ticks_row = container
            .query_selector(".demo-tick-row")
            .map_err(|e| format!("query tick row: {e:?}"))?
            .ok_or_else(|| "tick row present".to_owned())?;
        Ok(ticks_row
            .query_selector_all(".demo-tick-mark")
            .map_err(|e| format!("query tick marks: {e:?}"))?
            .length())
    };

    if endpoint_texts(&s2_slider)? != ("1%".to_owned(), "64%".to_owned()) {
        return Err(format!("unexpected s2 endpoint texts: {:?}", endpoint_texts(&s2_slider)?));
    }
    if endpoint_texts(&s3_slider)? != ("36%".to_owned(), "99%".to_owned()) {
        return Err(format!("unexpected s3 endpoint texts: {:?}", endpoint_texts(&s3_slider)?));
    }
    if tick_count(&s2_slider)? != 4 {
        return Err(format!(
            "1..64 in steps of 25, plus a trailing tick at 64, got {}",
            tick_count(&s2_slider)?
        ));
    }
    if tick_count(&s3_slider)? != 4 {
        return Err(format!(
            "36..99 in steps of 25, plus a trailing tick at 99, got {}",
            tick_count(&s3_slider)?
        ));
    }

    // Push stop 2 past stop 3's current value (65). The native max attribute above already stops it at 64,
    // before this demo's own `on_input` handler ever runs.
    dispatch_input(&s2_slider, "70")?;
    if s2_slider.value() != "64" {
        return Err(format!(
            "the browser's own max attribute should stop stop 2 at one point below stop 3, got {:?}",
            s2_slider.value()
        ));
    }
    if s2_stop.get_attribute("offset").as_deref() != Some("0.640") {
        return Err(format!("expected offset \"0.640\", got {:?}", s2_stop.get_attribute("offset")));
    }
    if s2_slider.get_attribute("aria-valuetext").as_deref() != Some("64%") {
        return Err(format!(
            "expected aria-valuetext \"64%\", got {:?}",
            s2_slider.get_attribute("aria-valuetext")
        ));
    }
    if s3_stop.get_attribute("offset").as_deref() != Some("0.65") {
        return Err(format!(
            "stop 3 must stay put while stop 2 moves, got {:?}",
            s3_stop.get_attribute("offset")
        ));
    }
    if s3_slider.min() != "65" {
        return Err(format!(
            "stop 3's live lower bound should follow stop 2's new value, got {:?}",
            s3_slider.min()
        ));
    }
    if endpoint_texts(&s3_slider)? != ("65%".to_owned(), "99%".to_owned()) {
        return Err(format!(
            "stop 3's own visible endpoint label should follow its live min, not stay at the original 36%, got \
             {:?}",
            endpoint_texts(&s3_slider)?
        ));
    }
    if tick_count(&s3_slider)? != 3 {
        return Err(format!(
            "65..99 in steps of 25, plus a trailing tick at 99 — fewer ticks fit the narrower live range, got {}",
            tick_count(&s3_slider)?
        ));
    }

    // Push stop 3 down past stop 2's new current value (64), not its original one (35). The native min
    // attribute, already updated above, stops it at 65 — proving the constraint tracks a live value, not one
    // fixed when the sliders were built.
    dispatch_input(&s3_slider, "50")?;
    if s3_slider.value() != "65" {
        return Err(format!(
            "the browser's own min attribute should stop stop 3 at one point above stop 2, got {:?}",
            s3_slider.value()
        ));
    }
    if s2_stop.get_attribute("offset").as_deref() != Some("0.640") {
        return Err(format!(
            "stop 2 must stay put while stop 3 moves, got {:?}",
            s2_stop.get_attribute("offset")
        ));
    }
    if s3_stop.get_attribute("offset").as_deref() != Some("0.650") {
        return Err(format!("expected offset \"0.650\", got {:?}", s3_stop.get_attribute("offset")));
    }
    if s3_slider.get_attribute("aria-valuetext").as_deref() != Some("65%") {
        return Err(format!(
            "expected aria-valuetext \"65%\", got {:?}",
            s3_slider.get_attribute("aria-valuetext")
        ));
    }
    if s2_slider.max() != "64" {
        return Err(format!(
            "stop 2's live upper bound should still track stop 3's value minus one, got {:?}",
            s2_slider.max()
        ));
    }

    // The fixed outer stops never move, however far the middle two are dragged.
    let s1_stop = find_el("#demo-lg-s stop:nth-child(1)")?;
    let s4_stop = find_el("#demo-lg-s stop:nth-child(4)")?;
    if s1_stop.get_attribute("offset").as_deref() != Some("0") {
        return Err(format!("expected offset \"0\", got {:?}", s1_stop.get_attribute("offset")));
    }
    if s4_stop.get_attribute("offset").as_deref() != Some("1") {
        return Err(format!("expected offset \"1\", got {:?}", s4_stop.get_attribute("offset")));
    }

    // --- gradient stroke: untouched by every slider above ---
    let stroke_stop_1 = find_el("#demo-lg-stroke stop:nth-child(1)")?;
    let stroke_stop_2 = find_el("#demo-lg-stroke stop:nth-child(2)")?;
    if stroke_stop_1.get_attribute("offset").as_deref() != Some("0") {
        return Err(format!(
            "expected offset \"0\", got {:?}",
            stroke_stop_1.get_attribute("offset")
        ));
    }
    if stroke_stop_1.get_attribute("stop-color").as_deref() != Some("mediumseagreen") {
        return Err(format!(
            "expected stop-color \"mediumseagreen\", got {:?}",
            stroke_stop_1.get_attribute("stop-color")
        ));
    }
    if stroke_stop_2.get_attribute("offset").as_deref() != Some("1") {
        return Err(format!(
            "expected offset \"1\", got {:?}",
            stroke_stop_2.get_attribute("offset")
        ));
    }
    if stroke_stop_2.get_attribute("stop-color").as_deref() != Some("coral") {
        return Err(format!(
            "expected stop-color \"coral\", got {:?}",
            stroke_stop_2.get_attribute("stop-color")
        ));
    }
    Ok(())
}
