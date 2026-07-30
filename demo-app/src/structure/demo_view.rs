use crate::{H, PAD_Y, W, caption, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgNode, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// view — named viewBox/preserveAspectRatio, navigated to via a URL fragment on an external reference
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-view", Size::new(W, H))?;

    // <view> only affects an SVG resource referenced *externally* — a same-document <image>/HTML <img>, or the
    // page's own address bar — never a live DOM handle the caller already has (that case is just
    // set_view_box/set_viewport directly). So unlike every other panel in this gallery, the content worth looking
    // at here is a separate, hand-authored standalone SVG file (view-demo.svg, served alongside this page — see its
    // own comment) rather than anything built through svg-dom's own API. Clicking a button below sets this
    // <image>'s href to the same file plus a #view fragment via SvgNode::set_href; the browser does the rest with
    // no JavaScript of svg-dom's own involved.
    let pic = svg.image("view-demo.svg", Point::new(40.0, PAD_Y), Size::new(130.0, 130.0))?;

    let button = |y: f64, label: &str, fill: &str| -> Result<SvgNode, Error> {
        let rect = svg.rect(Point::new(220.0, y), Size::new(180.0, 30.0))?;
        rect.set_fill(fill)?;
        rect.set_attrs([("rx", "6"), ("style", "cursor:pointer")])?;
        let text = svg.text(Point::new(220.0 + 90.0, y + 20.0), label)?;
        text.set_fill(WHITE)?;
        text.set_attrs([("font-size", "13"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;
        Ok(rect)
    };

    let overview_btn = button(PAD_Y, "Overview", RESET_IDLE)?;
    let overview_pic = pic.clone();
    overview_btn.on_click(move |_| {
        let _ = overview_pic.set_href("view-demo.svg");
    })?;

    // view-demo.svg's "top-left" view is viewBox="0 0 100 100" — the fixture's steelblue quadrant (x 0-100, y
    // 0-100). The button's own fill matches that quadrant, not the coral one (which is actually top-*right*).
    let tl_btn = button(PAD_Y + 40.0, "Top-left quadrant", STEELBLUE)?;
    let tl_pic = pic.clone();
    tl_btn.on_click(move |_| {
        let _ = tl_pic.set_href("view-demo.svg#top-left");
    })?;

    let br_btn = button(PAD_Y + 80.0, "Bottom-right quadrant", MEDIUM_SEA_GREEN)?;
    let br_pic = pic.clone();
    br_btn.on_click(move |_| {
        let _ = br_pic.set_href("view-demo.svg#bottom-right");
    })?;

    keep_demo_node(overview_btn);
    keep_demo_node(tl_btn);
    keep_demo_node(br_btn);

    caption(
        &svg,
        W / 2.0,
        "click a button — the browser switches the embedded SVG's view; no JavaScript of svg-dom's own is involved",
    )?;
    Ok(())
}
