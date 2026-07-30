use crate::{BAND, H, NS_XHTML, PAD_Y, W, caption, colours::*, dom_err, keep_demo_node};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// foreignObject — a rectangular region of the canvas laid out by the browser's own HTML engine, not SVG's
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-foreign-object", Size::new(W, H))?;

    // <foreignObject> paints nothing of its own — without a visible boundary, the demo would just show floating
    // text with no indication of the SVG-space rectangle it's laid out (and, by default, clipped) within.
    let boundary = svg.rect(Point::new(40.0, PAD_Y), Size::new(340.0, BAND))?;
    boundary.set_fill(NONE)?;
    boundary.set_stroke(GUIDE)?;
    boundary.set_attr("stroke-dasharray", "4 3")?;

    let fo = svg.foreign_object(Point::new(40.0, PAD_Y), Size::new(340.0, BAND))?;

    // svg-dom's own API deliberately stops here: no set_inner_html or set_content method exists on the returned SvgNode
    // — see SvgRoot::foreign_object's doc comment for why a string-based convenience method isn't offered (parsing
    // caller-supplied markup means taking on sanitisation/trust concerns that this crate has no business maintaining).
    //
    // Everything below builds the small <div>, <strong> and <em> tree through raw web-sys calls node by node, rather
    // than via set_inner_html, even though this particular string is a compile-time constant and would be perfectly
    // safe passed to set_inner_html too. Building it explicitly is what the documented escape hatch actually looks
    // like, which is the whole point of this demo.
    let document = fo
        .as_element()
        .owner_document()
        .ok_or_else(|| Error::Dom("no owner document".into()))?;
    let content = document.create_element_ns(Some(NS_XHTML), "div").map_err(dom_err)?;
    content
        .set_attribute(
            "style",
            "font: 13px/1.4 sans-serif; color: #eee; padding: 8px; box-sizing: border-box;",
        )
        .map_err(dom_err)?;

    let make_inline = |tag: &str, text: &str| -> Result<web_sys::Element, Error> {
        let el = document.create_element_ns(Some(NS_XHTML), tag).map_err(dom_err)?;
        el.set_text_content(Some(text));
        Ok(el)
    };

    content
        .append_child(&make_inline("strong", "Real HTML")?.into())
        .map_err(dom_err)?;
    content
        .append_child(&document.create_text_node(", laid out by the browser's own engine: this paragraph "))
        .map_err(dom_err)?;
    content.append_child(&make_inline("em", "wraps")?.into()).map_err(dom_err)?;
    content
        .append_child(&document.create_text_node(
            " to the box width exactly the way it would on an ordinary web page — something SVG's own \
             <text> element cannot do by itself.",
        ))
        .map_err(dom_err)?;

    fo.as_element().append_child(&content).map_err(dom_err)?;
    keep_demo_node(fo);

    caption(
        &svg,
        W / 2.0,
        "<foreignObject> embeds real, browser-laid-out HTML inside the SVG canvas — text wrapping included",
    )?;
    Ok(())
}
