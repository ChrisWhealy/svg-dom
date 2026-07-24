use super::{
    super::{Channel, SvgFilter, TransferFunction},
    SpaceSeparated,
};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feComponentTransfer>` primitive to this filter, with one `<feFuncX>` child per `(Channel,
    /// TransferFunction)` pair in `funcs`, in the order given.
    ///
    /// A channel not named in `funcs` gets no `<feFuncX>` child at all, which the SVG spec defines as identical to
    /// giving it an explicit [`TransferFunction::Identity`] — `feComponentTransfer` only touches the channels you
    /// actually mention.
    ///
    /// `funcs` is not deduplicated: naming the same [`Channel`] twice creates two `<feFuncX>` children of the same tag,
    /// in the order given. Per the SVG spec, when a `<feComponentTransfer>` has more than one child for the same
    /// channel, only the *last* one has any effect — the earlier ones are created but ignored, not applied in sequence.
    ///
    /// There is no error for this situation; so if you build `funcs` programmatically, you must take care not to supply
    /// multiple functions for the same `Channel`.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`, otherwise use the returned
    /// [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` explicitly, the same as every other
    /// primitive here. `result` works the same way too.
    ///
    /// # Errors
    ///
    /// - [`Error::Dom`] — the browser refused to create or append the `<feComponentTransfer>` element or any of its
    ///   `<feFuncX>` children.
    /// - [`Error::InvalidTransferFunction`] — a [`TransferFunction::Table`] with exactly one value, or a
    ///   [`TransferFunction::Discrete`] with zero values, was supplied; see that variant's own doc comment for why
    ///   neither has a defined SVG meaning.
    ///
    /// # Example
    ///
    /// Gamma-correct all three colour channels identically, and fade alpha to 60% opacity via a linear remap:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::{Channel, TransferFunction}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("gamma-fade")?;
    /// let gamma = TransferFunction::Gamma { amplitude: 1.0, exponent: 0.45, offset: 0.0 };
    /// flt.component_transfer(&[
    ///     (Channel::Red, gamma.clone()),
    ///     (Channel::Green, gamma.clone()),
    ///     (Channel::Blue, gamma),
    ///     (Channel::Alpha, TransferFunction::Linear { slope: 0.6, intercept: 0.0 }),
    /// ])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn component_transfer(&self, funcs: &[(Channel, TransferFunction)]) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feComponentTransfer", "SvgElement")?;
        for (channel, func) in funcs {
            let child = create_svg_element::<SvgElement>(&self.document, channel.tag(), "SvgElement")?;
            child.set_attribute("type", func.type_str()).map_err(dom_err)?;
            match func {
                TransferFunction::Identity => {},
                TransferFunction::Table(values) => {
                    if values.len() == 1 {
                        return Err(Error::InvalidTransferFunction(
                            "Table must have zero or at least two values; a single value has no defined SVG semantics",
                        ));
                    }
                    self.attrs
                        .borrow_mut()
                        .display_element(&child, "tableValues", SpaceSeparated(values))?;
                },
                TransferFunction::Discrete(values) => {
                    if values.is_empty() {
                        return Err(Error::InvalidTransferFunction(
                            "Discrete must have at least one value; an empty list has no defined SVG semantics",
                        ));
                    }
                    self.attrs
                        .borrow_mut()
                        .display_element(&child, "tableValues", SpaceSeparated(values))?;
                },
                TransferFunction::Linear { slope, intercept } => {
                    let mut attrs = self.attrs.borrow_mut();
                    attrs.display_element(&child, "slope", *slope)?;
                    attrs.display_element(&child, "intercept", *intercept)?;
                },
                TransferFunction::Gamma { amplitude, exponent, offset } => {
                    let mut attrs = self.attrs.borrow_mut();
                    attrs.display_element(&child, "amplitude", *amplitude)?;
                    attrs.display_element(&child, "exponent", *exponent)?;
                    attrs.display_element(&child, "offset", *offset)?;
                },
            }
            el.append_child(&child).map_err(dom_err)?;
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
