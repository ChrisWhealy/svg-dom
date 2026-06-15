use crate::{SvgRoot, error::Error, node::SvgNode};

impl SvgRoot {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
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
    /// | `H` / `h` | `x`                    | Horizontal line         |
    /// | `V` / `v` | `y`                    | Vertical line           |
    /// | `C` / `c` | `x1 y1 x2 y2 x y`      | Cubic Bézier            |
    /// | `S` / `s` | `x2 y2 x y`            | Smooth cubic Bézier     |
    /// | `Q` / `q` | `x1 y1 x y`            | Quadratic Bézier        |
    /// | `T` / `t` | `x y`                  | Smooth quadratic Bézier |
    /// | `A` / `a` | `rx ry rot laf sf x y` | Elliptical arc          |
    /// | `Z` / `z` | —                      | Close path              |
    ///
    /// The path can be updated later without recreating the element using [`SvgNode::set_d`].
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
        let n = self.append_new("path")?;
        n.set_attr("d", d)?;
        Ok(n)
    }
}
