use crate::{H, PAD_Y, W, caption, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Events — CSS class manipulation (add_class / remove_class / toggle_class / has_class)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    // Each tile's `on_click` handler only calls `toggle_class("selected")` — it never touches `fill`/`stroke`
    // directly. The gold highlight comes entirely from the `.tile.selected` rule in style.css, so clicking
    // demonstrates that a `class` attribute change alone is enough to restyle the element. The "selected: n / 3"
    // readout is recomputed from `has_class` on every tile after each click, rather than tracked in separate Rust
    // state, so it can never drift from what is actually in the DOM.
    const TILE_W: f64 = 140.0;
    const TILE_H: f64 = 60.0;
    const GAP: f64 = 30.0;
    const START_X: f64 = 40.0;

    let svg = SvgRoot::create_in("demo-events-classlist", Size::new(W, H))?;

    let readout = svg.text(Point::new(START_X, 24.0 + PAD_Y), "selected: 0 / 3")?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "15")?;

    let mut tiles = Vec::with_capacity(3);
    for i in 0..3 {
        let x = START_X + i as f64 * (TILE_W + GAP);
        let tile = svg.rect(Point::new(x, 45.0 + PAD_Y), Size::new(TILE_W, TILE_H))?;
        tile.set_fill(SLATE_GRAY)?;
        tile.set_attrs([("rx", "8"), ("class", "tile"), ("style", "cursor:pointer")])?;

        let label = svg.text(
            Point::new(x + TILE_W / 2.0, 45.0 + PAD_Y + TILE_H / 2.0 + 5.0),
            &format!("tile {}", i + 1),
        )?;
        label.set_fill(WHITE)?;
        label.set_attrs([("font-size", "14"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

        tiles.push(tile);
    }

    for tile in &tiles {
        let click_target = tile.clone();
        let peers = tiles.clone();
        let readout = readout.clone();
        tile.on_click(move |_| {
            // `"selected"` is a fixed, non-empty, whitespace-free literal, so `toggle_class` will never raise either
            // of its error cases (empty token / token containing ASCII whitespace).  Consequently, ignoring the
            // `Result` is not by oversight; it is done in the knowledge that under these particular conditions,
            // calling this otherwise fallible function (`toggle_class`) will never raise an `Err`.
            let _ = click_target.toggle_class("selected");
            let n = peers.iter().filter(|t| t.has_class("selected")).count();
            readout.set_text(&format!("selected: {n} / {}", peers.len()));
        })?;
    }

    caption(
        &svg,
        400.0,
        "on_click → toggle_class(\"selected\") · has_class() recomputes the readout straight from the DOM",
    )?;
    for tile in tiles {
        keep_demo_node(tile);
    }
    Ok(())
}
