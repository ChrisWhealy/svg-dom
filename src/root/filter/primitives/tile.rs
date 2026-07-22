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
    /// Two different rectangles are in play here: the *input* primitive's own subregion is the reference tile that
    /// gets repeated; `tile`'s own subregion (its `x`/`y`/`width`/`height`, left at their default in the example
    /// below) is the *destination* rectangle the repetitions fill — specified by the spec to default to the whole
    /// filter region, unlike an ordinary primitive.
    ///
    /// `tile` does not choose the reference tile itself: it is whatever `x`, `y`, `width`, and `height` the *input*
    /// primitive's own subregion was given. An ordinary primitive's default subregion, left unspecified, is
    /// generally the union of its own referenced inputs' subregions — but a generator with no referenced input, such
    /// as [`turbulence`](Self::turbulence) (used below) or [`image`](Self::image), defaults instead to the whole
    /// filter region, the same as when a referenced input is a standard input like `SourceGraphic`. So if the
    /// turbulence in the example below were left unnarrowed, there would be nothing smaller than the full region to
    /// repeat, and `tile`'s output would be indistinguishable from its unchanged input.
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
