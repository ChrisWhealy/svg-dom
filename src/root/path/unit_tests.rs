use super::{elliptical_arc::*, path_def::*};
use crate::root::utils::Point;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_d — one command per SVG letter
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn build_d_writes_absolute_move_and_line() {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(10.0, 10.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(100.0, 50.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(10.0, 90.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
    ]);
    assert_eq!(d, "M10 10L100 50L10 90Z");
}

#[test]
fn build_d_writes_relative_move_and_line() {
    let d = build_d(&[
        PathDef::Rel(PathDefRelative::MoveTo(Point::new(10.0, 10.0))),
        PathDef::Rel(PathDefRelative::LineTo(Point::new(90.0, 40.0))),
        PathDef::Rel(PathDefRelative::ClosePath),
    ]);
    assert_eq!(d, "m10 10l90 40z");
}

#[test]
fn build_d_writes_horizontal_and_vertical_lines() {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::HorizontalLineTo(50.0)),
        PathDef::Abs(PathDefAbsolute::VerticalLineTo(25.0)),
        PathDef::Rel(PathDefRelative::HorizontalLineTo(-10.0)),
        PathDef::Rel(PathDefRelative::VerticalLineTo(-5.0)),
    ]);
    assert_eq!(d, "M0 0H50V25h-10v-5");
}

#[test]
fn build_d_writes_cubic_and_smooth_cubic_bezier() {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::CubicBezierTo(
            Point::new(10.0, 10.0),
            Point::new(20.0, 10.0),
            Point::new(30.0, 0.0),
        )),
        PathDef::Abs(PathDefAbsolute::SmoothCubicBezierTo(
            Point::new(40.0, 10.0),
            Point::new(50.0, 0.0),
        )),
    ]);
    assert_eq!(d, "M0 0C10 10 20 10 30 0S40 10 50 0");
}

#[test]
fn build_d_writes_quadratic_and_smooth_quadratic_bezier() {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::QuadraticBezierTo(
            Point::new(10.0, 10.0),
            Point::new(20.0, 0.0),
        )),
        PathDef::Abs(PathDefAbsolute::SmoothQuadraticBezierTo(Point::new(30.0, 0.0))),
    ]);
    assert_eq!(d, "M0 0Q10 10 20 0T30 0");
}

#[test]
fn build_d_writes_elliptical_arc_with_size_and_sweep_flags() {
    let d = build_d(&[
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(10.0, 65.0))),
        PathDef::Abs(PathDefAbsolute::EllipticalArcTo(EllipticalArc {
            radii: Point::new(60.0, 60.0),
            x_axis_rotation: 0.0,
            size: ArcSize::Large,
            sweep: ArcSweep::Clockwise,
            to: Point::new(130.0, 65.0),
        })),
    ]);
    assert_eq!(d, "M10 65A60 60 0 1 1 130 65");
}

#[test]
fn build_d_writes_small_counter_clockwise_arc_flags_as_zero() {
    let d = build_d(&[PathDef::Rel(PathDefRelative::EllipticalArcTo(EllipticalArc {
        radii: Point::new(5.0, 5.0),
        x_axis_rotation: 0.0,
        size: ArcSize::Small,
        sweep: ArcSweep::CounterClockwise,
        to: Point::new(10.0, 0.0),
    }))]);
    assert_eq!(d, "a5 5 0 0 0 10 0");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// write_d — buffer reuse
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn write_d_clears_previous_contents_before_writing() {
    let mut buf = String::from("stale contents that must not survive");
    write_d(&mut buf, &[PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0)))]);
    assert_eq!(buf, "M1 2");
}

#[test]
fn write_d_matches_build_d_for_the_same_input() {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(5.0, 5.0))),
    ];
    let mut buf = String::new();
    write_d(&mut buf, &defs);
    assert_eq!(buf, build_d(&defs));
}

#[test]
fn build_d_of_empty_slice_is_empty_string() {
    assert_eq!(build_d(&[]), "");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_d_fixed / write_d_fixed
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn build_d_fixed_rounds_coordinates_to_requested_precision() {
    let d = build_d_fixed(
        &[
            PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0 / 3.0, 2.0 / 3.0))),
            PathDef::Abs(PathDefAbsolute::LineTo(Point::new(10.0, 20.0))),
        ],
        2,
    );
    assert_eq!(d, "M0.33 0.67L10.00 20.00");
}

#[test]
fn build_d_fixed_at_zero_decimals_rounds_to_integers() {
    let d = build_d_fixed(&[PathDef::Abs(PathDefAbsolute::LineTo(Point::new(1.6, 2.4)))], 0);
    assert_eq!(d, "L2 2");
}

#[test]
fn build_d_fixed_rounds_horizontal_and_vertical_line_arguments() {
    let d = build_d_fixed(
        &[
            PathDef::Abs(PathDefAbsolute::HorizontalLineTo(1.0 / 3.0)),
            PathDef::Rel(PathDefRelative::VerticalLineTo(-1.0 / 3.0)),
        ],
        1,
    );
    assert_eq!(d, "H0.3v-0.3");
}

#[test]
fn build_d_fixed_rounds_smooth_and_quadratic_bezier_arguments() {
    let d = build_d_fixed(
        &[
            PathDef::Abs(PathDefAbsolute::SmoothCubicBezierTo(
                Point::new(1.0 / 3.0, 0.0),
                Point::new(0.0, 0.0),
            )),
            PathDef::Abs(PathDefAbsolute::SmoothQuadraticBezierTo(Point::new(1.0 / 3.0, 0.0))),
        ],
        2,
    );
    assert_eq!(d, "S0.33 0.00 0.00 0.00T0.33 0.00");
}

/// The two elliptical-arc flags must never be affected by `dps`: the SVG `flag` grammar production is exactly one
/// `"0"` or `"1"` digit, not a decimal number, so rounding them to `"0.00"`/`"1.00"` would be invalid path syntax.
#[test]
fn build_d_fixed_never_rounds_elliptical_arc_flags() {
    let d = build_d_fixed(
        &[PathDef::Abs(PathDefAbsolute::EllipticalArcTo(EllipticalArc {
            radii: Point::new(1.0 / 3.0, 1.0 / 3.0),
            x_axis_rotation: 1.0 / 3.0,
            size: ArcSize::Large,
            sweep: ArcSweep::Clockwise,
            to: Point::new(1.0 / 3.0, 1.0 / 3.0),
        }))],
        2,
    );
    assert_eq!(d, "A0.33 0.33 0.33 1 1 0.33 0.33");
}

#[test]
fn write_d_fixed_clamps_dps_to_max() {
    let mut clamped = String::new();
    let mut at_max = String::new();
    let defs = [PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.5, 2.5)))];
    write_d_fixed(&mut clamped, &defs, usize::MAX);
    write_d_fixed(&mut at_max, &defs, 20);
    assert_eq!(
        clamped, at_max,
        "usize::MAX dps must produce the same output as the MAX_DPS clamp"
    );
}

#[test]
fn write_d_fixed_matches_build_d_fixed_for_the_same_input() {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(5.0 / 3.0, 5.0 / 3.0))),
    ];
    let mut buf = String::new();
    write_d_fixed(&mut buf, &defs, 3);
    assert_eq!(buf, build_d_fixed(&defs, 3));
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Capacity pre-reservation — build_d / build_d_fixed only
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn build_d_reserves_capacity_proportional_to_command_count() {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(3.0, 4.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
    ];
    let expected_min = defs.len() * 24;
    let d = build_d(&defs);
    assert!(
        d.capacity() >= expected_min,
        "expected capacity >= {expected_min}, got {}",
        d.capacity()
    );
}

#[test]
fn build_d_fixed_reserves_capacity_proportional_to_command_count() {
    let defs = [
        PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0))),
        PathDef::Abs(PathDefAbsolute::LineTo(Point::new(3.0, 4.0))),
        PathDef::Abs(PathDefAbsolute::ClosePath),
    ];
    let expected_min = defs.len() * 24;
    let d = build_d_fixed(&defs, 2);
    assert!(
        d.capacity() >= expected_min,
        "expected capacity >= {expected_min}, got {}",
        d.capacity()
    );
}

/// `write_d` must not reserve on the caller's behalf: it writes into a buffer the caller is expected to reuse
/// (and therefore already size correctly, via `SvgAttrs::with_capacity` if desired), so a fresh, empty buffer keeps
/// whatever capacity `String`'s own incremental growth produces rather than a `build_d`-style upfront reservation.
#[test]
fn write_d_does_not_preemptively_reserve_like_build_d_does() {
    let defs = [PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0)))];
    let mut buf = String::new();
    write_d(&mut buf, &defs);
    assert!(
        buf.capacity() < defs.len() * 24,
        "write_d should not pre-reserve a build_d-sized allocation for a single short command"
    );
}
