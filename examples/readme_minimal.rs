//! Compile-checked mirror of the "Minimal Demo" in `README.md`.
//!
//! CI builds this for `wasm32-unknown-unknown` so the README example cannot silently rot — in particular it guards the
//! `#[wasm_bindgen(start)] -> Result<(), JsValue>` boundary (an exported function's error type must be `Into<JsValue>`,
//! which `svg_dom::Error` is not).
//!
//! Keep this in sync with the README. The empty `main` exists only so cargo can build this as an example binary; the
//! real entry point is the `#[wasm_bindgen(start)]` function, exactly as a real app would write it.

use svg_dom::{
    AnimationLoop, SvgAttrs, SvgRoot,
    root::utils::{Point, Size},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    build().map_err(|e| JsValue::from_str(&e.to_string()))
}

fn build() -> Result<(), svg_dom::Error> {
    // Attach to <svg id="diagram"> already present in index.html.
    let svg = SvgRoot::attach("diagram")?;

    // Add a rectangle and give it a colour.
    let rect = svg.rect(Point::new(20.0, 20.0), Size::new(160.0, 80.0))?;
    let mut attrs = SvgAttrs::new();
    rect.attrs(&mut attrs)
        .fill("steelblue")?
        .stroke("white")?
        .stroke_width(2.0)?;

    // Clone the handle so the event closure can refer to the same DOM node.
    let rect_out = rect.clone();
    rect.on_pointerenter(move |_evt| {
        let _ = rect_out.set_fill("gold");
    })?;

    let rect_back = rect.clone();
    rect.on_pointerleave(move |_evt| {
        let _ = rect_back.set_fill("steelblue");
    })?;

    // Build a <g> group containing a circle and a label.
    let group = svg.group()?;
    let dot = svg.circle(Point::new(200.0, 60.0), 8.0)?;
    let label = svg.text(Point::new(215.0, 65.0), "node A")?;
    group.append(&dot)?;
    group.append(&label)?;

    // Animate: pulse the circle radius each frame. The AnimationLoop must outlive this function, so it is leaked.
    let anim = AnimationLoop::start_with_frame(move |ts, frame| {
        let r = 8.0 + 4.0 * (ts / 500.0).sin();
        let _ = frame.set_attr_fmt(&dot, "r", format_args!("{r}"));
    })?;
    std::mem::forget(anim);

    Ok(())
}

fn main() {}
