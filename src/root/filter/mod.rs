use crate::{
    Error, dom_err,
    root::{attrs::SvgAttrs, defs::URL_PREFIX},
};
use std::cell::RefCell;
use web_sys::{Document, SvgElement};

mod blend_mode;
mod channel;
mod color_matrix_type;
mod composite_operator;
mod filter_units;
mod transfer_function;
mod turbulence_type;

pub use blend_mode::BlendMode;
pub use channel::Channel;
pub use color_matrix_type::ColorMatrixType;
pub use composite_operator::CompositeOperator;
pub use filter_units::FilterUnits;
pub use transfer_function::TransferFunction;
pub use turbulence_type::TurbulenceType;

mod attrs;
mod primitives;
mod region;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// A `<filter>` element that applies raster effects (blur, colour manipulation, compositing, ...) to any element that
/// references it.
///
/// A `<filter>` is a container of one or more filter-primitive elements (`<feGaussianBlur>`, `<feOffset>`, etc.); the
/// browser evaluates them in document order and paints the final result in place of the referencing element.
///
/// Obtain one from [`SvgDefs::filter`](crate::SvgDefs::filter) or
/// [`SvgDefs::build_filter`](crate::SvgDefs::build_filter), and apply it to any element with
/// [`SvgNode::set_filter_ref`](crate::SvgNode::set_filter_ref) or [`SvgNode::set_filter`](crate::SvgNode::set_filter).
///
/// # Primitive coverage
///
/// Implemented filter effects:
///
/// - [`gaussian_blur`](Self::gaussian_blur)
/// - [`gaussian_blur_xy`](Self::gaussian_blur_xy) (`<feGaussianBlur>`),
/// - [`offset`](Self::offset) (`<feOffset>`),
/// - [`merge`](Self::merge) (`<feMerge>`/`<feMergeNode>`),
/// - [`flood`](Self::flood) (`<feFlood>`),
/// - [`composite`](Self::composite) (`<feComposite>`),
/// - [`blend`](Self::blend) (`<feBlend>`),
/// - [`drop_shadow`](Self::drop_shadow) (`<feDropShadow>`),
/// - [`color_matrix`](Self::color_matrix) (`<feColorMatrix>`)
/// - [`component_transfer`](Self::component_transfer) (`<feComponentTransfer>` with `<feFuncR>`/`<feFuncG>`/
///   `<feFuncB>`/`<feFuncA>`)
/// - [`turbulence`](Self::turbulence)
/// - [`turbulence_xy`](Self::turbulence_xy) (`<feTurbulence>`)
/// - [`displacement_map`](Self::displacement_map) (`<feDisplacementMap>`)
///
/// The first five, taken together, can be used to build a *true* tinted, opacity-controlled drop shadow (blur the
/// source alpha, composite a flood colour into the blurred mask, offset it, then merge it underneath the original
/// graphic; see [`composite`](Self::composite)'s example) rather than just a blurred copy of the source graphic's
/// own colour.
///
/// [`drop_shadow`](Self::drop_shadow) achieves the same effect using a single primitive, since the SVG specification
/// defines it as a browser-native shorthand for exactly that chain.
///
/// [`color_matrix`](Self::color_matrix) is independent of the shadow primitives — greyscale, saturation, hue
/// rotation, or an arbitrary linear colour transform via [`ColorMatrixType`].
///
/// [`blend`](Self::blend) is also independent of the shadow primitives: unlike [`composite`](Self::composite)'s
/// geometric (Porter-Duff) combination, it mixes two inputs' *colours* using a [`BlendMode`] — the standard
/// `<blend-mode>` keywords CSS `mix-blend-mode` also uses (see [`BlendMode`]'s own doc comment for two ways this
/// is not quite the same thing as CSS `mix-blend-mode` itself).
///
/// [`component_transfer`](Self::component_transfer) is likewise independent of the shadow primitives: a per-channel
/// remap ([`TransferFunction`]) applied to red/green/blue/alpha individually, the standard way to do gamma
/// correction, contrast/levels adjustment, posterisation ([`TransferFunction::Discrete`]), or an alpha fade/clip,
/// none of which [`color_matrix`](Self::color_matrix)'s whole-pixel linear transform can express.
///
/// [`turbulence`](Self::turbulence)/[`turbulence_xy`](Self::turbulence_xy) generate Perlin noise from nothing — the
/// only primitive here with no meaningful `in` — and [`displacement_map`](Self::displacement_map) uses another
/// input's channel values (typically that noise) to warp a second input pixel-by-pixel. Paired together they are
/// the standard route to hand-drawn/organic textures; see [`displacement_map`](Self::displacement_map)'s own doc
/// comment for a worked example.
///
/// The SVG filter specification defines around fifteen effect primitives in total (`feTile`, `feMorphology`, and
/// others), each with its own attribute grammar. See `docs/gaps.md` for the primitives still to be added.
///
/// The filter region ([`set_x`](Self::set_x), [`set_y`](Self::set_y), [`set_width`](Self::set_width),
/// [`set_height`](Self::set_height)) and coordinate-space ([`set_filter_units`](Self::set_filter_units),
/// [`set_primitive_units`](Self::set_primitive_units)) attributes each have a named setter.
/// [`SvgNode::set_attr`](crate::SvgNode::set_attr) on any node returned by a primitive method covers that primitive's
/// own attributes not yet wrapped by a named parameter (`in`, `result`, and so on).
///
/// # Example
///
/// ```rust,no_run
/// use svg_dom::{SvgRoot, root::utils::{Point, Size}};
///
/// let svg  = SvgRoot::attach("diagram")?;
/// let defs = svg.defs()?;
///
/// let blur = defs.filter("soft-blur")?;
/// blur.gaussian_blur(4.0)?;
///
/// let rect = svg.rect(Point::origin(), Size::new(120.0, 80.0))?;
/// rect.set_fill("steelblue")?;
/// rect.set_filter_ref(&blur)?;
/// Ok::<(), svg_dom::Error>(())
/// ```
pub struct SvgFilter {
    /// The complete `url(#id)` reference, built once at construction and kept in sync by [`set_id`](Self::set_id).
    /// Caching the full reference (rather than the bare id) means that
    /// [`SvgNode::set_filter_ref`](crate::SvgNode::set_filter_ref) can write it straight to the `filter` attribute with
    /// no per-call formatting allocation, however many elements the same filter is applied to.
    ///
    /// [`id`](Self::id) slices the bare id back out of this string rather than storing it separately.
    url_ref: String,
    element: SvgElement,
    document: Document,
    attrs: RefCell<SvgAttrs>,
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
impl SvgFilter {
    pub(crate) fn new(id: &str, element: SvgElement, document: Document) -> Self {
        let mut url_ref = String::with_capacity(URL_PREFIX.len() + id.len() + 1);
        url_ref.push_str(URL_PREFIX);
        url_ref.push_str(id);
        url_ref.push(')');
        Self {
            url_ref,
            element,
            document,
            attrs: RefCell::new(SvgAttrs::new()),
        }
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `id` of this filter.
    ///
    /// Pass this to [`SvgNode::set_filter`](crate::SvgNode::set_filter), or use
    /// [`SvgNode::set_filter_ref`](crate::SvgNode::set_filter_ref) with the handle to avoid touching the id.
    ///
    /// # ⚠️ Caveat ⚠️
    ///
    /// The returned value is sliced out of the cached `url(#id)` reference (see `url_ref`) built at construction time
    /// and kept in sync by [`set_id`](Self::set_id). The slice is exact because filter ids are restricted at validation
    /// time to match the pattern `[A-Za-z_][A-Za-z0-9_-]*`, which is pure ASCII, so byte offsets from `URL_PREFIX`'s
    /// length and the string's end always land on the bare id exactly.
    ///
    /// [`set_attr`](Self::set_attr) and [`set_attr_display`](Self::set_attr_display) reject `"id"` so they cannot
    /// desynchronise the cache through the normal API.
    ///
    /// The only remaining escape hatch is writing through [`as_element`](Self::as_element) directly, which bypasses
    /// all crate-level checks.
    ///
    /// Always use `set_id` to rename a filter after construction.
    pub fn id(&self) -> &str {
        &self.url_ref[URL_PREFIX.len()..self.url_ref.len() - 1]
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns the cached `url(#id)` reference, ready to write directly to a `filter` attribute.
    ///
    /// Visibility need only be `pub(crate)` since [`SvgNode::set_filter_ref`](crate::SvgNode::set_filter_ref) is the
    /// only caller that needs it; external callers use [`id`](Self::id) instead.
    pub(crate) fn url_ref(&self) -> &str {
        &self.url_ref
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Renames the filter by updating both the DOM `id` attribute and the cached value returned by [`id`](Self::id).
    ///
    /// This method takes `&mut self` because it mutates Rust-owned state (the cached reference string), unlike the
    /// other attribute setters that only write to the DOM.
    ///
    /// The new `id` is subject to the same validation rules as the id supplied at construction time: it must match the
    /// pattern `[A-Za-z_][A-Za-z0-9_-]*` — a letter or underscore followed by letters, digits, underscores, or hyphens.
    ///
    /// ⚠️ Caveat ⚠️
    ///
    /// Renaming a filter does not update any `filter` attributes already written to referencing elements — those store
    /// a snapshot of the reference at the time it was applied.
    ///
    /// Either rename before applying references, or reapply the reference after renaming it.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidFilterId`] — the new id failed validation.
    /// - [`Error::Dom`] — the browser refused to write the `id` attribute.
    pub fn set_id(&mut self, id: &str) -> Result<(), Error> {
        super::defs::validate_filter_id(id)?;
        self.element.set_attribute("id", id).map_err(dom_err)?;
        self.url_ref.clear();
        self.url_ref.push_str(URL_PREFIX);
        self.url_ref.push_str(id);
        self.url_ref.push(')');
        Ok(())
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Returns a reference to the underlying `web-sys` `SvgElement`.
    ///
    /// This provides a direct escape hatch to the DOM.
    ///
    /// Avoid writing the `id` attribute through this handle; use [`set_id`](Self::set_id) instead so the cached value
    /// stays in sync.
    pub fn as_element(&self) -> &SvgElement {
        &self.element
    }
}
