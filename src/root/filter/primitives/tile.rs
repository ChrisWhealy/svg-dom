use super::super::SvgFilter;
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feTile>` primitive to this filter, replicating its input across the whole of this primitive's own
    /// subregion.
    ///
    /// `<feTile>` has no numeric or enum-valued attributes needing a typed parameter. It accepts the optional input
    /// selector `in`, together with the common filter-primitive attributes `x`, `y`, `width`, `height`, and `result`
    /// — none of them wrapped by a named parameter here, so the returned [`SvgNode`]'s
    /// [`set_attr`](crate::SvgNode::set_attr) and [`set_attrs`](crate::SvgNode::set_attrs) covers all of them.
    ///
    /// ***⚠️ The tile is the input's own primitive subregion — narrow it, or tiling has no visible effect***
    ///
    /// The rectangle that gets repeated is not chosen by `tile` itself: it is whatever the `x`, `y`, `width`, and
    /// `height` of the *input* primitive's own subregion was given. A primitive's default subregion (when its `x`, `y`,
    /// `width`, and `height` are left unset) is the whole filter region. So if the input was never narrowed, there is
    /// nothing smaller than the full region to repeat and `tile`'s output is indistinguishable from its unchanged
    /// input.
    ///
    /// To get a visible tiling effect, explicitly set a smaller `x`, `y`, `width`, and `height` on the *input*
    /// primitive (via its own returned [`SvgNode`]'s `set_attr`/`set_attrs`) before reading it here — see the example
    /// below.
    ///
    /// This is a different mechanism from [`SvgDefs::pattern`](crate::SvgDefs::pattern) or
    /// [`SvgDefs::build_pattern`](crate::SvgDefs::build_pattern): a `<pattern>` is a paint server, applied via `fill`
    /// and `stroke`. `tile` instead repeats a *filter-generated* tile as one step inside a filter graph, so it can feed
    /// further primitives (colour-transformed, blended, composited, ...) the same way any other primitive's output can.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feTile>` element.
    ///
    /// # Example
    ///
    /// Generate noise, narrow it to a small tile, then repeat that tile across the whole filter region:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::TurbulenceType};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("tiled-noise")?;
    ///
    /// flt.turbulence(0.2, 2, 4.0, TurbulenceType::FractalNoise)?
    ///     .set_attrs([("x", "0"), ("y", "0"), ("width", "20"), ("height", "20")])?;
    /// flt.tile()?;
    ///
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn tile(&self) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feTile", "SvgElement")?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
