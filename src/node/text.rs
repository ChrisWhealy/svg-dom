use crate::{
    SvgNode, dom_err,
    error::Error,
    root::{SVG_NS, attrs::SvgAttrs},
};
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
/// Controls how glyphs are fitted onto the path referenced by a `<textPath>`.
///
/// Maps to the `method` attribute of `<textPath>`.
/// The default is [`Align`](TextPathMethod::Align).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextPathMethod {
    /// Each glyph is rendered at its natural size, then rotated and translated so it sits on the path.
    /// This is the SVG default and the only method with broad, reliable browser support.
    Align,
    /// Each glyph outline is stretched so consecutive glyphs follow the path's curvature without gaps.
    ///
    /// ⚠️ Caution ⚠️
    /// Support for `Stretch` is inconsistent across browser engines and can result in visual distortion; therefore,
    /// unless you have verified that `Stretch` renders acceptably on every target browser, you should prefer the use
    /// of `Align`.
    Stretch,
}

impl TextPathMethod {
    fn as_str(self) -> &'static str {
        match self {
            TextPathMethod::Align => "align",
            TextPathMethod::Stretch => "stretch",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls glyph-spacing adjustment along the path referenced by a `<textPath>`.
///
/// Maps to the `spacing` attribute of `<textPath>`.
/// The default is [`Auto`](TextPathSpacing::Auto).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextPathSpacing {
    /// The renderer automatically adjusts spacing to compensate for the path's curvature (SVG default).
    Auto,
    /// Spacing between glyphs follows the font's natural advance widths exactly, with no curvature compensation —
    /// text can visibly bunch or gap on tight curves.
    Exact,
}

impl TextPathSpacing {
    fn as_str(self) -> &'static str {
        match self {
            TextPathSpacing::Auto => "auto",
            TextPathSpacing::Exact => "exact",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// Controls which side of the path a `<textPath>`'s text is rendered on.
///
/// Maps to the SVG2 `side` attribute of `<textPath>`.
/// The default is [`Left`](TextPathSide::Left).
///
/// **Browser support caveat:** `side` is an SVG2 addition; verify it renders as expected on every browser you target
/// before relying on [`Right`](TextPathSide::Right) in production.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextPathSide {
    /// Text is placed along the path in its natural direction, reading left-to-right as the path is traversed
    /// (SVG default).
    Left,
    /// Text is placed on the opposite side of the path, as if the path were reversed.
    Right,
}

impl TextPathSide {
    fn as_str(self) -> &'static str {
        match self {
            TextPathSide::Left => "left",
            TextPathSide::Right => "right",
        }
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    /// # Text measurement
    ///
    /// Returns the rendered advance width of a text element's content, in user units, by wrapping
    /// [`SVGTextContentElement.getComputedTextLength()`]. Returns `None` for non-text elements.
    ///
    /// This reflects the actual font metrics in effect (family, size, `letter-spacing`, `word-spacing` etc), so it is
    /// the most reliable way to discover, for example, the width of a monospace digit (the CSS `ch` unit) at runtime
    /// rather than hard-coding a guess.
    ///
    /// The element must be attached to a rendered document for the measurement to be meaningful.
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

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the horizontal offset in user units, relative to the current text position.
    ///
    /// Useful on `<tspan>` children to shift individual words or characters horizontally without forcing a full
    /// absolute repositioning via `x`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let txt = svg.text(Point::new(20.0, 50.0), "")?;
    /// let span = txt.tspan("shifted")?;
    /// span.set_dx(8.0)?; // move 8 user units right
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_dx(&self, dx: f64) -> Result<(), Error> {
        let mut attrs = SvgAttrs::new();
        attrs.display(self, "dx", dx)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the vertical offset in user units, relative to the current text position.
    ///
    /// `dy` shifts the text position downward (positive values) or upward (negative values) without resetting the
    /// horizontal position.  This makes it useful for superscripts, subscripts, and controlled vertical nudges,
    /// but **not** for aligned multi-line text: after each span the horizontal position advances by the glyph advance
    /// width, so successive `dy`-only spans drift rightward.
    ///
    /// For aligned multi-line text — where every line should start at the same `x` coordinate — use
    /// [`tspan_line`](Self::tspan_line), which combines an absolute `x` reset with a `dy` advance.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let txt = svg.text(Point::new(20.0, 50.0), "")?;
    /// let span = txt.tspan("H")?;
    /// let sup  = span.tspan("2")?;
    /// sup.set_dy(-6.0)?;  // raise the superscript above the baseline
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_dy(&self, dy: f64) -> Result<(), Error> {
        let mut attrs = SvgAttrs::new();
        attrs.display(self, "dy", dy)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<tspan>` child element with `content` as its text, appends it to `self` and returns a handle.
    ///
    /// `<tspan>` inherits all text presentation attributes (`font-family`, `font-size`, `fill` etc) from its `<text>`
    /// parent; any attribute set on the returned `SvgNode` overrides the inherited value for that span only.
    ///
    /// The first `<tspan>` in a `<text>` inherits the parent's `x` / `y` position.
    /// Subsequent spans need a `dy` (or `dx`) to advance the current text position; use [`tspan_dy`](Self::tspan_dy)
    /// as a convenience, or call [`set_dy`](Self::set_dy) on the returned handle.
    ///
    /// When a `<text>` element contains `<tspan>` children the text content set directly on `<text>` should be empty
    /// (the factory sets it to `""` for you when you call [`SvgRoot::text`](crate::SvgRoot::text) with `""`).
    ///
    /// # Errors
    ///
    /// - [`Error::Dom`] — the browser refused to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let txt = svg.text(Point::new(20.0, 50.0), "")?;
    /// let line1 = txt.tspan("Hello, ")?;
    /// let line2 = txt.tspan("world")?;
    /// line2.set_fill("steelblue")?;
    /// line2.set_font_size(20.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn tspan(&self, content: &str) -> Result<SvgNode, Error> {
        let doc = self
            .inner
            .element
            .owner_document()
            .ok_or_else(|| Error::Dom("tspan: element has no owner document".into()))?;
        let el = doc
            .create_element_ns(Some(SVG_NS), "tspan")
            .map_err(dom_err)?
            .dyn_into::<web_sys::SvgElement>()
            .map_err(|_| Error::CastFailed("SvgElement"))?;
        let node = SvgNode::new(el);
        node.set_text(content);
        self.inner.element.append_child(node.as_element()).map_err(dom_err)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<tspan>` child with `content` and a relative vertical displacement `dy`, then returns the handle.
    ///
    /// Sets only `dy` — **no `x` attribute is written**.  The span continues the text run at the horizontal position
    /// where the previous span ended, then shifts down by `dy` user units.  Subsequent spans therefore start further
    /// right than the first line, producing a staircase rather than left-aligned text.
    ///
    /// This is correct for superscripts, subscripts, and deliberate vertical nudges within a single run.
    /// For aligned multi-line text use [`tspan_line`](Self::tspan_line), which also sets an absolute `x` coordinate.
    ///
    /// # Errors
    ///
    /// - [`Error::Dom`] — the browser refused to create or append the element, or failed to write `dy`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let txt  = svg.text(Point::new(20.0, 60.0), "CO")?;
    /// let sub2 = txt.tspan("2")?;
    /// sub2.set_font_size(9.0)?;
    /// txt.tspan_dy(4.0, "")?;  // nudge back down after a manual superscript/subscript
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn tspan_dy(&self, dy: f64, content: &str) -> Result<SvgNode, Error> {
        let node = self.tspan(content)?;
        node.set_dy(dy)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<tspan>` child with `content`, an absolute horizontal position `x`, and a relative vertical
    /// displacement `dy`, then returns the handle.
    ///
    /// This is the idiomatic way to produce aligned multi-line text inside a `<text>` element.
    /// The `x` attribute resets the horizontal start position to an absolute coordinate for each new line, so every
    /// line begins at the same `x` regardless of the rendered width of the preceding content.
    /// The `dy` attribute then advances the vertical position by that many user units relative to the previous line.
    ///
    /// The element is constructed detached: the `x` and `dy` attributes are written before the node is appended to the
    /// parent, so any failures during construction leave the parent element unchanged.
    ///
    /// # Errors
    ///
    /// - [`Error::Dom`] — the browser refused to create the element or write an attribute.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let txt = svg.text(Point::new(20.0, 40.0), "")?;
    /// txt.tspan("Line one")?;                      // inherits x=20 from <text>
    /// txt.tspan_line(20.0, 20.0, "Line two")?;    // resets to x=20, advances 20 down
    /// txt.tspan_line(20.0, 20.0, "Line three")?;  // same — each line starts at x=20
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn tspan_line(&self, x: f64, dy: f64, content: &str) -> Result<SvgNode, Error> {
        let doc = self
            .inner
            .element
            .owner_document()
            .ok_or_else(|| Error::Dom("tspan_line: element has no owner document".into()))?;
        let el = doc
            .create_element_ns(Some(SVG_NS), "tspan")
            .map_err(dom_err)?
            .dyn_into::<web_sys::SvgElement>()
            .map_err(|_| Error::CastFailed("SvgElement"))?;
        let node = SvgNode::new(el);
        node.set_text(content);
        // Write x and dy before appending so a write failure leaves the parent unchanged.
        let mut attrs = SvgAttrs::new();
        attrs.display(&node, "x", x)?;
        attrs.display(&node, "dy", dy)?;
        self.inner.element.append_child(node.as_element()).map_err(dom_err)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<textPath>` child element that glues `content` to the path referenced by `href`, appends it to
    /// `self` (a `<text>` element) and returns a handle.
    ///
    /// # Arguments
    ///
    /// * `href` — fragment reference to the path the text should follow, e.g. `"#wave"`, where `"wave"` is the `id`
    ///   attribute of the target `<path>` (or, per SVG2, a basic shape such as `<circle>` or `<rect>`).
    ///   The target is typically defined inside [`SvgDefs`](crate::SvgDefs) so it is not rendered on its own; give it
    ///   a stroke and no fill (or place it in `<defs>`) to keep the guide path invisible.
    /// * `content` — the visible text string that is drawn along the path, starting from
    ///   [`set_start_offset`](Self::set_start_offset) (default `0`, the path's own start point).
    ///
    /// Like [`tspan`](Self::tspan), the returned handle accepts the usual text styling helpers (`set_fill`,
    /// `set_font_size`, `set_font_family`) which override whatever the parent `<text>` inherited, plus the
    /// `<textPath>`-specific setters [`set_start_offset`](Self::set_start_offset),
    /// [`set_text_path_method`](Self::set_text_path_method), [`set_text_path_spacing`](Self::set_text_path_spacing),
    /// and [`set_text_path_side`](Self::set_text_path_side).
    ///
    /// # Errors
    ///
    /// - [`Error::Dom`] — the browser refused to create or append the element, or failed to write `href`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    ///
    /// // An invisible guide path the text will follow.
    /// let arc = defs.path("M 20 120 Q 200 20 380 120")?;
    /// arc.set_attr("id", "arc")?;
    ///
    /// let txt = svg.text(Point::origin(), "")?;
    /// let path_text = txt.text_path("#arc", "Curving along the path")?;
    /// path_text.set_font_size(16.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn text_path(&self, href: &str, content: &str) -> Result<SvgNode, Error> {
        let doc = self
            .inner
            .element
            .owner_document()
            .ok_or_else(|| Error::Dom("text_path: element has no owner document".into()))?;
        let el = doc
            .create_element_ns(Some(SVG_NS), "textPath")
            .map_err(dom_err)?
            .dyn_into::<web_sys::SvgElement>()
            .map_err(|_| Error::CastFailed("SvgElement"))?;
        let node = SvgNode::new(el);
        node.set_attr("href", href)?;
        node.set_text(content);
        self.inner.element.append_child(node.as_element()).map_err(dom_err)?;
        Ok(node)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `startOffset` attribute on a `<textPath>`, in user units measured along the referenced path.
    ///
    /// Determines where the text begins: `0.0` (the default) starts at the path's own start point; larger values
    /// slide the text further along the path (in the direction determined by [`TextPathSide`]) before the first
    /// glyph is drawn.
    ///
    /// To offset by a percentage of the path length instead of an absolute distance, use
    /// [`set_attr`](Self::set_attr) directly, e.g. `path_text.set_attr("startOffset", "50%")`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let arc  = defs.path("M 20 120 Q 200 20 380 120")?;
    /// arc.set_attr("id", "arc")?;
    ///
    /// let txt = svg.text(Point::origin(), "")?;
    /// let path_text = txt.text_path("#arc", "starts halfway along")?;
    /// path_text.set_start_offset(190.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_start_offset(&self, offset: f64) -> Result<(), Error> {
        let mut attrs = SvgAttrs::new();
        attrs.display(self, "startOffset", offset)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `method` attribute on a `<textPath>`, controlling how glyphs are fitted onto the path.
    ///
    /// See [`TextPathMethod`] for the available values; the SVG default is [`TextPathMethod::Align`].
    pub fn set_text_path_method(&self, method: TextPathMethod) -> Result<(), Error> {
        self.set_attr("method", method.as_str())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `spacing` attribute on a `<textPath>`, controlling glyph-spacing adjustment along the path.
    ///
    /// See [`TextPathSpacing`] for the available values; the SVG default is [`TextPathSpacing::Auto`].
    pub fn set_text_path_spacing(&self, spacing: TextPathSpacing) -> Result<(), Error> {
        self.set_attr("spacing", spacing.as_str())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the SVG2 `side` attribute on a `<textPath>`, controlling which side of the path the text renders on.
    ///
    /// See [`TextPathSide`] for the available values and a browser-support caveat; the SVG default is
    /// [`TextPathSide::Left`].
    pub fn set_text_path_side(&self, side: TextPathSide) -> Result<(), Error> {
        self.set_attr("side", side.as_str())
    }
}
