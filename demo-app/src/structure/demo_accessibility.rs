use crate::{H, PAD_Y, W, caption, colours::*, keep_demo_node};
use svg_dom::{
    Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// accessibility — set_title / set_desc, native tooltip, and read-back via title() / desc()
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    let svg = SvgRoot::create_in("demo-accessibility", Size::new(W, H))?;

    // Three labelled icon circles — non-interactive graphics, not buttons. Each carries its own <title> child (also
    // the native hover tooltip) and <desc> child (visible only to assistive technology), but deliberately no
    // role/tabindex/click or keyboard handling: <title>/<desc> make a graphic describable, they do not make it a
    // control, so this demo does not dress it up as one (no pointer cursor, no button semantics).
    //
    // Neither is used here as a stand-in for real accessible-name/description computation: aria-label/aria-labelledby
    // and aria-describedby would take precedence over these if present, which this demo deliberately doesn't exercise.
    //
    // The readout below echoes both back via the title()/desc() getters when a pointer enters an icon, so the invisible
    // <desc> becomes visible for the purposes of this demo — this hover-to-reveal is the same passive reveal a mouse
    // user gets from any native title tooltip, not a stand-in for clicking a control.
    let icons: [(f64, &str, &str, &str, &str); 3] = [
        (
            150.0,
            STEELBLUE,
            "Save",
            "Save icon",
            "Represents the save function: writes the current document to disk.",
        ),
        (
            400.0,
            ACCENT_AMBER,
            "Share",
            "Share icon",
            "Represents the share function: opens the share sheet for this item.",
        ),
        (
            650.0,
            CRIMSON,
            "Delete",
            "Delete icon",
            "Represents the delete function: permanently removes the selected item. This cannot be undone.",
        ),
    ];

    let icon_y = PAD_Y + 34.0;
    let readout = svg.text(
        Point::new(W / 2.0, PAD_Y + 92.0),
        "hover an icon to read its title and desc back",
    )?;
    readout.set_fill(TEXT)?;
    readout.set_attrs([("font-size", "13"), ("text-anchor", "middle")])?;

    for (cx, fill, label, title, desc) in icons {
        let icon = svg.circle(Point::new(cx, icon_y), 22.0)?;
        icon.set_fill(fill)?;
        icon.set_title(title)?;
        icon.set_desc(desc)?;

        let icon_label = svg.text(Point::new(cx, icon_y + 40.0), label)?;
        icon_label.set_fill(TEXT_MUTED)?;
        icon_label.set_attrs([("font-size", "12"), ("text-anchor", "middle"), ("style", "pointer-events:none")])?;

        let hover_readout = readout.clone();
        let hover_icon = icon.clone();
        icon.on_pointerenter(move |_| {
            let title = hover_icon.title().unwrap_or_default();
            let desc = hover_icon.desc().unwrap_or_default();
            hover_readout.set_text(&format!("title: \"{title}\"  ·  desc: \"{desc}\""));
        })?;

        keep_demo_node(icon);
    }

    caption(
        &svg,
        W / 2.0,
        "set_title() also drives the browser's native hover tooltip; set_desc() has no visible tooltip",
    )?;

    Ok(())
}
