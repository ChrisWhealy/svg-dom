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
fn demo_linear_gradient_sliders_update_stops_and_respect_ordering() {
    container("demo-linear-gradient");
    crate::paint::demo_linear_gradient::demo().expect("demo_linear_gradient::demo should build without error");

    let root = document().get_element_by_id("demo-linear-gradient").expect("container exists");

    let dispatch_input = |slider: &web_sys::HtmlInputElement, value: &str| {
        slider.set_value(value);
        let event = web_sys::Event::new("input").expect("create input event");
        slider.dispatch_event(&event).expect("dispatch input");
    };

    let find_el = |selector: &str| -> web_sys::Element {
        root.query_selector(selector)
            .unwrap_or_else(|_| panic!("invalid selector {selector:?}"))
            .unwrap_or_else(|| panic!("no element matching {selector:?}"))
    };

    let find_slider = |aria_label_selector: &str| -> web_sys::HtmlInputElement {
        root.query_selector(aria_label_selector)
            .expect("query slider")
            .unwrap_or_else(|| panic!("no slider matching {aria_label_selector:?}"))
            .dyn_into::<web_sys::HtmlInputElement>()
            .expect("slider is an HtmlInputElement")
    };

    // --- horizontal: the slider shifts #demo-lg-h's second stop ---
    let h_stop = find_el("#demo-lg-h stop:nth-child(2)");
    assert_eq!(h_stop.get_attribute("offset").as_deref(), Some("1"));

    let h_slider = find_slider("input[aria-label='horizontal gradient stop 2']");
    // Set at construction time, before any interaction, so a screen reader announces the real starting value.
    assert_eq!(h_slider.get_attribute("aria-valuetext").as_deref(), Some("100%"));
    dispatch_input(&h_slider, "40");
    assert_eq!(
        h_stop.get_attribute("offset").as_deref(),
        Some("0.40"),
        "moving the horizontal slider should update the second stop's offset"
    );
    assert_eq!(h_slider.get_attribute("aria-valuetext").as_deref(), Some("40%"));

    // --- vertical: the slider shifts #demo-lg-v's second stop ---
    let v_stop = find_el("#demo-lg-v stop:nth-child(2)");
    assert_eq!(v_stop.get_attribute("offset").as_deref(), Some("1"));

    let v_slider = find_slider("input[aria-label=\"shift the vertical gradient's second stop\"]");
    assert_eq!(v_slider.get_attribute("aria-valuetext").as_deref(), Some("100%"));
    assert_eq!(
        v_slider.get_attribute("aria-orientation").as_deref(),
        Some("vertical"),
        "a rotated <input type=range> stays a horizontal slider to assistive technology without this"
    );
    dispatch_input(&v_slider, "25");
    assert_eq!(
        v_stop.get_attribute("offset").as_deref(),
        Some("0.25"),
        "moving the vertical slider should update the second stop's offset"
    );
    assert_eq!(v_slider.get_attribute("aria-valuetext").as_deref(), Some("25%"));

    // The vertical slider's own keydown handler remaps ArrowUp/ArrowDown to match the visual "up is smaller"
    // scale, the opposite of a native horizontal range input's own default ArrowUp-increments behaviour. A
    // synthetic keydown dispatch never triggers a browser's native default action in the first place, so this
    // exercises only the demo's own handler, not any native fallback behaviour.
    let dispatch_keydown = |slider: &web_sys::HtmlInputElement, key: &str| {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(key);
        let event =
            web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).expect("create keydown event");
        slider.dispatch_event(&event).expect("dispatch keydown");
    };

    dispatch_keydown(&v_slider, "ArrowUp");
    assert_eq!(
        v_slider.value(),
        "24",
        "ArrowUp should decrement, matching the visual up-is-smaller scale"
    );
    assert_eq!(v_stop.get_attribute("offset").as_deref(), Some("0.24"));
    assert_eq!(v_slider.get_attribute("aria-valuetext").as_deref(), Some("24%"));

    dispatch_keydown(&v_slider, "ArrowDown");
    assert_eq!(
        v_slider.value(),
        "25",
        "ArrowDown should increment, matching the visual down-is-larger scale"
    );
    assert_eq!(v_stop.get_attribute("offset").as_deref(), Some("0.25"));
    assert_eq!(v_slider.get_attribute("aria-valuetext").as_deref(), Some("25%"));

    // --- diagonal: the slider rotates #demo-lg-d and updates the visible readout ---
    let d_gradient = find_el("#demo-lg-d");
    assert_eq!(
        d_gradient.get_attribute("gradientTransform").as_deref(),
        Some("rotate(45, 0.5, 0.5)")
    );

    // The readout text starts at "rotate 45°", the demo's own initial caption. No id distinguishes it from any
    // other <text>, so it is found by that starting content, the same way other tests in this file find theirs.
    let rotate_readout = {
        let texts = root.query_selector_all("text").expect("query text elements");
        let mut found = None;
        for i in 0..texts.length() {
            let el = texts
                .item(i)
                .expect("text item")
                .dyn_into::<web_sys::Element>()
                .expect("Element");
            if el.text_content().as_deref() == Some("rotate 45°") {
                found = Some(el);
                break;
            }
        }
        found.expect("no <text> element with initial content \"rotate 45°\"")
    };

    let rotate_slider = find_slider("input[aria-label='diagonal gradient rotation']");
    // The slider's min, max, and value all share one coordinate system: the total angle applied to the gradient.
    // Its raw value starts at 45, matching the rendered gradient and the visible readout, not a relative
    // displacement that would need translating before it means anything.
    assert_eq!(rotate_slider.min(), "-45");
    assert_eq!(rotate_slider.max(), "135");
    assert_eq!(rotate_slider.value(), "45");
    assert_eq!(rotate_slider.get_attribute("aria-valuetext").as_deref(), Some("rotate 45°"));
    dispatch_input(&rotate_slider, "15");
    assert_eq!(
        d_gradient.get_attribute("gradientTransform").as_deref(),
        Some("rotate(15, 0.5, 0.5)"),
        "the slider's own value should apply directly as the gradient's rotation angle"
    );
    assert_eq!(rotate_readout.text_content().as_deref(), Some("rotate 15°"));
    assert_eq!(rotate_slider.get_attribute("aria-valuetext").as_deref(), Some("rotate 15°"));

    // --- 4-stop spectrum: the two middle stops stay ordered ---
    let s2_stop = find_el("#demo-lg-s stop:nth-child(2)");
    let s3_stop = find_el("#demo-lg-s stop:nth-child(3)");
    assert_eq!(s2_stop.get_attribute("offset").as_deref(), Some("0.35"));
    assert_eq!(s3_stop.get_attribute("offset").as_deref(), Some("0.65"));

    let s2_slider = find_slider("input[aria-label='spectrum gradient stop 2']");
    let s3_slider = find_slider("input[aria-label='spectrum gradient stop 3']");
    assert_eq!(s2_slider.get_attribute("aria-valuetext").as_deref(), Some("35%"));
    assert_eq!(s3_slider.get_attribute("aria-valuetext").as_deref(), Some("65%"));

    // Each slider's own min/max attribute already exposes the live constraint at construction, before either
    // slider has fired an input event of its own. Stop 2's absolute min (1) and stop 3's absolute max (99) never
    // change; only the shared boundary between the two stops does.
    assert_eq!(s2_slider.min(), "1", "stop 2's absolute lower bound never changes");
    assert_eq!(
        s2_slider.max(),
        "64",
        "stop 2's live upper bound should track stop 3's value minus one"
    );
    assert_eq!(
        s3_slider.min(),
        "36",
        "stop 3's live lower bound should track stop 2's value plus one"
    );
    assert_eq!(s3_slider.max(), "99", "stop 3's absolute upper bound never changes");

    // The visible endpoint labels and tick marks beside each slider must describe the same live range its own
    // min/max attribute does, not the absolute range it was first built with — otherwise the rightmost tick and
    // the "64%" position on screen would silently disagree with what the thumb can actually reach.
    let endpoint_texts = |slider: &web_sys::HtmlInputElement| -> (String, String) {
        let container = slider
            .closest(".demo-slider-container")
            .expect("query closest container")
            .expect("slider has a .demo-slider-container ancestor");
        let labels = container
            .query_selector(".demo-endpoint-labels")
            .expect("query endpoint labels")
            .expect("endpoint-labels row present");
        let spans = labels.query_selector_all("span").expect("query endpoint spans");
        let lo = spans.item(0).expect("lo span").text_content().unwrap_or_default();
        let hi = spans.item(1).expect("hi span").text_content().unwrap_or_default();
        (lo, hi)
    };
    let tick_count = |slider: &web_sys::HtmlInputElement| -> u32 {
        let container = slider
            .closest(".demo-slider-container")
            .expect("query closest container")
            .expect("slider has a .demo-slider-container ancestor");
        let ticks_row = container
            .query_selector(".demo-tick-row")
            .expect("query tick row")
            .expect("tick row present");
        ticks_row
            .query_selector_all(".demo-tick-mark")
            .expect("query tick marks")
            .length()
    };

    assert_eq!(endpoint_texts(&s2_slider), ("1%".to_owned(), "64%".to_owned()));
    assert_eq!(endpoint_texts(&s3_slider), ("36%".to_owned(), "99%".to_owned()));
    assert_eq!(tick_count(&s2_slider), 4, "1..64 in steps of 25, plus a trailing tick at 64");
    assert_eq!(tick_count(&s3_slider), 4, "36..99 in steps of 25, plus a trailing tick at 99");

    // Push stop 2 past stop 3's current value (65). The native max attribute above already stops it at 64,
    // before this demo's own `on_input` handler ever runs.
    dispatch_input(&s2_slider, "70");
    assert_eq!(
        s2_slider.value(),
        "64",
        "the browser's own max attribute should stop stop 2 at one point below stop 3"
    );
    assert_eq!(s2_stop.get_attribute("offset").as_deref(), Some("0.640"));
    assert_eq!(s2_slider.get_attribute("aria-valuetext").as_deref(), Some("64%"));
    assert_eq!(
        s3_stop.get_attribute("offset").as_deref(),
        Some("0.65"),
        "stop 3 must stay put while stop 2 moves"
    );
    assert_eq!(
        s3_slider.min(),
        "65",
        "stop 3's live lower bound should follow stop 2's new value"
    );
    assert_eq!(
        endpoint_texts(&s3_slider),
        ("65%".to_owned(), "99%".to_owned()),
        "stop 3's own visible endpoint label should follow its live min, not stay at the original 36%"
    );
    assert_eq!(
        tick_count(&s3_slider),
        3,
        "65..99 in steps of 25, plus a trailing tick at 99 — fewer ticks fit the narrower live range"
    );

    // Push stop 3 down past stop 2's new current value (64), not its original one (35). The native min
    // attribute, already updated above, stops it at 65 — proving the constraint tracks a live value, not one
    // fixed when the sliders were built.
    dispatch_input(&s3_slider, "50");
    assert_eq!(
        s3_slider.value(),
        "65",
        "the browser's own min attribute should stop stop 3 at one point above stop 2"
    );
    assert_eq!(
        s2_stop.get_attribute("offset").as_deref(),
        Some("0.640"),
        "stop 2 must stay put while stop 3 moves"
    );
    assert_eq!(s3_stop.get_attribute("offset").as_deref(), Some("0.650"));
    assert_eq!(s3_slider.get_attribute("aria-valuetext").as_deref(), Some("65%"));
    assert_eq!(
        s2_slider.max(),
        "64",
        "stop 2's live upper bound should still track stop 3's value minus one"
    );

    // The fixed outer stops never move, however far the middle two are dragged.
    let s1_stop = find_el("#demo-lg-s stop:nth-child(1)");
    let s4_stop = find_el("#demo-lg-s stop:nth-child(4)");
    assert_eq!(s1_stop.get_attribute("offset").as_deref(), Some("0"));
    assert_eq!(s4_stop.get_attribute("offset").as_deref(), Some("1"));

    // --- gradient stroke: untouched by every slider above ---
    let stroke_stop_1 = find_el("#demo-lg-stroke stop:nth-child(1)");
    let stroke_stop_2 = find_el("#demo-lg-stroke stop:nth-child(2)");
    assert_eq!(stroke_stop_1.get_attribute("offset").as_deref(), Some("0"));
    assert_eq!(stroke_stop_1.get_attribute("stop-color").as_deref(), Some("mediumseagreen"));
    assert_eq!(stroke_stop_2.get_attribute("offset").as_deref(), Some("1"));
    assert_eq!(stroke_stop_2.get_attribute("stop-color").as_deref(), Some("coral"));
}
