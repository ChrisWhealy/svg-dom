use crate::{SvgRoot, error::Error, node::SvgNode, root::factory::SvgFactory};

impl SvgRoot {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates a `<style>` element containing `css`, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// `<style>` embeds a CSS stylesheet directly in the SVG document (the equivalent of an HTML `<style>` block).
    ///
    /// # Scope
    ///
    /// The style rules used here apply document-wide, regardless of where in the DOM tree they occur. Unlike
    /// `<defs>` content, which is only reachable by reference, SVG's CSS cascade is not scoped by DOM position. As a
    /// result, callers can select elements by `class`, `id`, or any other CSS selector, the same way they would
    /// style an HTML page.
    ///
    /// `css` is written as the element's text content via [`SvgNode::set_text`](crate::SvgNode::set_text) — plain text,
    /// not markup, so no HTML entity-escaping is needed for `<`/`>`/`&` inside the stylesheet.
    ///
    /// # Security
    ///
    /// ⚠️ Caveat ⚠️
    ///
    /// `css` content is written verbatim, with no validation or sanitisation!
    ///
    /// Do not pass attacker-controlled CSS without validating it first. A `url(...)` value can still fetch an arbitrary
    /// attacker-chosen resource, which is enough for tracking or, depending on what else is reachable from that fetch,
    /// more serious data exfiltration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<style>` element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    ///
    /// let svg = SvgRoot::attach("diagram")?;
    /// svg.style(".pulse { animation: pulse 2s ease-in-out infinite; } @keyframes pulse { 50% { opacity: 0.4; } }")?;
    ///
    /// let dot = svg.circle(Point::new(40.0, 40.0), 12.0)?;
    /// dot.set_attr("class", "pulse")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn style(&self, css: &str) -> Result<SvgNode, Error> {
        self.create_style(css)
    }
}
