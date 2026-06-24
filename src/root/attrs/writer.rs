use super::SvgAttrs;
use crate::{Error, SvgNode};

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Chainable attribute writer bound to a single [`SvgNode`].
///
/// Obtain one from [`SvgAttrs::writer`] or [`SvgNode::attrs`](crate::SvgNode::attrs).  Each call returns `&mut Self`, so
/// several attributes can be written in one expression while reusing the same scratch buffer.
pub struct AttrWriter<'a> {
    pub attrs: &'a mut SvgAttrs,
    pub node: &'a SvgNode,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl<'a> AttrWriter<'a> {
    /// Sets a string attribute.
    pub fn set(&mut self, name: &str, value: &str) -> Result<&mut Self, Error> {
        self.attrs.set(self.node, name, value)?;
        Ok(self)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats a displayable value through the reusable scratch buffer and writes it as an attribute.
    pub fn display<T: std::fmt::Display>(&mut self, name: &str, value: T) -> Result<&mut Self, Error> {
        self.attrs.display(self.node, name, value)?;
        Ok(self)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats `args` through the reusable scratch buffer and writes it as an attribute.
    pub fn fmt(&mut self, name: &str, args: std::fmt::Arguments<'_>) -> Result<&mut Self, Error> {
        self.attrs.fmt(self.node, name, args)?;
        Ok(self)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets several borrowed string attributes.
    pub fn apply<I, K, V>(&mut self, attrs: I) -> Result<&mut Self, Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.attrs.apply(self.node, attrs)?;
        Ok(self)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Convenience wrapper for `fill`.
    pub fn fill(&mut self, colour: &str) -> Result<&mut Self, Error> {
        self.set("fill", colour)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Convenience wrapper for `stroke`.
    pub fn stroke(&mut self, colour: &str) -> Result<&mut Self, Error> {
        self.set("stroke", colour)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Convenience wrapper for numeric `stroke-width`.
    pub fn stroke_width(&mut self, width: f64) -> Result<&mut Self, Error> {
        self.display("stroke-width", width)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Convenience wrapper for path-data `d`.
    pub fn d(&mut self, path: &str) -> Result<&mut Self, Error> {
        self.set("d", path)
    }
}
