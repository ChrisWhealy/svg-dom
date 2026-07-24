use super::SvgFilter;
use crate::{Error, dom_err, root::create_svg_element};
use web_sys::SvgElement;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// The single light source child required by [`SvgFilter::diffuse_lighting`](super::SvgFilter::diffuse_lighting) and
/// [`specular_lighting`](super::SvgFilter::specular_lighting). It firstly selects which of the three SVG light-source
/// elements (`<feDistantLight>`, `<fePointLight>`, `<feSpotLight>`) is to be appended, and also supplies that element's
/// attributes.
///
/// Every variant here holds only `f64`/`Option<f64>` fields, not `Vec`/`String`, so deriving `Copy` costs nothing and
/// rules out an unnecessary move/borrow decision at every call site, the same/ judgement as has already been applied to
/// small coordinate types such as [`Point`](crate::root::utils::Point).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightSource {
    /// A `<feDistantLight>`: the light arrives as parallel rays from infinitely distant source, with no position of its
    /// own.  This the standard choice for an evenly lit surface, as if lit by sunlight.
    Distant {
        /// Direction angle on the XY plane, in degrees clockwise from the positive x-axis.
        azimuth: f64,
        /// Direction angle from the XY plane towards the positive z-axis (which points towards the viewer), in degrees.
        /// `90.0` shines straight down at the surface; small values graze across it, exaggerating any bump-map detail.
        elevation: f64,
    },

    /// A `<fePointLight>`: the light radiates outward from a single point in the same way a bare bulb does. This
    /// illumination model follows the inverse square law, so points closer to `(x, y, z)` are lit more intensely than
    /// distant ones.
    Point {
        /// The light position, in the coordinate system established by 
        /// [`primitiveUnits`](super::SvgFilter::set_primitive_units).  This is the same `primitiveUnits`-dependent
        /// interpretation as used by [`gaussian_blur`](super::SvgFilter::gaussian_blur)'s `std_deviation`.
        x: f64,
        /// See `x`'s own field doc, above.
        y: f64,
        /// Height above the surface, in the same coordinate system as `x`/`y`. Larger values move the light further
        /// from the surface, softening the falloff between near and far points.
        z: f64,
    },

    /// A `<feSpotLight>`: a light source that emits light similar to [`Point`](Self::Point), but aimed at
    /// `(points_at_x, points_at_y, points_at_z)` and optionally narrowed to a cone via `limiting_cone_angle` — the
    /// standard choice for a directed, theatrical spotlight rather than an omnidirectional point source.
    Spot {
        /// Light position — see [`Point`](Self::Point)'s own field docs for the coordinate system.
        x: f64,
        /// See `x`'s own field doc, above.
        y: f64,
        /// See `x`'s own field doc, above.
        z: f64,
        /// `x` of the point this spotlight is aimed at, in the same coordinate system as `x`/`y`/`z`.
        points_at_x: f64,
        /// See `points_at_x`'s own field doc, above.
        points_at_y: f64,
        /// See `points_at_x`'s own field doc, above.
        points_at_z: f64,
        /// Controls how tightly the light concentrates towards the direct axis between the light and the point it
        /// is aimed at.  In spite of sharing the same SVG attribute name and default (`1.0`) as
        /// [`specular_lighting`](super::SvgFilter::specular_lighting)'s `specular_exponent` parameter, this one shapes
        /// the spotlight's beam (larger values narrow and brighten the beam's centre), the Phong shininess exponent
        /// shapes how sharp the *surface's* specular highlight looks.
        ///
        /// Do not confuse the two when translating an SVG example that uses `specularExponent` on both
        /// `<feSpecularLighting>` and `<feSpotLight>`.
        specular_exponent: f64,
        /// The half-angle, in degrees, of the cone beyond which no light is projected — `None` (the SVG default when
        /// the attribute is omitted entirely) applies *no* limiting cone at all, projecting light in every direction
        /// from the spotlight's position. This is not the same as `Some(0.0)`: an explicit `0.0` is a cone with zero
        /// width, i.e. no light reaches the surface at all, an easy mistake if `None` is confused with "the smallest
        /// angle".
        ///
        /// Use `None` for an unconstrained spotlight and `Some(angle)` to narrow its beam.
        limiting_cone_angle: Option<f64>,
    },
}

impl LightSource {
    pub(super) fn tag(&self) -> &'static str {
        match self {
            Self::Distant { .. } => "feDistantLight",
            Self::Point { .. } => "fePointLight",
            Self::Spot { .. } => "feSpotLight",
        }
    }
}

impl SvgFilter {
    // - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
    /// Creates the one light-source child `light_source` describes, writes its attributes, and appends it to
    /// `parent` — shared by [`diffuse_lighting`](Self::diffuse_lighting) and
    /// [`specular_lighting`](Self::specular_lighting), the only two elements a light source can appear inside.
    pub(super) fn append_light_source(&self, parent: &SvgElement, light_source: LightSource) -> Result<(), Error> {
        let child = create_svg_element::<SvgElement>(&self.document, light_source.tag(), "SvgElement")?;
        {
            let mut attrs = self.attrs.borrow_mut();
            match light_source {
                LightSource::Distant { azimuth, elevation } => {
                    attrs.display_element(&child, "azimuth", azimuth)?;
                    attrs.display_element(&child, "elevation", elevation)?;
                },
                LightSource::Point { x, y, z } => {
                    attrs.display_element(&child, "x", x)?;
                    attrs.display_element(&child, "y", y)?;
                    attrs.display_element(&child, "z", z)?;
                },
                LightSource::Spot {
                    x,
                    y,
                    z,
                    points_at_x,
                    points_at_y,
                    points_at_z,
                    specular_exponent,
                    limiting_cone_angle,
                } => {
                    attrs.display_element(&child, "x", x)?;
                    attrs.display_element(&child, "y", y)?;
                    attrs.display_element(&child, "z", z)?;
                    attrs.display_element(&child, "pointsAtX", points_at_x)?;
                    attrs.display_element(&child, "pointsAtY", points_at_y)?;
                    attrs.display_element(&child, "pointsAtZ", points_at_z)?;
                    attrs.display_element(&child, "specularExponent", specular_exponent)?;
                    if let Some(angle) = limiting_cone_angle {
                        attrs.display_element(&child, "limitingConeAngle", angle)?;
                    }
                },
            }
        }
        parent.append_child(&child).map_err(dom_err)?;
        Ok(())
    }
}
