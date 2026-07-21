use super::super::{ColorMatrixType, SvgFilter};
use crate::{Error, SvgNode, dom_err, root::create_svg_element};
use web_sys::SvgElement;

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Appends a `<feColorMatrix>` primitive to this filter, transforming colours via [`matrix_type`](ColorMatrixType).
    ///
    /// Writes the SVG `type` attribute from `matrix_type`'s variant, and — for every variant except
    /// [`LuminanceToAlpha`](ColorMatrixType::LuminanceToAlpha), which needs none — the matching `values` attribute:
    /// twenty space-separated numbers for [`Matrix`](ColorMatrixType::Matrix), or a single number for
    /// [`Saturate`](ColorMatrixType::Saturate)/[`HueRotate`](ColorMatrixType::HueRotate).
    ///
    /// If this is the filter's first primitive, its implicit input is `SourceGraphic`. Use the returned [`SvgNode`]'s
    /// [`set_attr`](crate::SvgNode::set_attr) to set `in` or `result`, neither of which has a dedicated setter yet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dom`] if the browser refuses to create or append the `<feColorMatrix>` element.
    ///
    /// # Example
    ///
    /// A fully desaturated (greyscale) copy of the source graphic:
    ///
    /// ```rust,no_run
    /// use svg_dom::{SvgRoot, root::filter::ColorMatrixType};
    ///
    /// let svg  = SvgRoot::attach("diagram")?;
    /// let defs = svg.defs()?;
    /// let grey = defs.filter("greyscale")?;
    /// grey.color_matrix(ColorMatrixType::Saturate(0.0))?;
    /// Ok::<(), svg_dom::Error>(())
    /// ```
    pub fn color_matrix(&self, matrix_type: ColorMatrixType) -> Result<SvgNode, Error> {
        let el = create_svg_element::<SvgElement>(&self.document, "feColorMatrix", "SvgElement")?;
        el.set_attribute("type", matrix_type.as_str()).map_err(dom_err)?;
        match matrix_type {
            ColorMatrixType::Matrix(m) => {
                self.attrs.borrow_mut().display_element(
                    &el,
                    "values",
                    format_args!(
                        "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                        m[0],
                        m[1],
                        m[2],
                        m[3],
                        m[4],
                        m[5],
                        m[6],
                        m[7],
                        m[8],
                        m[9],
                        m[10],
                        m[11],
                        m[12],
                        m[13],
                        m[14],
                        m[15],
                        m[16],
                        m[17],
                        m[18],
                        m[19],
                    ),
                )?;
            },
            ColorMatrixType::Saturate(v) | ColorMatrixType::HueRotate(v) => {
                self.attrs.borrow_mut().display_element(&el, "values", v)?;
            },
            ColorMatrixType::LuminanceToAlpha => {},
        }
        self.element.append_child(&el).map_err(dom_err)?;
        Ok(SvgNode::new(el))
    }
}
