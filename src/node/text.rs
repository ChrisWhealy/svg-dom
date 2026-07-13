use crate::{SvgNode, error::Error, root::attrs::SvgAttrs};
use wasm_bindgen::JsCast;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls which part of a `<text>` string aligns with the element's `x` coordinate.
///
/// Maps to the SVG `text-anchor` presentation attribute.
/// The default is [`Start`](TextAnchor::Start).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAnchor {
    /// The beginning of the text string is placed at the `x` coordinate.
    /// Default for left-to-right text.
    Start,
    /// The midpoint of the text string is placed at the `x` coordinate.
    Middle,
    /// The end of the text string is placed at the `x` coordinate.
    End,
}

impl TextAnchor {
    fn as_str(self) -> &'static str {
        match self {
            TextAnchor::Start => "start",
            TextAnchor::Middle => "middle",
            TextAnchor::End => "end",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Specifies which font baseline aligns with the `y` coordinate of a `<text>` element.
///
/// Maps to the SVG `dominant-baseline` presentation attribute.
/// The default is [`Auto`](DominantBaseline::Auto), which behaves like [`Alphabetic`](DominantBaseline::Alphabetic)
/// for most horizontal Latin text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DominantBaseline {
    /// Browser-determined; equivalent to `Alphabetic` for horizontal Latin text.
    Auto,
    /// The alphabetic baseline — the bottom of most Latin lowercase letters. The SVG default.
    Alphabetic,
    /// The midpoint of the em square aligns with `y`. Useful for centring text vertically on a point.
    Middle,
    /// The ideographic underline baseline (CJK scripts).
    Ideographic,
    /// The hanging baseline — used for Devanagari, Tibetan, and other Indic scripts.
    Hanging,
    /// The mathematical baseline, typically used for centred mathematical notation.
    Mathematical,
    /// The centre of the em square, derived from the font's own vertical metrics.
    Central,
    /// The bottom edge of the em square.
    TextBottom,
    /// The top edge of the em square.
    TextTop,
}

impl DominantBaseline {
    fn as_str(self) -> &'static str {
        match self {
            DominantBaseline::Auto => "auto",
            DominantBaseline::Alphabetic => "alphabetic",
            DominantBaseline::Middle => "middle",
            DominantBaseline::Ideographic => "ideographic",
            DominantBaseline::Hanging => "hanging",
            DominantBaseline::Mathematical => "mathematical",
            DominantBaseline::Central => "central",
            DominantBaseline::TextBottom => "text-bottom",
            DominantBaseline::TextTop => "text-top",
        }
    }
}

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

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `font-family` attribute.
    ///
    /// Accepts any CSS font-family value: a single family name (`"serif"`, `"monospace"`, `"Arial"`), a comma-separated
    /// fallback list (`"\"Helvetica Neue\", Arial, sans-serif"`), or a generic family keyword.
    ///
    /// Font family names that contain spaces must be quoted.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg   = SvgRoot::attach("diagram")?;
    /// let label = svg.text(Point::new(20.0, 40.0), "Hello")?;
    /// label.set_font_family("monospace")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_font_family(&self, family: &str) -> Result<(), Error> {
        self.set_attr("font-family", family)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `font-size` attribute in user units.
    ///
    /// This convenience setter formats `size` into a short-lived `String` allocated and dropped on each call (which is
    /// fine if you only need to perform a one-off styling);  however, if you need to animate the font size on a hot
    /// path, prefer [`set_attr_display`](Self::set_attr_display) with a reused buffer instead.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg   = SvgRoot::attach("diagram")?;
    /// let label = svg.text(Point::new(20.0, 40.0), "Hello")?;
    /// label.set_font_size(24.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_font_size(&self, size: f64) -> Result<(), Error> {
        let mut attrs = SvgAttrs::new();
        attrs.display(self, "font-size", size)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `text-anchor` attribute, controlling which part of the text string aligns with the element's `x`
    /// coordinate.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, TextAnchor, root::utils::Point};
    /// let svg   = SvgRoot::attach("diagram")?;
    /// let label = svg.text(Point::new(400.0, 40.0), "centred")?;
    /// label.set_text_anchor(TextAnchor::Middle)?;  // horizontally centred on x=400
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_text_anchor(&self, anchor: TextAnchor) -> Result<(), Error> {
        self.set_attr("text-anchor", anchor.as_str())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `dominant-baseline` attribute, controlling which font baseline aligns with the element's `y`
    /// coordinate.
    ///
    /// The default SVG behaviour (`Auto`/`Alphabetic`) places the text so that the alphabetic baseline sits on `y` —
    /// meaning ascenders rise above `y` and descenders fall below it.
    /// Use `Middle` or `Central` to vertically centre text on a point, or `Hanging` for scripts whose body hangs from
    /// the top of the line box.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, DominantBaseline, root::utils::Point};
    /// let svg   = SvgRoot::attach("diagram")?;
    /// let label = svg.text(Point::new(100.0, 90.0), "centred")?;
    /// label.set_dominant_baseline(DominantBaseline::Middle)?; // vertically centred on y=90
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_dominant_baseline(&self, baseline: DominantBaseline) -> Result<(), Error> {
        self.set_attr("dominant-baseline", baseline.as_str())
    }
}
