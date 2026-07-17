use crate::{
    dom_err,
    error::Error,
    root::{
        attrs::{AttrWriter, SvgAttrs},
        clip_path::SvgClipPath,
        defs::{
            URL_PREFIX, validate_clip_path_id, validate_filter_id, validate_gradient_id, validate_marker_id,
            validate_mask_id, validate_pattern_id,
        },
        filter::SvgFilter,
        gradient::{linear::SvgLinearGradient, radial::SvgRadialGradient},
        marker::SvgMarker,
        mask::SvgMask,
        path::path_def::{PathDef, build_d, validate_starts_with_moveto},
        pattern::SvgPattern,
    },
};

use super::SvgNode;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgNode {
    /// # Attribute access
    ///
    /// Sets an arbitrary attribute on this element.
    ///
    /// This is the low-level setter used by all the convenience methods such as `set_fill` and `set_stroke`, etc.
    /// Use it when you need to set an attribute not yet wrapped by a typed helper.
    ///
    /// # Security
    ///
    /// `name` and `value` are written **verbatim** via `setAttribute`. Setting an event-handler attribute (`onclick`,
    /// `onload`, ...) or an `href` of the form `javascript:...` from attacker-controlled input can execute script. Do not
    /// pass untrusted data as an attribute name or value without validating it first.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(100.0, 50.0))?;
    ///
    /// rect.set_attr("rx", "8")?;           // set radius of rounded corners
    /// rect.set_attr("opacity", "0.75")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_attr(&self, name: &str, value: &str) -> Result<(), Error> {
        self.inner.element.set_attribute(name, value).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Write an attribute only when it changes
    ///
    /// Reads the current value with `get_attribute` but writes it only if the value changes. This avoids a redundant DOM
    /// write in handlers where a value that doesn't change very often might be arbitrarily rewritten by the event
    /// handler. For example, a cursor style or `opacity` flag is updated on every `mousemove`/`pointermove`.
    ///
    /// **WARNING** This is not free and does not always represent a win.
    ///
    /// The read performed by `get_attribute` **allocates a fresh `String` for the current value which then crosses the
    /// WASM/JS boundary on every call**, even if nothing is written.
    ///
    /// So:
    ///
    /// * For attributes that change on *every* call (such as a drag `transform`), the plain [`set_attr`](Self::set_attr)
    ///   is cheaper — skip this entirely.
    /// * For *occasional* de-duplication it is fine as-is.
    /// * For a *genuinely high-frequency* path where the value usually repeats, prefer [`crate::node::CachedAttr`],
    ///   which remembers the last value on the Rust side: the unchanged case is then a plain string comparison with no
    ///   allocation and no call into JS at all.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg     = SvgRoot::attach("diagram")?;
    /// let surface = svg.rect(Point::origin(), Size::new(100.0, 50.0))?;
    ///
    /// // Called many times per second from a pointermove handler; the DOM is only touched when the cursor
    /// // actually needs to change.
    /// surface.set_attr_if_changed("style", "cursor:grab")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_attr_if_changed(&self, name: &str, value: &str) -> Result<(), Error> {
        if self.inner.element.get_attribute(name).as_deref() == Some(value) {
            return Ok(());
        }
        self.set_attr(name, value)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Write a numeric (or otherwise [`Display`](std::fmt::Display)) attribute through a caller-owned buffer
    ///
    /// Formats `value` into the supplied scratch buffer and writes it as `name`, reusing the buffer's allocation across
    /// calls. This is the allocation-light counterpart to the convenience numeric setters such as
    /// [`set_stroke_width`](Self::set_stroke_width), which each allocate a short-lived `String` per call.
    ///
    /// Reach for this on hot paths that update a numeric attribute every event or frame — an animated `stroke-width`, a
    /// live `rx`, `font-size`, `r`, and the like. Keep one buffer in the handler's state and pass it on every call, the
    /// same pattern the [transform setters](Self::set_translate) use.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let ring = svg.circle(Point::new(50.0, 50.0), 20.0)?;
    ///
    /// let mut buf = String::new();
    /// ring.set_attr_display(&mut buf, "stroke-width", 2.5)?; // no per-call String allocation
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_attr_display<T: std::fmt::Display>(
        &self,
        scratch: &mut String,
        name: &str,
        value: T,
    ) -> Result<(), Error> {
        use std::fmt::Write;
        scratch.clear();
        write!(scratch, "{value}")?;
        self.set_attr(name, scratch)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Binds a reusable [`SvgAttrs`] buffer to this node and returns a chainable attribute writer.
    ///
    /// Use this when setting several numeric or formatted attributes as it avoids the need to allocate a new `String`
    /// for each attribute value.
    pub fn attrs<'a>(&'a self, attrs: &'a mut SvgAttrs) -> AttrWriter<'a> {
        attrs.writer(self)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Multi-attribute setter
    ///
    /// Sets several attributes on this element in sequence.
    ///
    /// This is a convenience wrapper around repeated [`set_attr`](Self::set_attr) calls. It is useful when creating or
    /// updating an element whose geometry or style is described by several attributes at once. The setter accepts both
    /// borrowed and owned strings, so it works with literal values as well as values produced by `to_string()`.
    ///
    /// If the browser rejects one of the attributes, this returns the first DOM error and stops. Attributes already set
    /// before that error are left in place, matching the behaviour you would get from issuing the same `set_attr` calls
    /// manually.
    ///
    /// The same security caveat as [`set_attr`](Self::set_attr) applies: names and values are written verbatim, so do
    /// not pass untrusted input.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(80.0, 40.0))?;
    ///
    /// rect.set_attrs([
    ///     ("fill", "steelblue"),
    ///     ("stroke", "white"),
    ///     ("stroke-width", "2"),
    ///     ("rx", "6"),
    /// ])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_attrs<I, K, V>(&self, attrs: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (name, value) in attrs {
            self.set_attr(name.as_ref(), value.as_ref())?;
        }
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Read element attribute value
    ///
    /// Returns `None` if the attribute is not present.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(100.0, 50.0))?;
    /// rect.set_attr("class", "highlighted")?;
    ///
    /// assert_eq!(rect.attr("class").as_deref(), Some("highlighted"));
    /// assert_eq!(rect.attr("nonexistent"), None);
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn attr(&self, name: &str) -> Option<String> {
        self.inner.element.get_attribute(name)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Remove element attribute
    ///
    /// Has no effect if the attribute is not present.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(100.0, 50.0))?;
    /// rect.set_attr("opacity", "0.5")?;
    /// rect.remove_attr("opacity")?;         // element is fully opaque again
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn remove_attr(&self, name: &str) -> Result<(), Error> {
        self.inner.element.remove_attribute(name).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Add a CSS class
    ///
    /// Adds `class` to this element's `class` attribute via the DOM `classList` API.
    /// This has no effect if the class is already present.
    ///
    /// # Errors
    ///
    /// Per the DOM Standard, the underlying `classList.add` call throws, and this returns [`Error::Dom`], if `class`
    /// is:
    ///
    /// * empty (`""`)
    /// * contains ASCII whitespace (for example `"two classes"`)
    ///
    /// The `class` attribute is left unchanged when either error occurs.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(100.0, 50.0))?;
    ///
    /// rect.add_class("highlighted")?;
    /// assert!(rect.has_class("highlighted"));
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn add_class(&self, class: &str) -> Result<(), Error> {
        self.inner.element.class_list().add_1(class).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Remove a CSS class
    ///
    /// Removes `class` from this element's `class` attribute via the DOM `classList` API.
    /// This has no effect if the class is not present.
    ///
    /// # Errors
    ///
    /// Per the DOM Standard, the underlying `classList.remove` call throws, and this returns [`Error::Dom`], if
    /// `class` is:
    ///
    /// * empty (`""`)
    /// * contains ASCII whitespace (for example `"two classes"`)
    ///
    /// The `class` attribute is left unchanged when either error occurs.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(100.0, 50.0))?;
    ///
    /// rect.add_class("highlighted")?;
    /// rect.remove_class("highlighted")?;
    /// assert!(!rect.has_class("highlighted"));
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn remove_class(&self, class: &str) -> Result<(), Error> {
        self.inner.element.class_list().remove_1(class).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Toggle a CSS class
    ///
    /// Adds `class` if it is absent, removes it if it is present, and returns the class's new membership state:
    /// `true` if it is now present, `false` if it is now absent.
    ///
    /// # Errors
    ///
    /// Per the DOM Standard, the underlying `classList.toggle` call throws, and this returns [`Error::Dom`], if
    /// `class` is:
    ///
    /// * empty (`""`)
    /// * contains ASCII whitespace (for example `"two classes"`)
    ///
    /// The `class` attribute is left unchanged when either error occurs.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(100.0, 50.0))?;
    ///
    /// assert_eq!(rect.toggle_class("selected")?, true);  // absent -> added
    /// assert_eq!(rect.toggle_class("selected")?, false); // present -> removed
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn toggle_class(&self, class: &str) -> Result<bool, Error> {
        self.inner.element.class_list().toggle(class).map_err(dom_err)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Set a CSS class to a known state
    ///
    /// Adds `class` if `enabled` is `true`, removes it if `enabled` is `false`, via the DOM `classList.toggle(token,
    /// force)` overload. Unlike [`toggle_class`](Self::toggle_class), the caller supplies the desired end state
    /// directly, so the result never depends on whatever state happened to be present beforehand.
    ///
    /// Prefer this over a `has_class` check followed by `add_class`/`remove_class` — it is both simpler at the call
    /// site and uses a single DOM call instead of two.
    ///
    /// # Errors
    ///
    /// As per the DOM Standard, the underlying `classList.toggle` call throws, and this returns [`Error::Dom`], if
    /// `class` is:
    ///
    /// * empty (`""`)
    /// * contains ASCII whitespace (for example `"two classes"`)
    ///
    /// The `class` attribute is left unchanged when either error occurs.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(100.0, 50.0))?;
    /// let is_selected = true;
    ///
    /// rect.set_class_enabled("selected", is_selected)?;
    /// assert_eq!(rect.has_class("selected"), is_selected);
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_class_enabled(&self, class: &str, enabled: bool) -> Result<(), Error> {
        self.inner
            .element
            .class_list()
            .toggle_with_force(class, enabled)
            .map_err(dom_err)?;
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// # Test for a CSS class
    ///
    /// Returns `true` if `class` is present in this element's `class` attribute.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::origin(), Size::new(100.0, 50.0))?;
    ///
    /// assert!(!rect.has_class("highlighted"));
    /// rect.add_class("highlighted")?;
    /// assert!(rect.has_class("highlighted"));
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn has_class(&self, class: &str) -> bool {
        self.inner.element.class_list().contains(class)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `fill` attribute to a CSS colour value.
    ///
    /// Accepts any valid SVG paint value:
    ///
    /// * named colours (`"red"`)
    /// * hex codes (`"#ff0000"`)
    /// * `rgb()`/`hsl()` functions
    /// * `"none"` to make the fill transparent
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(80.0, 40.0))?;
    /// rect.set_fill("steelblue")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_fill(&self, colour: &str) -> Result<(), Error> {
        self.set_attr("fill", colour)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `stroke` attribute to a CSS colour value.
    ///
    /// Use in combination with [`set_stroke_width`](Self::set_stroke_width) to control the appearance of outlines and lines.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let rect = svg.rect(Point::new(0.0, 0.0), Size::new(80.0, 40.0))?;
    /// rect.set_stroke("black")?;
    /// rect.set_stroke_width(1.5)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_stroke(&self, colour: &str) -> Result<(), Error> {
        self.set_attr("stroke", colour)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `stroke-width` attribute in user units.
    ///
    /// This convenience setter formats `width` into a short-lived `String` that is allocated and dropped on each call —
    /// fine for one-off styling. If you animate the stroke width on a hot path (a pulsing highlight, a hover/drag
    /// emphasis), prefer [`set_attr_display`](Self::set_attr_display) with a reused buffer, or an
    /// [`AttrWriter`]/[`SvgAttrs`], to avoid that per-call allocation.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let line = svg.line(Point::new(0.0, 50.0), Point::new(200.0, 50.0))?;
    /// line.set_stroke("grey")?;
    /// line.set_stroke_width(3.0)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_stroke_width(&self, width: f64) -> Result<(), Error> {
        let mut attrs = SvgAttrs::new();
        attrs.display(self, "stroke-width", width)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Formats and writes `attr="url(#id)"` from a bare `&str` id, the shared shape behind every validated
    /// paint-server / marker / clip-path / filter reference setter (`set_marker_start`, `set_fill_gradient`, ...).
    ///
    /// Does not validate `id` — every caller below (`set_marker_start`, `set_fill_gradient`, ...) calls
    /// [`validate_marker_id`] or its siblings first, then delegates here.
    ///
    /// Every `_ref` variant instead writes its handle's own cached `url(#id)` reference directly via
    /// [`set_attr`](Self::set_attr) (`SvgMarker::url_ref`, `SvgLinearGradient::url_ref`, ...), bypassing this
    /// method entirely: that string is built once at the handle's construction time (and rebuilt in place by its
    /// `set_id`), so there is no bare id left to format here and no per-call allocation, however many elements the
    /// same handle is applied to.
    fn set_url_ref(&self, attr: &str, id: &str) -> Result<(), Error> {
        self.set_attr(attr, &format!("{URL_PREFIX}{id})"))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `marker-start` attribute, painting the given marker at the first vertex of the element's stroke.
    ///
    /// `marker_id` is the bare `id` of an [`SvgMarker`] defined in a [`SvgDefs`](crate::SvgDefs) block;
    /// the `url(#...)` wrapper is added automatically.
    /// The same validation rules that apply at marker construction time are enforced here: an id that does not match
    /// `[A-Za-z_][A-Za-z0-9_-]*` returns [`Error::InvalidMarkerId`].
    ///
    /// Prefer [`set_marker_start_ref`](Self::set_marker_start_ref) when you have the [`SvgMarker`] handle available,
    /// as it eliminates the risk of typos and `url(#...)` double-wrapping.
    pub fn set_marker_start(&self, marker_id: &str) -> Result<(), Error> {
        validate_marker_id(marker_id)?;
        self.set_url_ref("marker-start", marker_id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `marker-mid` attribute, painting the given marker at every intermediate vertex of the element's stroke.
    ///
    /// `marker_id` is the bare `id` of an [`SvgMarker`] defined in a [`SvgDefs`](crate::SvgDefs) block;
    /// the `url(#...)` wrapper is added automatically.
    /// The same validation rules that apply at marker construction time are enforced here: an id that does not match
    /// `[A-Za-z_][A-Za-z0-9_-]*` returns [`Error::InvalidMarkerId`].
    ///
    /// Prefer [`set_marker_mid_ref`](Self::set_marker_mid_ref) when you have the [`SvgMarker`] handle available,
    /// as it eliminates the risk of typos and `url(#...)` double-wrapping.
    pub fn set_marker_mid(&self, marker_id: &str) -> Result<(), Error> {
        validate_marker_id(marker_id)?;
        self.set_url_ref("marker-mid", marker_id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `marker-end` attribute, painting the given marker at the last vertex of the element's stroke.
    ///
    /// `marker_id` is the bare `id` of an [`SvgMarker`] defined in a [`SvgDefs`](crate::SvgDefs) block;
    /// the `url(#...)` wrapper is added automatically.
    /// The same validation rules that apply at marker construction time are enforced here: an id that does not match
    /// `[A-Za-z_][A-Za-z0-9_-]*` returns [`Error::InvalidMarkerId`].
    /// Prefer [`set_marker_end_ref`](Self::set_marker_end_ref) when you have the [`SvgMarker`] handle available, as it
    /// eliminates the risk of typos and `url(#...)` double-wrapping.
    pub fn set_marker_end(&self, marker_id: &str) -> Result<(), Error> {
        validate_marker_id(marker_id)?;
        self.set_url_ref("marker-end", marker_id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `marker-start` attribute from a live [`SvgMarker`] handle.
    ///
    /// This is the preferred alternative to [`set_marker_start`](Self::set_marker_start): the id is taken directly from
    /// the marker, so there is no risk of typos or `url(#...)` double-wrapping.
    ///
    /// The written attribute stores the marker's id as a string at call time.
    /// If the marker is later renamed with [`SvgMarker::set_id`](crate::SvgMarker::set_id), this element's attribute is
    /// not updated automatically — reapply the reference after renaming if needed.
    pub fn set_marker_start_ref(&self, marker: &SvgMarker) -> Result<(), Error> {
        self.set_attr("marker-start", marker.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `marker-mid` attribute from a live [`SvgMarker`] handle.
    ///
    /// This is the preferred alternative to [`set_marker_mid`](Self::set_marker_mid): the id is taken directly from
    /// the marker, so there is no risk of typos or `url(#...)` double-wrapping.
    ///
    /// The written attribute stores the marker's id as a string at call time.
    /// If the marker is later renamed with [`SvgMarker::set_id`](crate::SvgMarker::set_id), this element's attribute is
    /// not updated automatically — reapply the reference after renaming if needed.
    pub fn set_marker_mid_ref(&self, marker: &SvgMarker) -> Result<(), Error> {
        self.set_attr("marker-mid", marker.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `marker-end` attribute from a live [`SvgMarker`] handle.
    ///
    /// This is the preferred alternative to [`set_marker_end`](Self::set_marker_end): the id is taken directly from
    /// the marker, so there is no risk of typos or `url(#...)` double-wrapping.
    ///
    /// The written attribute stores the marker's id as a string at call time.
    /// If the marker is later renamed with [`SvgMarker::set_id`](crate::SvgMarker::set_id), this element's attribute is
    /// not updated automatically — reapply the reference after renaming if needed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let marker = defs.marker("arrow")?;
    /// marker.set_orient("auto")?;
    /// marker.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
    ///
    /// let line = svg.line(Point::new(20.0, 50.0), Point::new(180.0, 50.0))?;
    /// line.set_marker_end_ref(&marker)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_marker_end_ref(&self, marker: &SvgMarker) -> Result<(), Error> {
        self.set_attr("marker-end", marker.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `d` (path data) attribute on a `<path>` element.
    ///
    /// Alters an existing path created by [`SvgRoot::path`](crate::SvgRoot::path) without needing to recreate the DOM
    /// element.
    ///
    /// The `d` string uses standard SVG path commands where the arguments to the uppercase command supply absolute
    /// coordinates, and the arguments to the lowercase commands supply relative coordinates.
    ///
    /// | Command   | Arguments              | Description             |
    /// |:----------|:-----------------------|:------------------------|
    /// | `M` / `m` | `x y`                  | Move (no draw)          |
    /// | `L` / `l` | `x y`                  | Line                    |
    /// | `H` / `h` | `x`                    | Horizontal line         |
    /// | `V` / `v` | `y`                    | Vertical line           |
    /// | `C` / `c` | `x1 y1 x2 y2 x y`      | Cubic Bézier            |
    /// | `S` / `s` | `x2 y2 x y`            | Smooth cubic Bézier     |
    /// | `Q` / `q` | `x1 y1 x y`            | Quadratic Bézier        |
    /// | `T` / `t` | `x y`                  | Smooth quadratic Bézier |
    /// | `A` / `a` | `rx ry rot laf sf x y` | Elliptical arc          |
    /// | `Z` / `z` | —                      | Close path              |
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{root::utils::{Point, Size}, SvgRoot};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let path = svg.path("M 0 0 L 100 100")?;
    ///
    /// // Later: morph the path without touching any other attributes.
    /// path.set_d("M 0 0 Q 50 0 100 100")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_d(&self, path: &str) -> Result<(), Error> {
        self.set_attr("d", path)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `d` (path data) attribute on a `<path>` element from a sequence of typed [`PathDef`] segments.
    ///
    /// The type-safe alternative to [`set_d`](Self::set_d): the `d` string is built internally by [`build_d`] from
    /// well-formed commands, and the sequence is checked to ensure it begins with a `MoveTo` — see [`PathDef`]'s own
    /// documentation for exactly what is and is not guaranteed.
    ///
    /// This convenience setter formats through a fresh, short-lived `String` allocated and dropped on each call
    /// (fine for an occasional update); however, if you need to morph a path on the hot path (e.g. a `pointermove`
    /// handler, or on every animation frame), then prefer [`SvgAttrs::d_from_defs`](crate::SvgAttrs::d_from_defs) /
    /// [`AttrWriter::d_from_defs`](crate::AttrWriter::d_from_defs) or
    /// [`AnimationFrame::set_d_from_defs`](crate::AnimationFrame::set_d_from_defs), which reuse a caller-owned
    /// buffer instead.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidPathData`] — `defs` is non-empty and its first command is not a `MoveTo`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{PathDef, PathDefAbsolute, SvgRoot, root::utils::Point};
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let path = svg.path("M 0 0 L 100 100")?;
    ///
    /// // Later: morph the path without touching any other attributes.
    /// path.set_d_from_defs(&[
    ///     PathDef::Abs(PathDefAbsolute::MoveTo(Point::new(0.0, 0.0))),
    ///     PathDef::Abs(PathDefAbsolute::QuadraticBezierTo(Point::new(50.0, 0.0), Point::new(100.0, 100.0))),
    /// ])?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_d_from_defs(&self, defs: &[PathDef]) -> Result<(), Error> {
        validate_starts_with_moveto(defs)?;
        self.set_d(&build_d(defs))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `fill` attribute to reference a gradient by its bare `id`.
    ///
    /// The `url(#...)` wrapper is added automatically.
    ///
    /// The same validation rules that apply at gradient construction time are also enforced here:
    /// an id that does not match the pattern `[A-Za-z_][A-Za-z0-9_-]*` will return [`Error::InvalidGradientId`].
    ///
    /// Prefer [`set_fill_linear_gradient`](Self::set_fill_linear_gradient) or
    /// [`set_fill_radial_gradient`](Self::set_fill_radial_gradient) when you have the gradient handle available, as
    /// they eliminate the risk of typos and `url(#...)` double-wrapping.
    pub fn set_fill_gradient(&self, gradient_id: &str) -> Result<(), Error> {
        validate_gradient_id(gradient_id)?;
        self.set_url_ref("fill", gradient_id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `stroke` attribute to reference a gradient by its bare `id`.
    ///
    /// The `url(#...)` wrapper is added automatically.
    ///
    /// The same validation rules that apply at gradient construction time are also enforced here:
    /// an id that does not match the pattern `[A-Za-z_][A-Za-z0-9_-]*` will return [`Error::InvalidGradientId`].
    ///
    /// Prefer [`set_stroke_linear_gradient`](Self::set_stroke_linear_gradient) or
    /// [`set_stroke_radial_gradient`](Self::set_stroke_radial_gradient) when you have the gradient handle available, as
    /// they eliminate the risk of typos and `url(#...)` double-wrapping.
    pub fn set_stroke_gradient(&self, gradient_id: &str) -> Result<(), Error> {
        validate_gradient_id(gradient_id)?;
        self.set_url_ref("stroke", gradient_id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `fill` attribute to reference a [`SvgLinearGradient`] by its `id`.
    ///
    /// This is the preferred alternative to [`set_fill_gradient`](Self::set_fill_gradient): the id is taken directly
    /// from the gradient handle, so there is no risk of typos or `url(#...)` double-wrapping.
    ///
    /// The written attribute stores the gradient's id as a string at call time. If the gradient is later renamed with
    /// [`SvgLinearGradient::set_id`], this element's attribute is not updated automatically — reapply the reference
    /// after renaming if needed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let grad = defs.build_linear_gradient("banner", |g| {
    ///     g.add_stop(0.0, "steelblue")?;
    ///     g.add_stop(1.0, "coral")?;
    ///     Ok(())
    /// })?;
    ///
    /// let rect = svg.rect(Point::origin(), Size::new(200.0, 80.0))?;
    /// rect.set_fill_linear_gradient(&grad)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_fill_linear_gradient(&self, gradient: &SvgLinearGradient) -> Result<(), Error> {
        self.set_attr("fill", gradient.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `stroke` attribute to reference a [`SvgLinearGradient`] by its `id`.
    ///
    /// This is the preferred alternative to [`set_stroke_gradient`](Self::set_stroke_gradient) when you have
    /// the gradient handle.
    pub fn set_stroke_linear_gradient(&self, gradient: &SvgLinearGradient) -> Result<(), Error> {
        self.set_attr("stroke", gradient.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `fill` attribute to reference a [`SvgRadialGradient`] by its `id`.
    ///
    /// This is the preferred alternative to [`set_fill_gradient`](Self::set_fill_gradient) when you have
    /// the gradient handle.
    pub fn set_fill_radial_gradient(&self, gradient: &SvgRadialGradient) -> Result<(), Error> {
        self.set_attr("fill", gradient.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `stroke` attribute to reference a [`SvgRadialGradient`] by its `id`.
    ///
    /// This is the preferred alternative to [`set_stroke_gradient`](Self::set_stroke_gradient) when you have
    /// the gradient handle.
    pub fn set_stroke_radial_gradient(&self, gradient: &SvgRadialGradient) -> Result<(), Error> {
        self.set_attr("stroke", gradient.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `fill` attribute to reference a pattern by its bare `id`.
    ///
    /// The `url(#...)` wrapper is added automatically.
    ///
    /// The same validation rules that apply at pattern construction time are also enforced here: an id that does not
    /// match the pattern `[A-Za-z_][A-Za-z0-9_-]*` will return [`Error::InvalidPatternId`].
    ///
    /// Prefer [`set_fill_pattern_ref`](Self::set_fill_pattern_ref) when you have the pattern handle available, as it
    /// eliminates the risk of typos and `url(#...)` double-wrapping.
    pub fn set_fill_pattern(&self, pattern_id: &str) -> Result<(), Error> {
        validate_pattern_id(pattern_id)?;
        self.set_url_ref("fill", pattern_id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `stroke` attribute to reference a pattern by its bare `id`.
    ///
    /// The `url(#...)` wrapper is added automatically.
    ///
    /// The same validation rules that apply at pattern construction time are also enforced here: an id that does not
    /// match the pattern `[A-Za-z_][A-Za-z0-9_-]*` will return [`Error::InvalidPatternId`].
    ///
    /// Prefer [`set_stroke_pattern_ref`](Self::set_stroke_pattern_ref) when you have the pattern handle available.
    pub fn set_stroke_pattern(&self, pattern_id: &str) -> Result<(), Error> {
        validate_pattern_id(pattern_id)?;
        self.set_url_ref("stroke", pattern_id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `fill` attribute to reference a [`SvgPattern`] by its `id`.
    ///
    /// This is the preferred alternative to [`set_fill_pattern`](Self::set_fill_pattern): the id is taken directly from
    /// the pattern handle, so there is no risk of typos or `url(#...)` double-wrapping.
    pub fn set_fill_pattern_ref(&self, pattern: &SvgPattern) -> Result<(), Error> {
        self.set_attr("fill", pattern.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `stroke` attribute to reference a [`SvgPattern`] by its `id`.
    ///
    /// This is the preferred alternative to [`set_stroke_pattern`](Self::set_stroke_pattern): the id is taken directly
    /// from the pattern handle, so there is no risk of typos or `url(#...)` double-wrapping.
    pub fn set_stroke_pattern_ref(&self, pattern: &SvgPattern) -> Result<(), Error> {
        self.set_attr("stroke", pattern.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `clip-path` attribute by bare clip-path `id`, restricting the rendered region of this element to the
    /// shapes defined inside the named [`SvgClipPath`].
    ///
    /// `clip_path_id` is the bare `id` of a [`SvgClipPath`] defined in a [`SvgDefs`](crate::SvgDefs) block;
    /// the `url(#...)` wrapper is added automatically.
    /// The same validation rules that apply at clip-path construction time are enforced here: an id that does not match
    /// `[A-Za-z_][A-Za-z0-9_-]*` returns [`Error::InvalidClipPathId`].
    ///
    /// Prefer [`set_clip_path_ref`](Self::set_clip_path_ref) when you have the [`SvgClipPath`] handle available,
    /// as it eliminates the risk of typos and `url(#...)` double-wrapping.
    ///
    /// To remove the clip, call [`remove_clip_path`](Self::remove_clip_path).
    pub fn set_clip_path(&self, clip_path_id: &str) -> Result<(), Error> {
        validate_clip_path_id(clip_path_id)?;
        self.set_url_ref("clip-path", clip_path_id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `clip-path` attribute from a live [`SvgClipPath`] handle.
    ///
    /// This is the preferred alternative to [`set_clip_path`](Self::set_clip_path): the id is taken directly from the
    /// handle, so there is no risk of typos or `url(#...)` double-wrapping.
    ///
    /// The written attribute stores the clip path's id as a string at call time.
    /// If the clip path is later renamed with [`SvgClipPath::set_id`](crate::SvgClipPath::set_id), this element's
    /// attribute is not updated automatically — reapply the reference after renaming if needed.
    ///
    /// To remove the clip, call [`remove_clip_path`](Self::remove_clip_path).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    ///
    /// let clip = defs.build_clip_path("viewport", |c| {
    ///     c.circle(Point::new(60.0, 60.0), 55.0)?;
    ///     Ok(())
    /// })?;
    ///
    /// let bg = svg.rect(Point::origin(), Size::new(120.0, 120.0))?;
    /// bg.set_fill("steelblue")?;
    /// bg.set_clip_path_ref(&clip)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_clip_path_ref(&self, clip: &SvgClipPath) -> Result<(), Error> {
        self.set_attr("clip-path", clip.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes the `clip-path` attribute from this element, making the full element visible again.
    ///
    /// Has no effect if no clip path is currently applied.
    pub fn remove_clip_path(&self) -> Result<(), Error> {
        self.remove_attr("clip-path")
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `mask` attribute by bare mask `id`, revealing or hiding parts of this element according to the
    /// luminance or alpha of the named [`SvgMask`]'s rendered content.
    ///
    /// `mask_id` is the bare `id` of a [`SvgMask`] defined in a [`SvgDefs`](crate::SvgDefs) block; the `url(#...)`
    /// wrapper is added automatically.
    ///
    /// The same validation rules that apply at mask construction time are enforced here: an id that does not match
    /// `[A-Za-z_][A-Za-z0-9_-]*` returns [`Error::InvalidMaskId`].
    ///
    /// Prefer [`set_mask_ref`](Self::set_mask_ref) when you have the [`SvgMask`] handle available, as it eliminates
    /// the risk of typos and `url(#...)` double-wrapping.
    ///
    /// To remove the mask, call [`remove_mask`](Self::remove_mask).
    pub fn set_mask(&self, mask_id: &str) -> Result<(), Error> {
        validate_mask_id(mask_id)?;
        self.set_url_ref("mask", mask_id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `mask` attribute from a live [`SvgMask`] handle.
    ///
    /// This is the preferred alternative to [`set_mask`](Self::set_mask): the id is taken directly from the handle, so
    /// there is no risk of typos or `url(#...)` double-wrapping.
    ///
    /// The written attribute stores the mask's id as a string at call time.
    /// If the mask is later renamed with [`SvgMask::set_id`](crate::SvgMask::set_id), this element's attribute is not
    /// updated automatically — reapply the reference after renaming if needed.
    ///
    /// To remove the mask, call [`remove_mask`](Self::remove_mask).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    ///
    /// let fade = defs.build_mask("fade", |m| {
    ///     m.rect(Point::origin(), Size::new(120.0, 120.0))?.set_fill_gradient("fade-gradient")?;
    ///     Ok(())
    /// })?;
    ///
    /// let bg = svg.rect(Point::origin(), Size::new(120.0, 120.0))?;
    /// bg.set_fill("steelblue")?;
    /// bg.set_mask_ref(&fade)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_mask_ref(&self, mask: &SvgMask) -> Result<(), Error> {
        self.set_attr("mask", mask.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes the `mask` attribute from this element, making the full element visible again.
    ///
    /// Has no effect if no mask is currently applied.
    pub fn remove_mask(&self) -> Result<(), Error> {
        self.remove_attr("mask")
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `filter` attribute identified by filter `id`, applying the raster effects defined inside the named
    /// [`SvgFilter`] to this element.
    ///
    /// `filter_id` is the bare `id` of an [`SvgFilter`] defined in a [`SvgDefs`](crate::SvgDefs) block;
    /// the `url(#...)` wrapper is added automatically.
    /// The same validation rules that apply at filter construction time are enforced here: an id that does not match
    /// the pattern `[A-Za-z_][A-Za-z0-9_-]*` returns [`Error::InvalidFilterId`].
    ///
    /// Prefer [`set_filter_ref`](Self::set_filter_ref) when you have the [`SvgFilter`] handle available, as it
    /// eliminates the risk of typos and `url(#...)` double-wrapping.
    ///
    /// To remove the filter, call [`remove_filter`](Self::remove_filter).
    pub fn set_filter(&self, filter_id: &str) -> Result<(), Error> {
        validate_filter_id(filter_id)?;
        self.set_url_ref("filter", filter_id)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `filter` attribute from a live [`SvgFilter`] handle.
    ///
    /// This is the preferred alternative to [`set_filter`](Self::set_filter): the id is taken directly from the handle,
    /// so there is no risk of typos or `url(#...)` double-wrapping.
    ///
    /// The written attribute stores the filter's id as a string at call time. If the filter is later renamed using
    /// [`SvgFilter::set_id`](crate::SvgFilter::set_id), this element's attribute is not updated automatically — you
    /// must reapply the reference after renaming if needed.
    ///
    /// To remove the filter, call [`remove_filter`](Self::remove_filter).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    ///
    /// let blur = defs.build_filter("soft-blur", |f| {
    ///     f.gaussian_blur(4.0)?;
    ///     Ok(())
    /// })?;
    ///
    /// let rect = svg.rect(Point::origin(), Size::new(120.0, 80.0))?;
    /// rect.set_fill("steelblue")?;
    /// rect.set_filter_ref(&blur)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_filter_ref(&self, filter: &SvgFilter) -> Result<(), Error> {
        self.set_attr("filter", filter.url_ref())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Removes the `filter` attribute from this element, making its normal (unfiltered) rendering visible again.
    ///
    /// Has no effect if no filter is currently applied.
    pub fn remove_filter(&self) -> Result<(), Error> {
        self.remove_attr("filter")
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Sets the `href` attribute, which `<use>` elements use to reference a reusable asset.
    ///
    /// Pass a local fragment reference such as `"#my-shape"` (where `"my-shape"` is the `id` of the target element)
    /// or a full URL for cross-document references.
    /// Use this to redirect a `<use>` node to a different asset after it was created with
    /// [`SvgRoot::use_node`](crate::SvgRoot::use_node).
    ///
    /// # Security
    ///
    /// The value is written verbatim via `setAttribute`. Do not pass a `javascript:` URL or any other
    /// attacker-controlled value without validation.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::utils::Point};
    /// let svg = SvgRoot::attach("diagram")?;
    /// let u = svg.use_node("#dot", Point::new(50.0, 60.0))?;
    /// u.set_href("#diamond")?; // redirect to a different asset
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn set_href(&self, href: &str) -> Result<(), Error> {
        self.set_attr("href", href)
    }
}
