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
    assert_eq!(d, "M 10 10 L 100 50 L 10 90 Z");
}

#[test]
fn build_d_writes_relative_move_and_line() {
    let d = build_d(&[
        PathDef::Rel(PathDefRelative::MoveTo(Point::new(10.0, 10.0))),
        PathDef::Rel(PathDefRelative::LineTo(Point::new(90.0, 40.0))),
        PathDef::Rel(PathDefRelative::ClosePath),
    ]);
    assert_eq!(d, "m 10 10 l 90 40 z");
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
    assert_eq!(d, "M 0 0 H 50 V 25 h -10 v -5");
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
    assert_eq!(d, "M 0 0 C 10 10 20 10 30 0 S 40 10 50 0");
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
    assert_eq!(d, "M 0 0 Q 10 10 20 0 T 30 0");
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
    assert_eq!(d, "M 10 65 A 60 60 0 1 1 130 65");
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
    assert_eq!(d, "a 5 5 0 0 0 10 0");
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// write_d — buffer reuse
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[test]
fn write_d_clears_previous_contents_before_writing() {
    let mut buf = String::from("stale contents that must not survive");
    write_d(&mut buf, &[PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0, 2.0)))]);
    assert_eq!(buf, "M 1 2");
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
