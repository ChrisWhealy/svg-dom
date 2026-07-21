use super::super::{Channel, SvgFilter, TransferFunction};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use std::fmt;
use web_sys::SvgElement;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Formats a slice of `f64` space-separated, straight into the caller's `fmt::Formatter` — used to write
/// `tableValues` through [`SvgAttrs::display_element`](crate::root::attrs::SvgAttrs::display_element)'s scratch
/// buffer with no intermediate `String`/`Vec` allocation, the same technique this crate's internal `write_points`
/// helper uses for the `points` attribute.
struct SpaceSeparated<'a>(&'a [f64]);

impl fmt::Display for SpaceSeparated<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{v}")?;
        }
        Ok(())
    }
}

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feComponentTransfer>` primitive to this filter, with one `<feFuncX>` child per `(Channel,
    /// TransferFunction)` pair in `funcs`, in the order given.
    ///
    /// A channel not named in `funcs` gets no `<feFuncX>` child at all, which the SVG spec defines as identical to
    /// giving it an explicit [`TransferFunction::Identity`] — `feComponentTransfer` only touches the channels you
    /// actually mention.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`, otherwise use the returned
    /// [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` explicitly, the same as every other
    /// primitive here. `result` works the same way too.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feComponentTransfer>` element or any
    /// of its `<feFuncX>` children.
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
                TransferFunction::Table(values) | TransferFunction::Discrete(values) => {
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
