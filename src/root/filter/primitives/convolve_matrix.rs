use super::{
    super::{EdgeMode, SvgFilter},
    SpaceSeparated,
};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use std::fmt;
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Shared implementation behind [`convolve_matrix`](Self::convolve_matrix) and
    /// [`convolve_matrix_xy`](Self::convolve_matrix_xy): creates a `<feConvolveMatrix>`, writes `order` alongside
    /// `kernelMatrix`, `divisor`, `edgeMode`, and `preserveAlpha`, then appends it.
    ///
    /// `order` is a pre-built [`fmt::Arguments`] rather than a `&str`.
    /// This lets the two public callers pass either a single number or an `"x y"` pair through
    /// [`display_element`](crate::root::attrs::SvgAttrs::display_element)'s retained scratch buffer, without first
    /// collecting into an owned `String`.
    /// The private `gaussian_blur_args`/`turbulence_args`/`morphology_args` helpers use the same technique for
    /// their own `<number-optional-number>` attribute.
    fn convolve_matrix_args(
        &self,
        order: fmt::Arguments<'_>,
        kernel_matrix: &[f64],
        divisor: f64,
        edge_mode: EdgeMode,
        preserve_alpha: bool,
    ) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feConvolveMatrix", "SvgElement")?;
        el.set_attribute("edgeMode", edge_mode.as_str()).map_err(dom_err)?;
        el.set_attribute("preserveAlpha", if preserve_alpha { "true" } else { "false" })
            .map_err(dom_err)?;
        {
            let mut attrs = self.attrs.borrow_mut();
            attrs.display_element(&el, "order", order)?;
            attrs.display_element(&el, "kernelMatrix", SpaceSeparated(kernel_matrix))?;
            attrs.display_element(&el, "divisor", divisor)?;
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feConvolveMatrix>` primitive to this filter, applying a square `order`×`order` matrix convolution.
    /// This is the general image-processing operation behind sharpening, blurring, embossing, and edge-detection
    /// kernels.
    ///
    /// `kernel_matrix` must contain exactly `order * order` values, in row-major order (left-to-right, top-to-bottom,
    /// matching the SVG spec's own reading order for the rectangle it describes).
    /// Per the SVG specification, the kernel is applied *rotated 180 degrees* relative to the input.
    /// This matches the convolution convention used in most computer-graphics textbooks.
    /// For a kernel that is not rotationally symmetric, a directional edge-detect for instance, write it already
    /// accounting for this rotation.
    /// Equivalently, treat the values you supply as already describing the rotated kernel.
    ///
    /// For each output pixel, every kernel entry is multiplied by the corresponding input pixel in its
    /// `order`×`order` neighbourhood.
    /// The products are summed, divided by `divisor`, and `bias` is added: `(Σ kernel × source) / divisor + bias`.
    /// `bias` defaults to `0.0` unless set via the generic escape hatch — see below.
    ///
    /// Prefer small kernels, normally `3` or `5`.
    /// This per-pixel sum is taken over `order * order` entries, so rendering cost rises with the *square* of
    /// `order`.
    /// The SVG specification itself recommends small values (`3` given as its own example).
    /// It warns that larger ones "may result in very high CPU overhead" without a proportionate visual benefit.
    /// This is a rendering-cost warning about the browser's own evaluation of the resulting `<feConvolveMatrix>`,
    /// not a cost inside this crate.
    /// Serializing `kernel_matrix` here is linear in its length, regardless of `order`.
    /// A large `order` is not rejected; it is legal SVG and occasionally necessary, just markedly more expensive to
    /// render than the `3`×`3`/`5`×`5` kernels used throughout this primitive's own examples.
    ///
    /// `divisor` scales the summed products down to a usable range.
    /// For a kernel whose values already sum to `1.0`, the common case for a blur or sharpen kernel that should
    /// preserve overall brightness, `1.0` is the natural choice.
    /// A kernel whose values sum to something else, many edge-detect kernels sum to `0`, still needs an explicit
    /// `divisor`.
    /// There is no single value that is "natural" for every such kernel.
    /// `1.0` is a reasonable default when in doubt, and is what every example below uses.
    ///
    /// Per the SVG spec, `divisor: 0.0` is not an error.
    /// The renderer silently substitutes the sum of `kernel_matrix`'s own values instead, or `1.0` if that sum is
    /// itself `0.0`, rather than dividing by zero.
    /// This crate does not special-case or reject `0.0` before it reaches the DOM, since the fallback is already
    /// well-defined.
    /// Pass the value you actually intend, rather than relying on it.
    ///
    /// `edge_mode` selects how the input is virtually extended wherever the kernel overhangs its border — see
    /// [`EdgeMode`] for the three keywords and what each looks like in practice.
    ///
    /// `preserve_alpha`, if `true`, un-premultiplies colour before convolving, so only R/G/B are affected and alpha
    /// passes through completely unfiltered.
    /// It then re-premultiplies the result.
    /// This is the usual choice when convolving a partially-transparent input whose edges should not visibly erode
    /// or bleed.
    /// If `false` (the SVG default), the convolution runs directly on premultiplied R/G/B/A, so the alpha channel is
    /// convolved too, alongside colour.
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`.
    /// Use the returned [`SvgNode`]'s [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result`, since neither
    /// has a dedicated setter.
    /// Do the same for `bias`, `targetX`, or `targetY`.
    /// Each of these keeps its own SVG default unless set explicitly — see the warning below for `bias`.
    ///
    /// `kernelUnitLength` is also reachable the same way, but is a deprecated legacy attribute for requesting
    /// explicit kernel sampling intervals.
    /// The current Filter Effects specification marks it deprecated and slated for removal.
    /// It does not reliably achieve the device-independent rendering it was meant to provide.
    /// It remains available through `set_attr` since a deprecated attribute is not a removed one, but should not be
    /// relied upon for platform-independent rendering.
    ///
    /// See [`convolve_matrix_xy`](Self::convolve_matrix_xy) for an `order_x`×`order_y` rectangular kernel — the SVG
    /// `order` attribute accepts either one or two numbers, and this method covers only the one-number,
    /// square-kernel form.
    ///
    /// ***⚠️ A `kernel_matrix` whose length does not equal `order * order` is not rejected***.
    /// Per the SVG spec, `<feConvolveMatrix>` "acts as a pass through filter" in that case, and `in` is rendered
    /// unchanged.
    /// This crate does not raise an error, nor does the browser refuse to render.
    /// Double-check `kernel_matrix.len()` against `order * order` yourself; a silently inert filter is easy to
    /// mistake for a filter that simply has no visible effect on the chosen input.
    ///
    /// ***⚠️ `bias` defaults to `0.0`, which clamps every negative convolution result to black***.
    /// A kernel whose values can produce a negative sum, most edge-detect and emboss kernels, needs a non-zero
    /// `bias` to shift that range back into the visible `0.0`–`1.0` window.
    /// `0.5` is the standard choice for a *classic* embossed-grey look.
    /// This makes the flat (zero-response) areas of the image render as mid-grey, rather than black.
    /// Set it via `set_attr("bias", "0.5")` on the returned node — see the emboss example below.
    ///
    /// `order` itself, unlike the two caveats above, *is* rejected when it is `0`.
    /// The SVG spec requires `order`'s component(s) to be an integer greater than zero.
    /// Unlike the length-mismatch or zero-`divisor` cases, it gives no defined fallback for a zero component.
    /// This crate therefore returns [`Error::InvalidConvolveMatrixOrder`], rather than serializing a value the
    /// specification never assigns a meaning to.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidConvolveMatrixOrder`] if `order` is `0`.
    /// - [`Error::Dom`] if the browser refuses to create or append the `<feConvolveMatrix>` element.
    ///
    /// # Example
    ///
    /// A classic 3×3 sharpen kernel — its values already sum to `1.0`, so `divisor` is `1.0` and the result needs no
    /// `bias`:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::EdgeMode};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("sharpen")?;
    /// #[rustfmt::skip]
    /// let kernel = [
    ///      0.0, -1.0,  0.0,
    ///     -1.0,  5.0, -1.0,
    ///      0.0, -1.0,  0.0,
    /// ];
    /// flt.convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, false)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    ///
    /// A 3×3 emboss kernel — its values sum to `0.0`, so a flat region of input convolves to `0.0`.
    /// `bias: 0.5` (set via the generic escape hatch, since it is not one of this method's own parameters) shifts
    /// that midpoint up to mid-grey instead of black:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::EdgeMode};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("emboss")?;
    /// #[rustfmt::skip]
    /// let kernel = [
    ///     -2.0, -1.0, 0.0,
    ///     -1.0,  1.0, 1.0,
    ///      0.0,  1.0, 2.0,
    /// ];
    /// flt.convolve_matrix(3, &kernel, 1.0, EdgeMode::Duplicate, true)?
    ///     .set_attr("bias", "0.5")?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn convolve_matrix(
        &self,
        order: u32,
        kernel_matrix: &[f64],
        divisor: f64,
        edge_mode: EdgeMode,
        preserve_alpha: bool,
    ) -> Result<SvgNode, Error> {
        if order == 0 {
            return Err(Error::InvalidConvolveMatrixOrder("order must be greater than zero"));
        }
        self.convolve_matrix_args(format_args!("{order}"), kernel_matrix, divisor, edge_mode, preserve_alpha)
    }

    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feConvolveMatrix>` primitive to this filter with an `order_x`×`order_y` rectangular kernel,
    /// writing the SVG `order="order_x order_y"` two-number form internally.
    ///
    /// `kernel_matrix` must contain exactly `order_x * order_y` values: `order_x` columns per row, `order_y` rows.
    /// This is the same row-major, 180-degree-rotated sense [`convolve_matrix`](Self::convolve_matrix)'s own doc
    /// comment describes.
    ///
    /// A rectangular kernel is the natural shape for a directional effect.
    /// A `1`×`n` or `n`×`1` kernel applies along one axis only, a horizontal-only or vertical-only blur, sharpen, or
    /// motion-streak.
    /// A square kernel of the same total width instead produces an isotropic effect along both axes at once.
    ///
    /// See [`convolve_matrix`](Self::convolve_matrix) for `divisor`, `edge_mode`, `preserve_alpha`, the `in`/`result`/
    /// `bias`/`targetX`/`targetY` escape hatch, the length-mismatch pass-through caveat, the `bias` warning, and the
    /// small-kernel performance recommendation.
    /// All of these apply identically here.
    /// The rendering cost rises with `order_x * order_y` just as it does with `order * order` for a square kernel.
    ///
    /// `kernelUnitLength` is likewise reachable via the same escape hatch, but see
    /// [`convolve_matrix`](Self::convolve_matrix)'s own doc comment for why it is a deprecated attribute this crate
    /// does not recommend relying on.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidConvolveMatrixOrder`] if `order_x` or `order_y` is `0` — see
    ///   [`convolve_matrix`](Self::convolve_matrix)'s own doc comment for why this, unlike a `kernel_matrix`
    ///   length mismatch or a zero `divisor`, is rejected rather than documented.
    /// - [`Error::Dom`] if the browser refuses to create or append the `<feConvolveMatrix>` element.
    ///
    /// # Example
    ///
    /// A 1×3 horizontal-only blur — three equal weights along `x`, none along `y`:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::EdgeMode};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let flt  = defs.filter("horizontal-streak")?;
    /// let kernel = [1.0, 1.0, 1.0];
    /// flt.convolve_matrix_xy(3, 1, &kernel, 3.0, EdgeMode::Duplicate, false)?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn convolve_matrix_xy(
        &self,
        order_x: u32,
        order_y: u32,
        kernel_matrix: &[f64],
        divisor: f64,
        edge_mode: EdgeMode,
        preserve_alpha: bool,
    ) -> Result<SvgNode, Error> {
        if order_x == 0 || order_y == 0 {
            return Err(Error::InvalidConvolveMatrixOrder(
                "order_x and order_y must both be greater than zero",
            ));
        }
        self.convolve_matrix_args(
            format_args!("{order_x} {order_y}"),
            kernel_matrix,
            divisor,
            edge_mode,
            preserve_alpha,
        )
    }
}
