pub(crate) mod elliptical_arc;
pub(crate) mod path_def;

pub use elliptical_arc::EllipticalArc;
pub use path_def::{PathDef, PathDefAbsolute, PathDefRelative, build_d, build_d_fixed, write_d, write_d_fixed};

use crate::{SvgRoot, error::Error, node::SvgNode, root::factory::SvgFactory};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgRoot {
    /// Creates a `<path>` element from an SVG path-data string, appends it to the root, and returns its [`SvgNode`]
    /// handle.
    ///
    /// The `d` string uses standard SVG path commands where the arguments to the uppercase command supply absolute
    /// coordinates, and the arguments to the lowercase commands supply relative coordinates.
    ///
    /// | Command   | Arguments              | Description             |
    /// |:----------|:-----------------------|:------------------------|
    /// | `M` / `m` | `x y`                  | Move (no draw)          |
    /// | `L` / `l` | `x y`                  | Line                    |
    /// | `H` / `h` | `x`                    | Horizontal Line         |
    /// | `V` / `v` | `y`                    | Vertical Line           |
    /// | `C` / `c` | `x1 y1 x2 y2 x y`      | Cubic Bézier            |
    /// | `S` / `s` | `x2 y2 x y`            | Smooth Cubic Bézier     |
    /// | `Q` / `q` | `x1 y1 x y`            | Quadratic Bézier        |
    /// | `T` / `t` | `x y`                  | Smooth Quadratic Bézier |
    /// | `A` / `a` | `rx ry rot laf sf x y` | Elliptical arc          |
    /// | `Z` / `z` | —                      | Close path              |
    ///
    /// The path can be updated later without recreating the element using [`SvgNode::set_d`].
    ///
    /// Since `d` is free text, nothing stops it from being malformed — a typo'd command letter, a missing argument,
    /// or a mismatched flag are all accepted silently by the SVG parser, which just stops rendering at the bad
    /// token. When the path data is being constructed rather than pasted in from an external source, prefer
    /// [`path_from_defs`](Self::path_from_defs), which builds `d` from typed [`PathDef`] segments so malformed path
    /// data cannot be constructed in the first place.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::SvgRoot;
    /// let svg = SvgRoot::attach("diagram")?;
    /// let path = svg.path("M 10 10 L 100 50 L 10 90 Z")?;
    /// path.set_fill("none")?;
    /// path.set_stroke("black")?;
    ///
    /// // Mutate the existing DOM node — the element does not need to be destroyed then recreated.
    /// path.set_d("M 20 20 Q 100 0 180 20")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn path(&self, d: &str) -> Result<SvgNode, Error> {
        self.create_path(d)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<path>` element from a sequence of typed [`PathDef`] segments, appends it to the root and returns
    /// its [`SvgNode`] handle.
    ///
    /// This is the type-safe alternative to [`path`](Self::path): the `d` attribute is built internally by [`build_d`]
    /// from `defs` making malformed path strings impossible.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{PathDef, PathDefAbsolute, SvgRoot, root::utils::Point};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// let path = svg.path_from_defs(&[
    ///     PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(10.0, 10.0))),
    ///     PathDef::Abs(PathDefAbsolute::LineTo(Point::new(100.0, 50.0))),
    ///     PathDef::Abs(PathDefAbsolute::LineTo(Point::new(10.0, 90.0))),
    ///     PathDef::Abs(PathDefAbsolute::ClosePath),
    /// ])?;
    /// path.set_fill("none")?;
    /// path.set_stroke("black")?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn path_from_defs(&self, defs: &[PathDef]) -> Result<SvgNode, Error> {
        self.create_path_from_defs(defs)
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[cfg(test)]
mod unit_tests;
