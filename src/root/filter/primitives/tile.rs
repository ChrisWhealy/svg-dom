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
    /// [`set_attr`](crate::SvgNode::set_attr) and [`set_attrs`](crate::SvgNode::set_attrs) cover all of them.
    ///
    /// ***⚠️ The reference tile must be smaller than `tile`'s own output subregion, or tiling has no visible effect***
    ///
    /// Two different rectangles are in play here: the selected input's *effective* primitive subregion is the
    /// reference tile that gets repeated; `tile`'s own subregion (its `x`/`y`/`width`/`height`, left at their
    /// default in the example below) is the *destination* rectangle the repetitions fill — specified by the spec to
    /// default to the whole filter region, unlike an ordinary primitive.
    ///
    /// `tile` does not choose the reference tile itself: it is whatever the selected input's effective subregion
    /// resolves to. An ordinary primitive's default subregion, left unspecified, is the union of its own referenced
    /// inputs' subregions — so the immediate input to `tile` need not itself carry explicit `x`/`y`/`width`/`height`;
    /// it inherits a narrower default from an earlier primitive in the chain that was itself explicitly narrowed. A
    /// generator with no referenced input, such as [`turbulence`](Self::turbulence) (used below) or
    /// [`image`](Self::image), has no earlier primitive to inherit from, so its own default subregion is the whole
    /// filter region instead — which is why the example below narrows it explicitly rather than relying on
    /// inheritance.
    ///
    /// Whichever primitive in the chain actually needs narrowing, its effective subregion must be smaller than
    /// `tile`'s own output subregion, or there is nothing smaller than the destination rectangle to repeat, and
    /// `tile`'s output is indistinguishable from its unchanged input.
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
