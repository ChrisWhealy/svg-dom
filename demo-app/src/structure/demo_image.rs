use crate::{
    BAND, H, PAD_Y, W, caption,
    colours::*,
    dom_err,
    foreign_html::{foreign_object_document, radio_group},
    keep_demo_node,
};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// image — embed a raster or SVG image
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-image", Size::new(W, H))?;

    // A 60×40 four-quadrant colour grid, embedded as a base64 PNG data URI.
    // The source has an aspect ratio of 3:2, and the display box below uses a 1:1 aspect ratio.
    // This mismatch is what makes each preserveAspectRatio value look different.
    //
    // The source must be a raster image, not an embedded SVG. Chromium does not clip a nested SVG `<image>` correctly
    // when preserveAspectRatio is set to "slice". The overflow that "slice" should crop away stays visible in Chromium.
    // As a result, a nested-SVG "slice" looks identical to "meet" in Chromium, but switching to a raster source avoids
    // this bug.
    //
    // A small white and black dot sits near the source's left edge, at x=5.  The purpose of this dot is to make each
    // "slice" visually distinguishable from "none".
    //
    // Both values scale the source to a different size and the grid's colour boundaries sit exactly on the source's
    // centre line.
    //
    // A symmetric centre-crop therefore maps that centre back onto itself. Without the dot, "none" and "slice" would
    // place the quadrant colours identically. The off-centre dot breaks this symmetry.
    //
    // "slice" crops the source to its central 40 units, from x=10 to x=50 causing the dot to fall outside that window,
    // so "slice" hides it. "meet" and "none" show the whole source, so both keep the dot visible.
    const SRC: &str = "data:image/png;base64,\
        iVBORw0KGgoAAAANSUhEUgAAADwAAAAoCAIAAAAt2Q6oAAAAfklEQVR42u3UwQmAMBBE0S3Ms2db\
        sQLBlrQXy1kFQQIhgiYTovzhn8O7bKyfV1E+DaIMNGjQoEGDBg0a9H/RFq119Kn0YEXcQnQsLuXW\
        oj0x0BXRvr2vW0ZRn0XfHGKj6OP11JeXI5ajL3e4THENtCLQoEGDBg0aNGjQoJ+0A+7MRi7Hd5r9\
        AAAAAElFTkSuQmCC";

    // Alternative image: slate-blue with a centred white circle.
    // The set_href panel below swaps to this image.
    // This is a PNG for the same nested-SVG reason as SRC above.
    const ALT: &str = "data:image/png;base64,\
        iVBORw0KGgoAAAANSUhEUgAAADwAAAAoCAIAAAAt2Q6oAAAAfElEQVR42u3YwQ2AMAxDUc/JCOza\
        NZDYgCML0EsT0qb5kgd4t8TWebR0EWjQoLdD39fzmUXRPe4fdMVwfekKFru4FS+2uzVFbHRrltji\
        LoP2Eg+7a6B9xWNu0KBBgwbNRVwCzZdHCdizbmUttlknhKxjTeJZjNUUNOhi6BfLEZpQ5CIbXgAA\
        AABJRU5ErkJggg==";

    // 100×100 px square display boxes; the 3:2 source makes preserveAspectRatio effects clear.
    let img_w = 100.0_f64;
    let img_h = 100.0_f64;
    let y0 = PAD_Y + (BAND - img_h) / 2.0;

    // Thin guide outline showing an image's bounding box.
    let slot = |x: f64| -> Result<(), Error> {
        let r = svg.rect(Point::new(x, y0), Size::new(img_w, img_h))?;
        r.set_fill(NONE)?;
        r.set_stroke(GUIDE)?;
        r.set_stroke_width(1.0)?;
        Ok(())
    };

    // Interactive preserveAspectRatio, chosen by radio button.
    // This is the same pattern demo_text uses for its text-anchor radio group.
    // radio_group() builds three real HTML radio buttons inside a <foreignObject>.
    // Selecting an option calls set_attr("preserveAspectRatio", value) on the one live image.
    let interactive_x = 80.0;
    slot(interactive_x)?;
    let interactive_img = svg.image(SRC, Point::new(interactive_x, y0), Size::new(img_w, img_h))?;
    interactive_img.set_attr("preserveAspectRatio", "xMidYMid meet")?;
    caption(&svg, interactive_x + img_w / 2.0, "preserveAspectRatio")?;

    const OPTIONS: [(&str, &str); 3] = [("xMidYMid meet", "meet"), ("none", "none"), ("xMidYMid slice", "slice")];

    let radio_fo = svg.foreign_object(
        Point::new(interactive_x + img_w + 30.0, PAD_Y + 5.0),
        Size::new(220.0, BAND - 10.0),
    )?;
    let radio_document = foreign_object_document(&radio_fo)?;

    let target = interactive_img.clone();
    let group = radio_group(
        &radio_document,
        "preserveAspectRatio",
        "demo-image-par",
        &OPTIONS,
        "xMidYMid meet",
        move |value| {
            let _ = target.set_attr("preserveAspectRatio", value);
        },
    )?;
    radio_fo.as_element().append_child(&group).map_err(dom_err)?;
    keep_demo_node(radio_fo);

    // set_href swap.
    // The element is created with SRC, and its source is then swapped to ALT after creation.
    let href_x = 590.0;
    slot(href_x)?;
    let swap_img = svg.image(SRC, Point::new(href_x, y0), Size::new(img_w, img_h))?;
    swap_img.set_href(ALT)?;
    caption(&svg, href_x + img_w / 2.0, "set_href swap")?;

    Ok(())
}
