pub(crate) mod demo_text;
pub(crate) mod demo_text_path;
pub(crate) mod demo_tspan;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds an SVG path `d` string for `periods` cycles of a sine wave.
/// The wave is `width` user units wide, `amplitude` user units tall and starts at `(x0, y0)`.
///
/// The path is sampled as short straight-line segments, `STEP` user units apart.
/// This is dense enough to give the visual appearance of a smooth curve at demo scale.
/// Bézier curves would fit the wave more precisely, but this approach is far simpler.
/// Deriving cubic control points for a true sine curve is not simple.
/// A circular arc's Bézier approximation constant does not apply to a sine curve.
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
