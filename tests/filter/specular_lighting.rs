use crate::common::*;
use svg_dom::LightSource;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// specular_lighting primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `specular_lighting` appends a `<feSpecularLighting>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_specular_lighting_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-specular-lighting-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fslc").map_err(|e| e.to_string())?;
    filter
        .specular_lighting(
            6.0,
            1.0,
            20.0,
            "white",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feSpecularLighting"`.
#[wasm_bindgen_test]
fn should_create_fe_specular_lighting_element() -> Result<(), String> {
    let svg = make_svg("filter-specular-lighting-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fslt").map_err(|e| e.to_string())?;
    let lit = filter
        .specular_lighting(
            6.0,
            1.0,
            20.0,
            "white",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )
        .map_err(|e| e.to_string())?;
    check_eq(lit.as_element().tag_name(), "feSpecularLighting".to_owned())
}

/// `specular_lighting` writes `surfaceScale`, `specularConstant`, `specularExponent`, and `lighting-color`.
#[wasm_bindgen_test]
fn should_set_specular_lighting_attrs() -> Result<(), String> {
    let svg = make_svg("filter-specular-lighting-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fsla").map_err(|e| e.to_string())?;
    let lit = filter
        .specular_lighting(
            4.5,
            1.2,
            24.0,
            "steelblue",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )
        .map_err(|e| e.to_string())?;
    let el = lit.as_element();
    check_eq(el.get_attribute("surfaceScale"), Some("4.5".into()))?;
    check_eq(el.get_attribute("specularConstant"), Some("1.2".into()))?;
    check_eq(el.get_attribute("specularExponent"), Some("24".into()))?;
    check_eq(el.get_attribute("lighting-color"), Some("steelblue".into()))
}

/// `specular_lighting`'s own `specular_exponent` parameter (Phong shininess) is written to the outer
/// `<feSpecularLighting>` element, distinct from `LightSource::Spot`'s own `specular_exponent` field, which is
/// written to the nested `<feSpotLight>` child instead — the two must not collide.
#[wasm_bindgen_test]
fn should_keep_specular_exponent_distinct_from_spot_light_specular_exponent() -> Result<(), String> {
    let svg = make_svg("filter-specular-lighting-spot");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fsls").map_err(|e| e.to_string())?;
    let lit = filter
        .specular_lighting(
            1.0,
            1.0,
            30.0,
            "white",
            LightSource::Spot {
                x: 600.0,
                y: 600.0,
                z: 400.0,
                points_at_x: 0.0,
                points_at_y: 0.0,
                points_at_z: 0.0,
                specular_exponent: 3.0,
                limiting_cone_angle: Some(5.5),
            },
        )
        .map_err(|e| e.to_string())?;
    let outer = lit.as_element();
    let light = outer.first_element_child().ok_or("expected a light-source child")?;
    check_eq(light.tag_name(), "feSpotLight".to_owned())?;
    check_eq(outer.get_attribute("specularExponent"), Some("30".into()))?;
    check_eq(light.get_attribute("specularExponent"), Some("3".into()))?;
    check_eq(light.get_attribute("limitingConeAngle"), Some("5.5".into()))
}

/// The generic `SvgNode::set_attr` escape hatch on the returned primitive node covers attributes not yet wrapped
/// by a named parameter, such as `in`, `result`, and `kernelUnitLength`.
#[wasm_bindgen_test]
fn should_set_unwrapped_attrs_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-specular-lighting-escape-hatch");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fsleh").map_err(|e| e.to_string())?;
    let lit = filter
        .specular_lighting(
            6.0,
            1.0,
            20.0,
            "white",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )
        .map_err(|e| e.to_string())?;
    lit.set_attr("in", "SourceAlpha").map_err(|e| e.to_string())?;
    lit.set_attr("result", "highlight").map_err(|e| e.to_string())?;
    lit.set_attr("kernelUnitLength", "1 1").map_err(|e| e.to_string())?;
    let el = lit.as_element();
    check_eq(el.get_attribute("in"), Some("SourceAlpha".into()))?;
    check_eq(el.get_attribute("result"), Some("highlight".into()))?;
    check_eq(el.get_attribute("kernelUnitLength"), Some("1 1".into()))
}
