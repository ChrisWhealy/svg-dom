use super::{DemoClosure, H, NS_XHTML, PAD_Y, W, caption, colours::*, keep_demo_closure, keep_demo_node};
// `dom_err` is this crate's own private DOM-error mapper (defined in `lib.rs`) and is not part of `svg_dom`'s public
// API.  It is kept as a separate `use` from the `svg_dom::` import below for exactly that reason (see structure.rs's
// identical split for the same rationale).
use crate::dom_err;
use svg_dom::{
    DominantBaseline, Error, PathDef, PathDefAbsolute, SvgRoot, TextAnchor,
    root::utils::{Point, Size},
};
use wasm_bindgen::{JsCast, prelude::*};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// text
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_text() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-text", Size::new(W, H))?;
    let mut demo_x: f64 = 6.0;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // text-anchor (interactive)
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // A single text element sits on the dashed vertical guide at x=100.
    // Three real HTML `<input type="radio">` elements are laid out inside a `<foreignObject>` within the same
    // SVG-space position.  When the radio buttons change, the text's `text-anchor` is updated live.
    //
    // The use of native HTML radio buttons was chosen over hand-drawn SVG ones specifically because of what they give
    // for free: screen-reader semantics, keyboard navigation, and browser-enforced mutual exclusivity via the shared
    // `name` attribute — all of which a custom-drawn SVG control would have had to implement by hand.
    let vguide = svg.line(Point::new(100.0, 30.0), Point::new(100.0, 66.0))?;
    vguide.set_stroke(GUIDE)?;
    vguide.set_attr("stroke-dasharray", "4 3")?;

    let anchor_text = svg.text(Point::new(100.0, 56.0), "sample text")?;
    anchor_text.set_fill(STEELBLUE)?;
    anchor_text.set_font_size(13.0)?;
    anchor_text.set_text_anchor(TextAnchor::Start)?;

    // Use a foreignObject to house the HTML radio buttons and keep them within the SVG viewport.
    let fo = svg.foreign_object(Point::new(demo_x, 76.0), Size::new(180.0, 72.0))?;
    let document = fo
        .as_element()
        .owner_document()
        .ok_or_else(|| Error::Dom("cannot identify owner document for SVG foreignObject".into()))?;

    // Use a <fieldset>/<legend> pair rather than a plain <div>, so assistive technology gets a programmatic group name
    // ("text-anchor") it can use to tie the three radio buttons together.  The same relationship a sighted user gets
    // for free just by seeing them clustered under a caption. A bare <div> wrapper leaves each radio individually
    // labelled ("start"/"middle"/"end") but with no indication of what value they collectively control.
    let list = document.create_element_ns(Some(NS_XHTML), "fieldset").map_err(dom_err)?;
    list.set_attribute(
        "style",
        "display:flex; flex-direction:column; gap:6px; font:13px sans-serif; color:#c9d1d9; \
         border:none; margin:0; padding:0;",
    )
    .map_err(dom_err)?;

    let legend = document.create_element_ns(Some(NS_XHTML), "legend").map_err(dom_err)?;
    legend
        .set_attribute("style", "font-size:11px; color:#8b949e; padding:0; margin:0 0 4px;")
        .map_err(dom_err)?;
    legend.set_text_content(Some("text-anchor"));
    list.append_child(&legend).map_err(dom_err)?;

    const RADIO_OPTIONS: [(TextAnchor, &str); 3] = [
        (TextAnchor::Start, "start"),
        (TextAnchor::Middle, "middle"),
        (TextAnchor::End, "end"),
    ];

    for (value, label) in RADIO_OPTIONS {
        let row = document.create_element_ns(Some(NS_XHTML), "label").map_err(dom_err)?;
        row.set_attribute("style", "display:flex; align-items:center; gap:6px; cursor:pointer;")
            .map_err(dom_err)?;

        let input = document
            .create_element_ns(Some(NS_XHTML), "input")
            .map_err(dom_err)?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| Error::Dom("createElement(\"input\") did not return an HtmlInputElement".into()))?;
        input.set_type("radio");
        input.set_name("demo-text-anchor");
        input.set_checked(value == TextAnchor::Start);

        // Each radio button's closure already knows which TextAnchor it represents, so the handler needs no lookup.
        // Had we attempted to "roll our own" radio buttons using SCG elements, we would be responsible for writing the
        // logic to clear the other two dots itself.  Using HTML inside a foreignObject gives us all this standard
        // functionality for free.
        let target = anchor_text.clone();
        let on_change: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let _ = target.set_text_anchor(value);
        });
        input
            .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_change);

        row.append_child(&input).map_err(dom_err)?;
        row.append_child(&document.create_text_node(label)).map_err(dom_err)?;
        list.append_child(&row).map_err(dom_err)?;
    }

    fo.as_element().append_child(&list).map_err(dom_err)?;
    keep_demo_node(fo);

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // dominant-baseline (interactive)
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // A single text element sits on the dashed horizontal guide at y=45 — the same "one text element, one fixed
    // anchor coordinate" shape the text-anchor demo above uses, just on the other axis. Three real HTML
    // `<input type="radio">` elements, laid out inside a second `<foreignObject>` below it, set its
    // `dominant-baseline` live on change — see the text-anchor demo above for why native radios were chosen over
    // hand-drawn SVG ones, and `SvgRoot::foreign_object`'s own doc comment for the raw web-sys escape hatch both
    // of these `<foreignObject>`s use.
    demo_x += 209.0;
    let hguide = svg.line(Point::new(demo_x, 45.0), Point::new(demo_x + 150.0, 45.0))?;
    hguide.set_stroke(GUIDE)?;
    hguide.set_attr("stroke-dasharray", "4 3")?;

    let baseline_text = svg.text(Point::new(demo_x + 45.0, 45.0), "baseline")?;
    baseline_text.set_fill(STEELBLUE)?;
    baseline_text.set_font_size(13.0)?;
    baseline_text.set_text_anchor(TextAnchor::Middle)?;
    baseline_text.set_dominant_baseline(DominantBaseline::Alphabetic)?;

    let db_fo = svg.foreign_object(Point::new(demo_x, 76.0), Size::new(185.0, 78.0))?;
    let db_document = db_fo
        .as_element()
        .owner_document()
        .ok_or_else(|| Error::Dom("cannot identify owner document for SVG foreignObject".into()))?;

    // Use a <fieldset>/<legend> again — a plain <div> wrapper is not enough on its own to give assistive technology a
    // programmatic name for what these three radio buttons collectively control.
    let db_list = db_document.create_element_ns(Some(NS_XHTML), "fieldset").map_err(dom_err)?;
    db_list
        .set_attribute(
            "style",
            "display:flex; flex-direction:column; gap:6px; font:13px sans-serif; color:#c9d1d9; \
             border:none; margin:0; padding:0;",
        )
        .map_err(dom_err)?;

    let db_legend = db_document.create_element_ns(Some(NS_XHTML), "legend").map_err(dom_err)?;
    db_legend
        .set_attribute("style", "font-size:11px; color:#8b949e; padding:0; margin:0 0 4px;")
        .map_err(dom_err)?;
    db_legend.set_text_content(Some("dominant-baseline"));
    db_list.append_child(&db_legend).map_err(dom_err)?;

    const BASELINE_OPTIONS: [(DominantBaseline, &str); 3] = [
        (DominantBaseline::Alphabetic, "alphabetic"),
        (DominantBaseline::Middle, "middle"),
        (DominantBaseline::Hanging, "hanging"),
    ];

    for (value, label) in BASELINE_OPTIONS {
        let row = db_document.create_element_ns(Some(NS_XHTML), "label").map_err(dom_err)?;
        row.set_attribute("style", "display:flex; align-items:center; gap:6px; cursor:pointer;")
            .map_err(dom_err)?;

        let input = db_document
            .create_element_ns(Some(NS_XHTML), "input")
            .map_err(dom_err)?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| Error::Dom("createElement(\"input\") did not return an HtmlInputElement".into()))?;
        input.set_type("radio");
        input.set_name("demo-dominant-baseline");
        input.set_checked(value == DominantBaseline::Alphabetic);

        let target = baseline_text.clone();
        let on_change: DemoClosure = Closure::new(move |_: web_sys::Event| {
            let _ = target.set_dominant_baseline(value);
        });
        input
            .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())
            .map_err(dom_err)?;
        keep_demo_closure(on_change);

        row.append_child(&input).map_err(dom_err)?;
        row.append_child(&db_document.create_text_node(label)).map_err(dom_err)?;
        db_list.append_child(&row).map_err(dom_err)?;
    }

    db_fo.as_element().append_child(&db_list).map_err(dom_err)?;
    keep_demo_node(db_fo);

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // font-family
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // The same word rendered in the three CSS generic font families.
    demo_x += 180.0;
    let ff_serif = svg.text(Point::new(demo_x, 30.0), "Serif")?;
    ff_serif.set_fill(PLAIN_TEXT)?;
    ff_serif.set_font_size(18.0)?;
    ff_serif.set_font_family("serif")?;

    let ff_sans = svg.text(Point::new(demo_x, 77.0), "Sans-serif")?;
    ff_sans.set_fill(STEELBLUE)?;
    ff_sans.set_font_size(18.0)?;
    ff_sans.set_font_family("sans-serif")?;

    let ff_mono = svg.text(Point::new(demo_x, 128.0), "Monospace")?;
    ff_mono.set_fill(CORAL)?;
    ff_mono.set_font_size(18.0)?;
    ff_mono.set_font_family("monospace")?;

    caption(&svg, demo_x + 50.0, "font-family")?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // font-size
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Three labels at the same x, each rendered at a different font size.
    demo_x += 150.0;
    let fs_s = svg.text(Point::new(demo_x, 30.0), "small — 11px")?;
    fs_s.set_fill(PLAIN_TEXT)?;
    fs_s.set_font_size(11.0)?;

    let fs_m = svg.text(Point::new(demo_x, 77.0), "medium — 17px")?;
    fs_m.set_fill(PLAIN_TEXT)?;
    fs_m.set_font_size(17.0)?;

    let fs_l = svg.text(Point::new(demo_x, 128.0), "large — 26px")?;
    fs_l.set_fill(PLAIN_TEXT)?;
    fs_l.set_font_size(26.0)?;

    caption(&svg, demo_x + 50.0, "font-size")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// tspan — multi-line and inline mixed-style text
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_tspan() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-tspan", Size::new(W, H))?;

    // ── multi-line (tspan_line) ───────────────────────────────────────────────
    // A <text> with three <tspan> children.  The first inherits x from <text>. Subsequent spans use `tspan_line`, which
    // resets `x` to the same absolute coordinate and advances `y` by `dy`, so every line aligns with the left edge.
    const LINE_X: f64 = 50.0;
    const LINE_DY: f64 = 22.0;
    let ml = svg.text(Point::new(LINE_X, 50.0 + PAD_Y), "")?;
    ml.set_fill(PLAIN_TEXT)?;
    ml.set_font_size(15.0)?;

    ml.tspan("The quick brown fox")?;
    ml.tspan_line(LINE_X, LINE_DY, "jumps over the")?;
    ml.tspan_line(LINE_X, LINE_DY, "lazy dog.")?;

    caption(&svg, 200.0, "multi-line (tspan_line)")?;

    // ── inline mixed styles ───────────────────────────────────────────────────
    // A single <text> element whose <tspan> children each override fill and font-size, producing a mixed-style run on
    // one baseline.
    let mx = svg.text(Point::new(420.0, 90.0 + PAD_Y), "")?;

    let w1 = mx.tspan("small ")?;
    w1.set_fill(PLAIN_TEXT)?;
    w1.set_font_size(12.0)?;
    w1.set_dominant_baseline(DominantBaseline::Middle)?;

    let w2 = mx.tspan("MEDIUM ")?;
    w2.set_fill(STEELBLUE)?;
    w2.set_font_size(18.0)?;
    w2.set_dominant_baseline(DominantBaseline::Middle)?;

    let w3 = mx.tspan("LARGE")?;
    w3.set_fill(CORAL)?;
    w3.set_font_size(26.0)?;
    w3.set_dominant_baseline(DominantBaseline::Middle)?;

    caption(&svg, 600.0, "inline mixed styles (tspan)")?;

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds an SVG path `d` string tracing `periods` cycles of a sine wave, `width` user units wide and `amplitude`
/// tall, starting at `(x0, y0)`.
///
/// Sampled as short straight-line segments (`STEP` user units apart) rather than fitted with Bézier curves — dense
/// enough to read as a smooth curve at demo scale, and far simpler than deriving cubic control points that actually
/// hug a sine function (a circular arc's Bézier approximation constant does not carry over to a sine curve).
fn sine_wave_path(x0: f64, y0: f64, width: f64, amplitude: f64, periods: f64) -> String {
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// textPath — gluing text to a curve
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(super) fn demo_text_path() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-text-path", Size::new(W, H))?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // text follows a sine wave
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // The guide wave lives in <defs> which means it is never rendered unless it is first referenced by `href` — which
    // is the usual way to use a textPath: the geometry is simply a positioning aid for the text.
    //
    // Four periods of the sine wave are defined, but the text only covers part of the path's full length.
    let wave_d = sine_wave_path(20.0, 90.0, 1000.0, 40.0, 4.0);
    let defs = svg.build_defs(|d| {
        d.path(&wave_d)?.set_attr("id", "demo-tp-wave")?;
        Ok(())
    })?;
    let _ = defs; // held only to construct the wave; the id reference below is what matters

    let curve = svg.text(Point::origin(), "")?;
    let curve_path = curve.text_path("#demo-tp-wave", "Now riding a sine wave, up and down")?;
    curve_path.set_fill(PLAIN_TEXT)?;
    curve_path.set_font_size(14.0)?;

    caption(&svg, 200.0, "textPath — follows a sine wave")?;

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // Interactive startOffset
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    // The dashed guide arc is drawn directly on the canvas rather than being defined as a reusable object in <defs>, so
    // the effect of `set_start_offset` is visible. A single `<textPath>` element reads "Offset <n>" and slides along
    // the arc as the native HTML `<input type="range">` (housed in a `<foreignObject>`) is moved. The slider's own
    // `max` is the arc's `total_length()`, truncated to a whole user unit so it can never let the offset run past the
    // path's end.
    let guide = svg.path_from_defs(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(430.0, 130.0))),
        PathDef::Abs(PathDefAbsolute::QuadraticBezierTo(
            Point::new(600.0, 45.0),
            Point::new(770.0, 130.0),
        )),
    ])?;
    guide.set_attr("id", "demo-tp-offset-arc")?;
    guide.set_fill(NONE)?;
    guide.set_stroke(GUIDE)?;
    guide.set_attr("stroke-dasharray", "4 3")?;

    // Measured once, right after the (already-live) guide has been created. `guide` is unconditionally a <path>, which
    // always implements `SVGGeometryElement` (see total_length's own doc comment for exactly which element types do
    // not), so `None` here would mean the browser's own getTotalLength() failed on an element that must support it —
    // which would represent a broken demo invariant rather than a normal runtime condition; hence an explicit
    // construction error rather than an `unwrap_or(0.0)` that would otherwise leave the slider silently stuck at a max
    // of zero.
    let offset_max = guide
        .total_length()
        .ok_or_else(|| Error::Dom("demo-tp-offset-arc has no measurable path length".into()))?;
    // Floored, not rounded: see this section's own doc comment above for why.
    let max_rounded = offset_max.floor() as i64;

    let offset_text = svg.text(Point::origin(), "")?;
    let offset_path = offset_text.text_path("#demo-tp-offset-arc", "Offset 0")?;

    offset_path.set_fill(WHITE)?;
    offset_path.set_font_size(16.0)?;

    // The slider sits above the curve. Tall enough now for two extra rows below the track itself: hand-drawn tick marks
    // and a plain flex row giving the 0/max endpoints an explicit numeric label. Ticks are drawn by hand rather than
    // via a <datalist> linked through the slider's `list` attribute, which, at first, appears to be the more obviously
    // "native" choice.  However, the downside is that a <datalist>'s tick marks are only rendered by Chrome/Edge.
    // Firefox and Safari apply its snap-to-value behaviour but do not render any visible marks at all, so when using
    // those browsers, the ticks marks simply do not exist.
    //
    // As a result, we instead use hand-drawn marks (plain absolutely-positioned <span>s) at the correct percentage
    // along the track's width — nothing here depends on any browser-specific rendering, so it is expected to render
    // identically everywhere. That expectation is only actually verified for Firefox and Chrome, though: this
    // repository's CI (see `browser-tests` in .github/workflows/ci.yml) tests both, but WebKit/Safari needs a macOS
    // runner and lacks reliable headless support, so it is not exercised here and remains an unverified assumption,
    // not a proven guarantee.
    let slider_fo = svg.foreign_object(Point::new(430.0, 6.0), Size::new(340.0, 50.0))?;
    let slider_document = slider_fo
        .as_element()
        .owner_document()
        .ok_or_else(|| Error::Dom("cannot identify owner document for SVG foreignObject".into()))?;

    let slider_container = slider_document.create_element_ns(Some(NS_XHTML), "div").map_err(dom_err)?;
    slider_container
        .set_attribute("style", "display:flex; flex-direction:column; gap:2px;")
        .map_err(dom_err)?;

    let slider = slider_document
        .create_element_ns(Some(NS_XHTML), "input")
        .map_err(dom_err)?
        .dyn_into::<web_sys::HtmlInputElement>()
        .map_err(|_| Error::Dom("createElement(\"input\") did not return an HtmlInputElement".into()))?;
    slider.set_type("range");
    slider.set_min("0");
    slider.set_max(&max_rounded.to_string());
    slider.set_step("1");
    slider.set_value("0");
    slider.set_attribute("style", "width:100%;").map_err(dom_err)?;

    // No visible <label> — the nearby "Offset <n>" SVG text already carries this information for sighted users, but SVG
    // text content is not programmatically associated with an HTML control the way <label> and aria-labelledby are, so
    // an explicit aria-label is needed to give this slider an accessible name.
    slider.set_attribute("aria-label", "textPath startOffset").map_err(dom_err)?;
    // aria-valuetext starts matching the slider's own initial value/text content below, and is kept in sync by every
    // `input` event (see that handler below for why this is the right way to expose the current offset to assistive
    // technology, rather than create a separate self-announcing live region (`aria-live`) on top of a control that
    // already has its own accessible value semantics).
    slider.set_attribute("aria-valuetext", "Offset 0").map_err(dom_err)?;

    // Tick marks every 25 units plus one at the end: a row exactly as wide as the slider.
    // It uses `position:relative` and one `position:absolute` mark per tick at `left: {value / max * 100}%`
    // to draw hand-drawn tick marks that look the same on browsers that don't render tick marks at all.
    const TICK_STEP: i64 = 25;
    let ticks_row = slider_document.create_element_ns(Some(NS_XHTML), "div").map_err(dom_err)?;

    ticks_row
        .set_attribute("style", "position:relative; height:6px;")
        .map_err(dom_err)?;

    let add_tick = |value: i64| -> Result<(), Error> {
        let percent = if max_rounded == 0 { 0.0 } else { value as f64 / max_rounded as f64 * 100.0 };
        let mark = slider_document.create_element_ns(Some(NS_XHTML), "span").map_err(dom_err)?;
        mark.set_attribute(
            "style",
            &format!(
                "position:absolute; left:{percent:.2}%; top:0; width:1px; height:6px; \
                 background:#8b949e; transform:translateX(-50%);"
            ),
        )
        .map_err(dom_err)?;
        ticks_row.append_child(&mark).map_err(dom_err)?;
        Ok(())
    };

    let mut tick = 0;
    while tick < max_rounded {
        add_tick(tick)?;
        tick += TICK_STEP;
    }
    add_tick(max_rounded)?;

    let endpoint_labels = slider_document.create_element_ns(Some(NS_XHTML), "div").map_err(dom_err)?;
    endpoint_labels
        .set_attribute(
            "style",
            "display:flex; justify-content:space-between; font:11px sans-serif; color:#8b949e;",
        )
        .map_err(dom_err)?;

    let label_zero = slider_document.create_element_ns(Some(NS_XHTML), "span").map_err(dom_err)?;
    label_zero.set_text_content(Some("0"));

    let label_max = slider_document.create_element_ns(Some(NS_XHTML), "span").map_err(dom_err)?;
    label_max.set_text_content(Some(&max_rounded.to_string()));
    endpoint_labels.append_child(&label_zero).map_err(dom_err)?;
    endpoint_labels.append_child(&label_max).map_err(dom_err)?;

    // The closure reads the slider's value directly (a clone of the same `HtmlInputElement`, not the event's target).
    // While the slider is being dragged, the `input` event that fires continuously, rather than `change` that fires
    // only once the user lets go.
    let slider_value = slider.clone();
    let target = offset_path.clone();
    let mut label_buf = String::new();
    let on_input: DemoClosure = Closure::new(move |_: web_sys::Event| {
        let offset: f64 = slider_value.value().parse().unwrap_or(0.0);
        let _ = target.set_start_offset(offset);
        // When in the home position (offset 0), the text color is white; anywhere else along the path, is orange.
        let _ = target.set_fill(if offset == 0.0 { WHITE } else { CORAL });
        let _ = target.set_text_fmt(&mut label_buf, format_args!("Offset {offset:.0}"));
        // Keeps the accessible value in step with the same text sighted users see on the curve, rather than
        // leaving assistive technology to announce just the bare number a native range input reports by default.
        let _ = slider_value.set_attribute("aria-valuetext", &label_buf);
    });
    slider
        .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
        .map_err(dom_err)?;
    keep_demo_closure(on_input);

    slider_container.append_child(&slider).map_err(dom_err)?;
    slider_container.append_child(&ticks_row).map_err(dom_err)?;
    slider_container.append_child(&endpoint_labels).map_err(dom_err)?;
    slider_fo.as_element().append_child(&slider_container).map_err(dom_err)?;
    keep_demo_node(slider_fo);

    caption(&svg, 600.0, "startOffset — drag the slider")?;

    Ok(())
}
