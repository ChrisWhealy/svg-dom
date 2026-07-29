use std::rc::Rc;

// Only what this file's own helpers (below) directly use. `H`/`W`/`PAD_Y`/`caption`/`colours::*`/`keep_demo_node`
// are crate-root items the *demo* sub-modules need, not this one — each of `demo_text`/`demo_text_path`/
// `demo_tspan` imports those itself, directly from `crate::`, rather than this file re-exporting them via
// `super::` for children to find — re-exporting would make this file's own `use` list satisfy the unused-import
// lint only by accident (whether an item here is "used" would depend on what the children happen to need), not
// because this file's own code (`xhtml`/`foreign_object_document`/`radio_group`/`sine_wave_path`) actually uses it.
use crate::{DemoClosure, NS_XHTML, dom_err, keep_demo_closure};
use svg_dom::{Error, SvgNode};
use wasm_bindgen::{JsCast, prelude::*};

pub(crate) mod demo_text;
pub(crate) mod demo_text_path;
pub(crate) mod demo_tspan;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Shared plumbing for the HTML controls the interactive demos below build inside a <foreignObject>. Kept private
// and local to this file, not hoisted to lib.rs — nothing outside texts.rs needs it.
//
// Deliberately *not* abstracted any further than this: each demo's own `set_text_anchor`/`set_dominant_baseline`/
// `set_start_offset` call stays written out at its own call site (see `radio_group`'s `on_select` parameter) rather
// than being folded into these helpers too. The point of this gallery is the "Rust source" frame each panel shows
// underneath itself — if the actual svg_dom call being demonstrated moved into shared gallery machinery, that frame
// would stop teaching it.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Creates an XHTML-namespaced element — the `create_element_ns(Some(NS_XHTML), tag).map_err(dom_err)` every plain
/// HTML element built inside a `<foreignObject>` in this file needs, shortened to one call.
pub(crate) fn xhtml(document: &web_sys::Document, tag: &str) -> Result<web_sys::Element, Error> {
    document.create_element_ns(Some(NS_XHTML), tag).map_err(dom_err)
}

/// The owner `Document` of an SVG node built via `SvgRoot::foreign_object` — every interactive control in this file
/// needs it, to build the plain HTML it puts inside that `<foreignObject>`.
pub(crate) fn foreign_object_document(fo: &SvgNode) -> Result<web_sys::Document, Error> {
    fo.as_element()
        .owner_document()
        .ok_or_else(|| Error::Dom("cannot identify owner document for SVG foreignObject".into()))
}

/// Builds a `<fieldset class="demo-control-group">` containing a `<legend>` and one
/// `<label class="demo-control-row">`-wrapped `<input type="radio">` per entry in `options`, with `default`
/// pre-checked — the exact shape both radio-driven demos below need. See `demo/style.css`'s `.demo-control-group`
/// rule for why the styling itself lives there rather than as an inline `style` string repeated per call.
///
/// Selecting an option dispatches a real `change` event and, on it, calls `on_select` with that option's value.
/// `on_select` is where each caller writes its own `set_text_anchor`/`set_dominant_baseline`/etc call — this
/// function only builds the shared shape (a labelled, mutually-exclusive HTML radio group wired to *some*
/// callback), never the SVG attribute the callback actually sets; see this module's own doc comment above for why.
///
/// Every closure this creates is parked via [`keep_demo_closure`] so it survives after this function returns.
/// Returns the finished, but not yet attached, `<fieldset>` — the caller still decides where it goes (inside its
/// own `<foreignObject>`) and still calls [`keep_demo_node`] on that `<foreignObject>`'s own `SvgNode`.
pub(crate) fn radio_group<T: Copy + PartialEq + 'static>(
    document: &web_sys::Document,
    legend_text: &str,
    input_name: &str,
    options: &[(T, &str)],
    default: T,
    on_select: impl Fn(T) + 'static,
) -> Result<web_sys::Element, Error> {
    let fieldset = xhtml(document, "fieldset")?;
    fieldset.set_attribute("class", "demo-control-group").map_err(dom_err)?;

    let legend = xhtml(document, "legend")?;
    legend.set_text_content(Some(legend_text));
    fieldset.append_child(&legend).map_err(dom_err)?;

    // Shared across every radio's own listener below, so each one calls the same caller-supplied logic.
    let on_select = Rc::new(on_select);

    for &(value, label) in options {
        let row = xhtml(document, "label")?;
        row.set_attribute("class", "demo-control-row").map_err(dom_err)?;

        let input = xhtml(document, "input")?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| Error::Dom("createElement(\"input\") did not return an HtmlInputElement".into()))?;
        input.set_type("radio");
        input.set_name(input_name);
        input.set_checked(value == default);

        // Each radio's own closure already knows which option value it represents, so on_select needs no lookup —
        // rolling this by hand with SVG elements would have needed to clear the other two selections itself; the
        // browser owns that entirely via the shared `name` above.
        let on_select = on_select.clone();
        let on_change: DemoClosure = Closure::new(move |_: web_sys::Event| on_select(value));
        input
            .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_change);

        row.append_child(&input).map_err(dom_err)?;
        row.append_child(&document.create_text_node(label)).map_err(dom_err)?;
        fieldset.append_child(&row).map_err(dom_err)?;
    }

    Ok(fieldset)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds an SVG path `d` string tracing `periods` cycles of a sine wave, `width` user units wide and `amplitude`
/// tall, starting at `(x0, y0)`.
///
/// Sampled as short straight-line segments (`STEP` user units apart) rather than fitted with Bézier curves — dense
/// enough to read as a smooth curve at demo scale, and far simpler than deriving cubic control points that actually
/// hug a sine function (a circular arc's Bézier approximation constant does not carry over to a sine curve).
pub(crate) fn sine_wave_path(x0: f64, y0: f64, width: f64, amplitude: f64, periods: f64) -> String {
    use std::fmt::Write;
    const STEP: f64 = 4.0;

    let mut path_d = format!("M {x0:.1} {y0:.1}");
    let mut x = STEP;
    while x <= width {
        let y = y0 - amplitude * (2.0 * std::f64::consts::PI * periods * x / width).sin();
        let _ = write!(path_d, " L {:.1} {:.1}", x0 + x, y);
        x += STEP;
    }
    path_d
}
