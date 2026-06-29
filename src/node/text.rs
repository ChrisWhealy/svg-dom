use crate::{SvgNode, error::Error};
use wasm_bindgen::JsCast;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Text measurement
    ///
    /// Returns the rendered advance width of a text element's content, in user units, by wrapping
    /// [`SVGTextContentElement.getComputedTextLength()`]. Returns `None` for non-text elements.
    ///
    /// This reflects the actual font metrics in effect (family, size, `letter-spacing`, `word-spacing`), so it is the
    /// reliable way to discover, for example, the width of a monospace digit (the CSS `ch` unit) at runtime rather than
    /// hard-coding a guess. The element must be attached to a rendered document for the measurement to be meaningful.
    ///
    /// **Performance:** this call triggers a browser layout read — the browser must compute font metrics and text layout
    /// before it can return a value. Do not call it inside a hot animation or pointer-move callback unless you have
    /// determined that this cost is acceptable.
    ///
    /// [`SVGTextContentElement.getComputedTextLength()`]: https://developer.mozilla.org/docs/Web/API/SVGTextContentElement/getComputedTextLength
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::Point, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let probe = svg.text(Point::origin(), "0")?;
    /// let ch = probe.computed_text_length().unwrap_or(0.0); // width of one monospace digit
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn computed_text_length(&self) -> Option<f64> {
        self.inner
            .element
            .dyn_ref::<web_sys::SvgTextContentElement>()
            .map(|t| t.get_computed_text_length() as f64)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Text content
    ///
    /// Replaces the element's text content. Use on a `<text>` element to update the string it displays without
    /// recreating it.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::Point, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let label = svg.text(Point::new(10.0, 20.0), "0")?;
    /// label.set_text("42"); // live-update the displayed value
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_text(&self, content: &str) {
        self.inner.element.set_text_content(Some(content));
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Text content from `format_args!`, through a caller-owned buffer
    ///
    /// Formats `args` into the supplied scratch buffer and sets the result as this element's text content, reusing the
    /// buffer's allocation across calls. This is the text-content counterpart to
    /// [`set_attr_display`](Self::set_attr_display): use it for a label whose value changes on every event — a
    /// coordinate or status readout updated on each `pointermove`, say — where `set_text(&format!(...))` would allocate
    /// and drop a fresh `String` per event.
    ///
    /// Keep one buffer in the handler's state and pass it on every call. If instead the text usually *repeats* between
    /// events, prefer [`CachedAttr::set_text`](crate::CachedAttr::set_text), which skips the DOM write entirely when the
    /// value is unchanged.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::Point, SvgRoot};
    /// let svg     = SvgRoot::attach("diagram")?;
    /// let readout = svg.text(Point::new(10.0, 20.0), "")?;
    ///
    /// let mut buf = String::new();
    /// let (x, y) = (12.0, 34.0);
    /// readout.set_text_fmt(&mut buf, format_args!("box: {x:.0}, {y:.0}"))?; // no per-call String allocation
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_text_fmt(&self, scratch: &mut String, args: std::fmt::Arguments<'_>) -> Result<(), Error> {
        use std::fmt::Write;
        scratch.clear();
        scratch.write_fmt(args)?;
        self.set_text(scratch);
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Text content from a [`Display`](std::fmt::Display) value, through a caller-owned buffer
    ///
    /// Convenience wrapper over [`set_text_fmt`](Self::set_text_fmt) for the common case of a single displayable value
    /// (a counter, a measurement) rather than a formatted string.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::Point, SvgRoot};
    /// let svg   = SvgRoot::attach("diagram")?;
    /// let label = svg.text(Point::new(10.0, 20.0), "")?;
    ///
    /// let mut buf = String::new();
    /// label.set_text_display(&mut buf, 42)?; // live counter, no per-call allocation
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_text_display<T: std::fmt::Display>(&self, scratch: &mut String, value: T) -> Result<(), Error> {
        self.set_text_fmt(scratch, format_args!("{value}"))
    }
}
