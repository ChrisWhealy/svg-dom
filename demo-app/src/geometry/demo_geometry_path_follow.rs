use crate::{H, PAD_Y, W, caption, colours::*, keep_demo_anim};
use svg_dom::{
    AnimationLoop, Error, SvgRoot,
    root::utils::{Point, Size},
};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Geometry — total_length / point_at_length (a marker chasing a lap around an ellipse track)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
pub(crate) fn demo() -> Result<(), Error> {
    // `total_length()` is measured once at setup.  This means the track's geometry never changes, so there is no
    // reason to re-measure it every frame.
    //
    // By contrast, `point_at_length()` genuinely belongs on the animation's hot path, so the runner's position needs
    // to be recomputed every frame from the current lap fraction. That is exactly the per-frame browser measurement
    // the method's own doc comment cautions about — a legitimate demonstration here (one using a simple ellipse, not
    // a whole scene's worth of paths), but the acceptable cost has not been independently profiled.
    //
    // ⚠️ A caller implementing this for real should profile it against their own path complexity and target browser
    // before assuming the functionality scales adequately.
    const CX: f64 = W / 2.0;
    const CY: f64 = (PAD_Y / 2.0) + BAND_HALF;
    const BAND_HALF: f64 = 65.0;
    const RX: f64 = 200.0;
    const RY: f64 = 48.0;
    const LAP_MS: f64 = 4000.0;

    let svg = SvgRoot::create_in("demo-geometry-path-follow", Size::new(W, H))?;

    let track = svg.ellipse(Point::new(CX, CY), Size::new(RX, RY))?;
    track.set_fill(NONE)?;
    track.set_stroke(GUIDE)?;
    track.set_stroke_width(2.0)?;
    track.set_attr("stroke-dasharray", "5 4")?;

    // Measured once — the track's shape is static for the life of this demo.
    let total = track.total_length().unwrap_or(0.0);

    let runner = svg.circle(Point::new(CX + RX, CY), 8.0)?;
    runner.set_fill(ACCENT_BLUE)?;

    let readout = svg.text(Point::new(20.0, 20.0), &format!("total length: {total:.0}"))?;
    readout.set_fill(TEXT)?;
    readout.set_attr("font-size", "14")?;

    let lap_readout = svg.text(Point::new(20.0, 40.0), "distance: 0 / 0")?;
    lap_readout.set_fill(TEXT_MUTED)?;
    lap_readout.set_attr("font-size", "12")?;

    let anim = AnimationLoop::start_with_frame(move |ts, frame| {
        let t = (ts % LAP_MS) / LAP_MS;
        let distance = t * total;
        if let Ok(p) = track.point_at_length(distance) {
            let _ = frame.set_attr_fmt(&runner, "cx", format_args!("{:.1}", p.x));
            let _ = frame.set_attr_fmt(&runner, "cy", format_args!("{:.1}", p.y));
        }
        let _ = frame.set_text_fmt(&lap_readout, format_args!("distance: {distance:.0} / {total:.0}"));
    })?;

    caption(
        &svg,
        400.0,
        "total_length() measured once at setup · point_at_length() drives the runner every frame — profile per-frame browser measurement like this for your own path complexity and target browser",
    )?;
    keep_demo_anim(anim);
    Ok(())
}
