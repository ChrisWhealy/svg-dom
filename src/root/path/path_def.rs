use super::elliptical_arc::EllipticalArc;
use crate::root::utils::{MAX_DPS, Point};
use std::fmt::Write;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// One SVG path-data command using coordinates absolute in the current user-coordinate system (the uppercase SVG
/// path commands).
///
/// See [`PathDefRelative`] for the coordinate-relative counterpart, and [`PathDef`] for the combined type used to
/// build a `d` attribute from a mixed sequence of both.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathDefAbsolute {
    /// `M x y` — starts a new subpath at the given point without drawing.
    MoveTo(Point),
    /// `L x y` — draws a straight line to the given point.
    LineTo(Point),
    /// `H x` — draws a horizontal line to the given `x`, keeping the current `y`.
    HorizontalLineTo(f64),
    /// `V y` — draws a vertical line to the given `y`, keeping the current `x`.
    VerticalLineTo(f64),
    /// `C x1 y1 x2 y2 x y` — cubic Bézier curve through two control points to an end point.
    CubicBezierTo(Point, Point, Point),
    /// `S x2 y2 x y` — cubic Bézier curve that mirrors the previous curve's final control point.
    SmoothCubicBezierTo(Point, Point),
    /// `Q x1 y1 x y` — quadratic Bézier curve through one control point to an end point.
    QuadraticBezierTo(Point, Point),
    /// `T x y` — quadratic Bézier curve that mirrors the previous curve's control point.
    SmoothQuadraticBezierTo(Point),
    /// `A rx ry x-axis-rotation large-arc-flag sweep-flag x y` — elliptical arc.
    EllipticalArcTo(EllipticalArc),
    /// `Z` — closes the current subpath by drawing a straight line back to its start.
    ClosePath,
}

impl PathDefAbsolute {
    fn write(self, out: &mut String, dps: Option<usize>) {
        match (self, dps) {
            (Self::MoveTo(p), Some(n)) => {
                let n = n.min(MAX_DPS);
                let _ = write!(out, "M{:.n$} {:.n$}", p.x, p.y);
            },
            (Self::MoveTo(p), None) => {
                let _ = write!(out, "M{} {}", p.x, p.y);
            },
            (Self::LineTo(p), Some(n)) => {
                let n = n.min(MAX_DPS);
                let _ = write!(out, "L{:.n$} {:.n$}", p.x, p.y);
            },
            (Self::LineTo(p), None) => {
                let _ = write!(out, "L{} {}", p.x, p.y);
            },
            (Self::HorizontalLineTo(x), Some(n)) => {
                let _ = write!(out, "H{:.*}", n.min(MAX_DPS), x);
            },
            (Self::HorizontalLineTo(x), None) => {
                let _ = write!(out, "H{x}");
            },
            (Self::VerticalLineTo(y), Some(n)) => {
                let _ = write!(out, "V{:.*}", n.min(MAX_DPS), y);
            },
            (Self::VerticalLineTo(y), None) => {
                let _ = write!(out, "V{y}");
            },
            (Self::CubicBezierTo(c1, c2, to), Some(n)) => {
                let n = n.min(MAX_DPS);
                let _ = write!(
                    out,
                    "C{:.n$} {:.n$} {:.n$} {:.n$} {:.n$} {:.n$}",
                    c1.x, c1.y, c2.x, c2.y, to.x, to.y
                );
            },
            (Self::CubicBezierTo(c1, c2, to), None) => {
                let _ = write!(out, "C{} {} {} {} {} {}", c1.x, c1.y, c2.x, c2.y, to.x, to.y);
            },
            (Self::SmoothCubicBezierTo(c2, to), Some(n)) => {
                let n = n.min(MAX_DPS);
                let _ = write!(out, "S{:.n$} {:.n$} {:.n$} {:.n$}", c2.x, c2.y, to.x, to.y);
            },
            (Self::SmoothCubicBezierTo(c2, to), None) => {
                let _ = write!(out, "S{} {} {} {}", c2.x, c2.y, to.x, to.y);
            },
            (Self::QuadraticBezierTo(c, to), Some(n)) => {
                let n = n.min(MAX_DPS);
                let _ = write!(out, "Q{:.n$} {:.n$} {:.n$} {:.n$}", c.x, c.y, to.x, to.y);
            },
            (Self::QuadraticBezierTo(c, to), None) => {
                let _ = write!(out, "Q{} {} {} {}", c.x, c.y, to.x, to.y);
            },
            (Self::SmoothQuadraticBezierTo(to), Some(n)) => {
                let _ = write!(out, "T{:.*} {:.*}", n.min(MAX_DPS), to.x, n.min(MAX_DPS), to.y);
            },
            (Self::SmoothQuadraticBezierTo(to), None) => {
                let _ = write!(out, "T{} {}", to.x, to.y);
            },
            (Self::EllipticalArcTo(arc), dps) => arc.write(out, 'A', dps),
            (Self::ClosePath, _) => out.push('Z'),
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// One SVG path-data command using coordinates relative to the current point (the lowercase SVG path commands).
///
/// See [`PathDefAbsolute`] for the absolute-coordinate counterpart, and [`PathDef`] for the combined type used to
/// build a `d` attribute from a mixed sequence of both.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathDefRelative {
    /// `m dx dy` — starts a new subpath at the given offset from the current point, without drawing.
    MoveTo(Point),
    /// `l dx dy` — draws a straight line to the given offset from the current point.
    LineTo(Point),
    /// `h dx` — draws a horizontal line `dx` units from the current point.
    HorizontalLineTo(f64),
    /// `v dy` — draws a vertical line `dy` units from the current point.
    VerticalLineTo(f64),
    /// `c dx1 dy1 dx2 dy2 dx dy` — cubic Bézier curve through two control points to an end point, all relative to
    /// the current point.
    CubicBezierTo(Point, Point, Point),
    /// `s dx2 dy2 dx dy` — cubic Bézier curve that mirrors the previous curve's final control point.
    SmoothCubicBezierTo(Point, Point),
    /// `q dx1 dy1 dx dy` — quadratic Bézier curve through one control point to an end point.
    QuadraticBezierTo(Point, Point),
    /// `t dx dy` — quadratic Bézier curve that mirrors the previous curve's control point.
    SmoothQuadraticBezierTo(Point),
    /// `a rx ry x-axis-rotation large-arc-flag sweep-flag dx dy` — elliptical arc.
    EllipticalArcTo(EllipticalArc),
    /// `z` — closes the current subpath by drawing a straight line back to its start.
    ClosePath,
}

impl PathDefRelative {
    fn write(self, out: &mut String, dps: Option<usize>) {
        match (self, dps) {
            (Self::MoveTo(p), Some(n)) => {
                let n = n.min(MAX_DPS);
                let _ = write!(out, "m{:.n$} {:.n$}", p.x, p.y);
            },
            (Self::MoveTo(p), None) => {
                let _ = write!(out, "m{} {}", p.x, p.y);
            },
            (Self::LineTo(p), Some(n)) => {
                let n = n.min(MAX_DPS);
                let _ = write!(out, "l{:.n$} {:.n$}", p.x, p.y);
            },
            (Self::LineTo(p), None) => {
                let _ = write!(out, "l{} {}", p.x, p.y);
            },
            (Self::HorizontalLineTo(x), Some(n)) => {
                let _ = write!(out, "h{:.*}", n.min(MAX_DPS), x);
            },
            (Self::HorizontalLineTo(x), None) => {
                let _ = write!(out, "h{x}");
            },
            (Self::VerticalLineTo(y), Some(n)) => {
                let _ = write!(out, "v{:.*}", n.min(MAX_DPS), y);
            },
            (Self::VerticalLineTo(y), None) => {
                let _ = write!(out, "v{y}");
            },
            (Self::CubicBezierTo(c1, c2, to), Some(n)) => {
                let n = n.min(MAX_DPS);
                let _ = write!(
                    out,
                    "c{:.n$} {:.n$} {:.n$} {:.n$} {:.n$} {:.n$}",
                    c1.x, c1.y, c2.x, c2.y, to.x, to.y
                );
            },
            (Self::CubicBezierTo(c1, c2, to), None) => {
                let _ = write!(out, "c{} {} {} {} {} {}", c1.x, c1.y, c2.x, c2.y, to.x, to.y);
            },
            (Self::SmoothCubicBezierTo(c2, to), Some(n)) => {
                let n = n.min(MAX_DPS);
                let _ = write!(out, "s{:.n$} {:.n$} {:.n$} {:.n$}", c2.x, c2.y, to.x, to.y);
            },
            (Self::SmoothCubicBezierTo(c2, to), None) => {
                let _ = write!(out, "s{} {} {} {}", c2.x, c2.y, to.x, to.y);
            },
            (Self::QuadraticBezierTo(c, to), Some(n)) => {
                let n = n.min(MAX_DPS);
                let _ = write!(out, "q{:.n$} {:.n$} {:.n$} {:.n$}", c.x, c.y, to.x, to.y);
            },
            (Self::QuadraticBezierTo(c, to), None) => {
                let _ = write!(out, "q{} {} {} {}", c.x, c.y, to.x, to.y);
            },
            (Self::SmoothQuadraticBezierTo(to), Some(n)) => {
                let _ = write!(out, "t{:.*} {:.*}", n.min(MAX_DPS), to.x, n.min(MAX_DPS), to.y);
            },
            (Self::SmoothQuadraticBezierTo(to), None) => {
                let _ = write!(out, "t{} {}", to.x, to.y);
            },
            (Self::EllipticalArcTo(arc), dps) => arc.write(out, 'a', dps),
            (Self::ClosePath, _) => out.push('z'),
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A single SVG path-data command, in either absolute or relative form.
///
/// A `<path>`'s `d` attribute is built from an ordered sequence of these — see [`write_d`] / [`build_d`], or the
/// `path_from_defs` factory method available everywhere [`SvgRoot::path`](crate::SvgRoot::path) is.  Because each
/// segment is a typed, well-formed command rather than free text, the resulting `d` string can never contain a
/// mistyped command letter, a missing argument, or any other malformed path data — mistakes a hand-written `d`
/// string can make silently, since an SVG path parser accepts a partially-broken string and simply stops rendering
/// at the first bad token rather than rejecting it outright.
///
/// Absolute and relative commands can be freely mixed in the same sequence, exactly as real SVG path data allows —
/// for example an initial [`PathDefAbsolute::MoveTo`] followed by a run of [`PathDefRelative`] draw commands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathDef {
    /// An absolute-coordinate command.
    Abs(PathDefAbsolute),
    /// A relative-coordinate command.
    Rel(PathDefRelative),
}

impl PathDef {
    fn write(self, out: &mut String, dps: Option<usize>) {
        match self {
            PathDef::Abs(cmd) => cmd.write(out, dps),
            PathDef::Rel(cmd) => cmd.write(out, dps),
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Writes SVG path-data into a caller-owned buffer from a sequence of [`PathDef`] commands, replacing any previous
/// contents.
///
/// This is the buffer-reusing counterpart to [`build_d`]; use it on a hot path — an animation that rebuilds a curve
/// every frame, say — to avoid allocating a fresh `String` on every call.
///
/// No whitespace is written between commands for the same reason that a command letter cannot appear inside a number.
/// Therefore, a command letter unambiguously terminates the previous command's last argument.
///
/// Omitting the whitespace both after a command letter and before the next argument is a standard, lossless path-size
/// optimisation.  For example, `"M10 10L100 50Z"` is semantically identical to `"M 10 10 L 100 50 Z"` in every
/// conforming SVG implementation, so the emitted `d` string can be shorter without sacrificing precision or validity.
///
/// Every numeric argument uses the default shortest round-trip representation; see [`write_d_fixed`] for a version
/// that trims each coordinate to a fixed number of decimal places instead.
pub fn write_d(out: &mut String, defs: &[PathDef]) {
    out.clear();
    for def in defs {
        def.write(out, None);
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Like [`write_d`], but writes every coordinate, length, and rotation angle with `dps` fixed decimal places
/// (clamped to `MAX_DPS` = 20).  The basic principle at work here is that shorter path strings mean less data needs to
/// be sent across the WASM/JS boundary and consequently requires less DOM attribute storage.
///
/// Mirrors `write_points`'s fixed-precision mode (the shared internal helper behind
/// [`SvgAttrs::points_fixed`](crate::SvgAttrs::points_fixed)): use this for path data whose coordinates come from a
/// calculation (an animation, a procedurally sampled curve) rather than a literal value, so the emitted `d` string
/// does not carry more digits of precision than the caller actually needs.
///
/// The two [`EllipticalArc`] flags (`large-arc-flag`, `sweep-flag`) are never affected by `dps`: the SVG grammar
/// requires these Boolean flags to be represented by the digits `"0"` or `"1"`.  Consequently, they are always written
/// as plain integers regardless of the requested precision.
pub fn write_d_fixed(out: &mut String, defs: &[PathDef], dps: usize) {
    out.clear();
    for def in defs {
        def.write(out, Some(dps));
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Builds a fresh SVG path-data (`d` attribute) string from a sequence of [`PathDef`] commands.
///
/// See [`write_d`] for a version that reuses a caller-owned buffer instead of allocating a new `String` on every call,
/// and for why the output omits whitespace that the SVG path grammar does not require.
///
/// # Example
///
/// ```rust
/// use svg_dom::{PathDef, PathDefAbsolute, build_d, root::utils::Point};
///
/// let d = build_d(&[
///     PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(10.0, 10.0))),
///     PathDef::Abs(PathDefAbsolute::LineTo(Point::new(100.0, 50.0))),
///     PathDef::Abs(PathDefAbsolute::LineTo(Point::new(10.0, 90.0))),
///     PathDef::Abs(PathDefAbsolute::ClosePath),
/// ]);
/// assert_eq!(d, "M10 10L100 50L10 90Z");
/// ```
pub fn build_d(defs: &[PathDef]) -> String {
    let mut out = String::new();
    write_d(&mut out, defs);
    out
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Like [`build_d`], but writes every coordinate, length, and rotation angle with `dps` fixed decimal places
/// (clamped to `MAX_DPS` = 20). See [`write_d_fixed`] for the full rationale.
///
/// # Example
///
/// ```rust
/// use svg_dom::{PathDef, PathDefAbsolute, build_d_fixed, root::utils::Point};
///
/// let d = build_d_fixed(&[
///     PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(1.0 / 3.0, 2.0 / 3.0))),
///     PathDef::Abs(PathDefAbsolute::LineTo(Point::new(10.0, 20.0))),
/// ], 2);
/// assert_eq!(d, "M0.33 0.67L10.00 20.00");
/// ```
pub fn build_d_fixed(defs: &[PathDef], dps: usize) -> String {
    let mut out = String::new();
    write_d_fixed(&mut out, defs, dps);
    out
}
