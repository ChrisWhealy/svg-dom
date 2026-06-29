use crate::{
    SvgRoot,
    error::Error,
    node::SvgNode,
    root::{
        factory::SvgFactory,
        utils::{Point, Size},
    },
};

impl SvgRoot {
    /// Creates an `<image>` element, appends it to the root, and returns its [`SvgNode`] handle.
    ///
    /// `<image>` embeds a raster image (PNG, JPEG, WebP, etc...) or another SVG into the current SVG document.
    /// The image is fetched from `href` and rendered inside the rectangle defined by `top_left` and `size`.
    ///
    /// # Arguments
    ///
    /// * `href` — URL of the image to embed.
    ///   Accepts any URL that the browser can fetch: a relative path (`"images/photo.png"`), an absolute URL, or a
    ///   `data:` URI for self-contained content.
    ///   When using `data:image/svg+xml;base64 ...`, base64-encoding the SVG avoids having to percent-encode `<`, `>`,
    ///   and `#` that would otherwise break the data URI parser.
    /// * `top_left` — position of the top-left corner of the image rectangle, in user units.
    /// * `size` — width and height of the image rectangle, in user units.
    ///   Both must be set; omitting either makes the image invisible.
    ///
    /// # Aspect-ratio control
    ///
    /// By default, SVG preserves the image's natural aspect ratio and centres it inside the rectangle
    /// (`preserveAspectRatio="xMidYMid meet"`).
    /// Override this with [`set_attr`](crate::SvgNode::set_attr):
    ///
    /// | Value | Effect |
    /// |---|---|
    /// | `"xMidYMid meet"` | Fit whole image inside the box, adding blank bars if needed (default) |
    /// | `"none"` | Stretch to fill the box exactly, ignoring the source aspect ratio |
    /// | `"xMidYMid slice"` | Scale up to fill the box and clip any overflow |
    ///
    /// # Updating after creation
    ///
    /// To swap the image source after creation, call [`SvgNode::set_href`](crate::SvgNode::set_href).
    /// To reposition or resize, use [`set_attr`](crate::SvgNode::set_attr) with `"x"`, `"y"`, `"width"`, or
    /// `"height"`.
    ///
    /// # Security
    ///
    /// The `href` value is written verbatim to the DOM via `setAttribute`.
    /// Do not pass a `javascript:` URL or any other attacker-controlled string without validation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the element.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use svg_dom::{SvgRoot, root::utils::{Point, Size}};
    /// let svg = SvgRoot::attach("diagram")?;
    ///
    /// // Embed a PNG at (10, 10) in a 120×80 box.
    /// let img = svg.image("images/photo.png", Point::new(10.0, 10.0), Size::new(120.0, 80.0))?;
    ///
    /// // Stretch to fill the box, ignoring the source aspect ratio.
    /// img.set_attr("preserveAspectRatio", "none")?;
    ///
    /// // Later: swap to a different image without recreating the element.
    /// img.set_href("images/other.png")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn image(&self, href: &str, top_left: Point, size: Size) -> Result<SvgNode, Error> {
        self.create_image(href, top_left, size)
    }
}
