use super::super::{CompositeOperator, SvgFilter};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feComposite>` primitive to this filter, combining this primitive's `in` input with `in2` using
    /// the given Porter-Duff [`operator`](CompositeOperator).
    ///
    /// ***⚠️ [`CompositeOperator::Arithmetic`] needs `k1`–`k4` to be set manually*** — see that variant's own doc for
    /// why skipping them silently produces transparent black rather than an error.
    ///
    /// `in2` is written directly.
    ///
    /// ***IMPORTANT*** The value of `in2` is not validated.  It is typically another primitive's `result` name, or one
    /// of the SVG keyword inputs `"SourceGraphic"`/`"SourceAlpha"`).
    ///
    /// `in` is not set by this method: if this is the filter's first primitive, its implicit input is `SourceGraphic`,
    /// otherwise use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` explicitly, the same
    /// as every other primitive here.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feComposite>` element.
    ///
    /// # Example
    ///
    /// A true tinted, semi-transparent drop shadow (as opposed to the blurred-copy approximation produced by
    /// [`merge`](Self::merge)) by flooding a colour and compositing it into the blurred alpha mask before offsetting
    /// and merging it underneath the original graphic:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::CompositeOperator};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let shadow = defs.filter("shadow")?;
    /// shadow.gaussian_blur(4.0)?.set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
    /// shadow.flood("black", 0.5)?.set_attr("result", "colour")?;
    /// shadow.composite("blur", CompositeOperator::In)?.set_attrs([("in", "colour"), ("result", "tinted")])?;
    /// shadow.offset(4.0, 4.0)?.set_attrs([("in", "tinted"), ("result", "offset-shadow")])?;
    /// shadow.merge(&["offset-shadow", "SourceGraphic"])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn composite(&self, in2: &str, operator: CompositeOperator) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feComposite", "SvgElement")?;
        el.set_attribute("in2", in2).map_err(dom_err)?;
        el.set_attribute("operator", operator.as_str()).map_err(dom_err)?;
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
