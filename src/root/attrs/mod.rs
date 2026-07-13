mod writer;

pub use writer::AttrWriter;

use crate::{
    Error, SvgNode, dom_err,
    root::utils::{Point, write_points},
};
use std::fmt::{self, Write};
use web_sys::Element;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Reusable scratch buffer for writing SVG attributes.
///
/// `SvgAttrs` is useful when many numeric or formatted attributes need to be written in succession.  The internal
/// `String` is allocated once and reused for every [`display`](AttrWriter::display) or [`fmt`](AttrWriter::fmt) call,
/// avoiding the short-lived allocations caused by `value.to_string()` and `format!(...)`.
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgAttrs, SvgRoot, root::utils::{Point, Size}};
///
/// let svg = SvgRoot::attach("diagram")?;
/// let rect = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
///
/// let mut attrs = SvgAttrs::new();
/// attrs.writer(&rect)
///     .set("fill", "steelblue")?
///     .set("stroke", "white")?
///     .display("stroke-width", 2.0)?
///     .fmt("transform", format_args!("translate({:.1}, {:.1})", 10.0, 20.0))?;
/// # Ok::<(), svg_dom::Error>(())
/// ```
#[derive(Debug, Default)]
pub struct SvgAttrs {
    scratch: String,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgAttrs {
    /// Creates an empty reusable attribute writer.
    pub fn new() -> Self {
        Self::default()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a reusable attribute writer with pre-allocated scratch capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            scratch: String::with_capacity(capacity),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the currently allocated scratch capacity.
    pub fn capacity(&self) -> usize {
        self.scratch.capacity()
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Clears the scratch buffer without releasing its allocation.
    pub fn clear(&mut self) {
        self.scratch.clear();
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Binds this reusable buffer to a node and returns a chainable writer.
    pub fn writer<'a>(&'a mut self, node: &'a SvgNode) -> AttrWriter<'a> {
        AttrWriter::new(self, node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets a string attribute using the reusable writer.
    pub fn set(&mut self, node: &SvgNode, name: &str, value: &str) -> Result<(), Error> {
        node.set_attr(name, value)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats a displayable value into the reusable scratch buffer and writes it as an attribute.
    pub fn display<T: fmt::Display>(&mut self, node: &SvgNode, name: &str, value: T) -> Result<(), Error> {
        self.scratch.clear();
        write!(self.scratch, "{value}")?;
        node.set_attr(name, &self.scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats `args` into the reusable scratch buffer and writes it as an attribute.
    pub fn fmt(&mut self, node: &SvgNode, name: &str, args: fmt::Arguments<'_>) -> Result<(), Error> {
        self.scratch.clear();
        self.scratch.write_fmt(args)?;
        node.set_attr(name, &self.scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats `points` into the reusable scratch buffer as an SVG `points` list (`"x,y x,y ..."`) and writes it as the
    /// node's `points` attribute.
    ///
    /// This is the allocation-light way to set or update the vertices of a `<polyline>` or `<polygon>` — for instance
    /// a shape whose points are recomputed every animation frame. Reusing one `SvgAttrs` buffer across calls avoids
    /// the fresh `String` that [`SvgRoot::polyline`](crate::SvgRoot::polyline) /
    /// [`SvgRoot::polygon`](crate::SvgRoot::polygon) would otherwise build per call.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgAttrs, SvgRoot, root::utils::Point};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// let poly = svg.polyline(&[Point::origin()])?;
    ///
    /// let mut attrs = SvgAttrs::new();
    /// // Re-point the polyline without allocating a new string each time.
    /// attrs.points(&poly, &[Point::new(0.0, 0.0), Point::new(20.0, 40.0), Point::new(40.0, 0.0)])?;
    /// # Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn points(&mut self, node: &SvgNode, points: &[Point]) -> Result<(), Error> {
        write_points(&mut self.scratch, points, None);
        node.set_attr("points", &self.scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Like [`points`](Self::points), but writes each coordinate with `dps` fixed decimal places.
    ///
    /// Use this for large or animated `<polyline>`/`<polygon>` geometry where sub-pixel precision is less important
    /// than performance: it shortens the `points` string meaning less data crosses the WASM/JS boundary and uses less
    /// DOM attribute storage.
    ///
    /// `dps` is clamped to 20 — `f64` only carries ~17 significant digits, so values above that produce meaningless
    /// trailing zeros with no benefit.
    ///
    /// Prefer [`points`](Self::points) when exact coordinates matter.
    pub fn points_fixed(&mut self, node: &SvgNode, points: &[Point], dps: usize) -> Result<(), Error> {
        write_points(&mut self.scratch, points, Some(dps));
        node.set_attr("points", &self.scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets several borrowed string attributes on a node.
    pub fn apply<I, K, V>(&mut self, node: &SvgNode, attrs: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (name, value) in attrs {
            self.set(node, name.as_ref(), value.as_ref())?;
        }
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    pub(crate) fn display_element<T: fmt::Display>(
        &mut self,
        el: &impl AsRef<Element>,
        name: &str,
        value: T,
    ) -> Result<(), Error> {
        self.scratch.clear();
        write!(self.scratch, "{value}")?;
        el.as_ref().set_attribute(name, &self.scratch).map_err(dom_err)
    }
}
