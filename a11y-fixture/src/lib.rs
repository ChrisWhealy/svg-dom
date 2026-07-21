//! WASM fixture for the `tests/accessibility_tree.rs` Chrome-DevTools-Protocol integration test.
//!
//! Builds a handful of SVG elements exercising `set_title`/`set_desc` against real accessible-name/description
//! computation rules (ARIA precedence, blank-value rejection) so the test can inspect the actual browser-computed
//! accessibility tree via CDP, not just the resulting DOM shape.
//!
//! Every element gets an explicit `role="img"` so Chrome always creates an accessibility-tree node for it,
//! regardless of any SVG-specific pruning heuristics that might otherwise apply to a plain, otherwise-unremarkable
//! shape.

use svg_dom::{Error, SvgRoot, root::utils::Point};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    build().map_err(|e| JsValue::from_str(&e.to_string()))
}

fn build() -> Result<(), Error> {
    let svg = SvgRoot::create_in("stage", svg_dom::root::utils::Size::new(200.0, 200.0))?;

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

    // Signals to the driving test (polling via `wait_for_element`) that the fixture has finished building.
    let ready = svg.rect(Point::new(0.0, 0.0), svg_dom::root::utils::Size::new(1.0, 1.0))?;
    ready.as_element().set_id("fixture-ready");

    Ok(())
}

fn js_to_dom_err(e: JsValue) -> Error {
    Error::Dom(format!("{e:?}"))
}
