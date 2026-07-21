use super::super::{BlendMode, SvgFilter};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feBlend>` primitive to this filter, blending this primitive's `in` input with `in2` using the
    /// given [`BlendMode`].
    ///
    /// Unlike [`composite`](Self::composite), which combines two inputs geometrically (Porter-Duff, based on where each
    /// input is opaque), `blend` combines them photometrically — how their *colours* mix where both are visible, using
    /// the standard blend-mode maths CSS `mix-blend-mode` also uses. See [`BlendMode`]'s own doc comment for two ways
    /// this is not quite identical to CSS `mix-blend-mode` itself (a smaller keyword set, and `linearRGB` by default).
    ///
    /// `in2` is written directly.
    ///
    /// ***IMPORTANT*** The value of `in2` is not validated. It is typically another primitive's `result` name, or
    /// one of the SVG keyword inputs (`"SourceGraphic"`/`"SourceAlpha"`).
    ///
    /// `in` is not set by this method: if this is the filter's first primitive, its implicit input is `SourceGraphic`,
    /// otherwise use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` explicitly, the same
    /// as every other primitive here.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feBlend>` element.
    ///
    /// # ⚠️ Tinting with a flood colour needs a final `composite(In)` to preserve transparency
    ///
    /// [`flood`](Self::flood) paints its colour *opaquely* across the entire filter region — a rectangle, unrelated
    /// to whatever shape or transparency the source graphic actually has. Blending that flood straight against
    /// `SourceGraphic` (as in the example below) only changes *colour*; it does not touch *alpha*. Per the SVG
    /// filter specification, `feBlend`'s result alpha is the union of its two inputs' alpha, so wherever the flood
    /// is opaque — everywhere in the filter region, including the fully transparent corners of a circle's bounding
    /// box, or the transparent parts of an image — the blended result stays opaque too, and the flood colour shows
    /// through where the source graphic had nothing at all.
    ///
    /// Composite the blended result back `In` the original `SourceGraphic` afterwards to clip it to the source's
    /// own alpha coverage, discarding the leaked flood outside it.
    ///
    /// # Example
    ///
    /// Multiplying a flood colour over the source graphic — a common way to tint an image without flattening its
    /// own shading to a single colour, the way [`composite`](Self::composite)'s `In` operator alone would:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::{BlendMode, CompositeOperator}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let tint = defs.filter("tint")?;
    /// tint.flood("steelblue", 1.0)?.set_attr("result", "colour")?;
    /// tint.blend("colour", BlendMode::Multiply)?
    ///     .set_attrs([("in", "SourceGraphic"), ("result", "tinted")])?;
    /// // Clip back to the source's own alpha coverage — without this, a non-rectangular or partially transparent
    /// // source graphic would show the flood colour leaking through wherever it was itself transparent.
    /// tint.composite("SourceGraphic", CompositeOperator::In)?.set_attr("in", "tinted")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn blend(&self, in2: &str, mode: BlendMode) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feBlend", "SvgElement")?;
        el.set_attribute("in2", in2).map_err(dom_err)?;
        el.set_attribute("mode", mode.as_str()).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
