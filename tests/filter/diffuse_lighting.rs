use crate::common::*;
use svg_dom::LightSource;
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// diffuse_lighting primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `diffuse_lighting` appends a `<feDiffuseLighting>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_diffuse_lighting_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-diffuse-lighting-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdlc").map_err(|e| e.to_string())?;
    filter
        .diffuse_lighting(
            6.0,
            1.0,
            "white",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feDiffuseLighting"`.
#[wasm_bindgen_test]
fn should_create_fe_diffuse_lighting_element() -> Result<(), String> {
    let svg = make_svg("filter-diffuse-lighting-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdlt").map_err(|e| e.to_string())?;
    let lit = filter
        .diffuse_lighting(
            6.0,
            1.0,
            "white",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )
        .map_err(|e| e.to_string())?;
    check_eq(lit.as_element().tag_name(), "feDiffuseLighting".to_owned())
}

/// `diffuse_lighting` writes `surfaceScale`, `diffuseConstant`, and `lighting-color`.
#[wasm_bindgen_test]
fn should_set_diffuse_lighting_attrs() -> Result<(), String> {
    let svg = make_svg("filter-diffuse-lighting-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdla").map_err(|e| e.to_string())?;
    let lit = filter
        .diffuse_lighting(
            4.5,
            1.2,
            "steelblue",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )
        .map_err(|e| e.to_string())?;
    let el = lit.as_element();
    check_eq(el.get_attribute("surfaceScale"), Some("4.5".into()))?;
    check_eq(el.get_attribute("diffuseConstant"), Some("1.2".into()))?;
    check_eq(el.get_attribute("lighting-color"), Some("steelblue".into()))
}

/// `LightSource::Distant` appends a `<feDistantLight>` child with `azimuth`/`elevation`.
#[wasm_bindgen_test]
fn should_append_distant_light_child() -> Result<(), String> {
    let svg = make_svg("filter-diffuse-lighting-distant");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdld").map_err(|e| e.to_string())?;
    let lit = filter
        .diffuse_lighting(
            1.0,
            1.0,
            "white",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )
        .map_err(|e| e.to_string())?;
    let light = lit.as_element().first_element_child().ok_or("expected a light-source child")?;
    check_eq(light.tag_name(), "feDistantLight".to_owned())?;
    check_eq(light.get_attribute("azimuth"), Some("235".into()))?;
    check_eq(light.get_attribute("elevation"), Some("55".into()))
}

/// `LightSource::Point` appends a `<fePointLight>` child with `x`/`y`/`z`.
#[wasm_bindgen_test]
fn should_append_point_light_child() -> Result<(), String> {
    let svg = make_svg("filter-diffuse-lighting-point");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdlp").map_err(|e| e.to_string())?;
    let lit = filter
        .diffuse_lighting(1.0, 1.0, "white", LightSource::Point { x: 150.0, y: 60.0, z: 20.0 })
        .map_err(|e| e.to_string())?;
    let light = lit.as_element().first_element_child().ok_or("expected a light-source child")?;
    check_eq(light.tag_name(), "fePointLight".to_owned())?;
    check_eq(light.get_attribute("x"), Some("150".into()))?;
    check_eq(light.get_attribute("y"), Some("60".into()))?;
    check_eq(light.get_attribute("z"), Some("20".into()))
}

/// `LightSource::Spot` with `limiting_cone_angle: Some(_)` appends a `<feSpotLight>` child with every attribute,
/// including `limitingConeAngle`.
#[wasm_bindgen_test]
fn should_append_spot_light_child_with_cone_angle() -> Result<(), String> {
    let svg = make_svg("filter-diffuse-lighting-spot-cone");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdlsc").map_err(|e| e.to_string())?;
    let lit = filter
        .diffuse_lighting(
            1.0,
            1.0,
            "white",
            LightSource::Spot {
                x: 600.0,
                y: 600.0,
                z: 400.0,
                points_at_x: 0.0,
                points_at_y: 0.0,
                points_at_z: 0.0,
                specular_exponent: 1.0,
                limiting_cone_angle: Some(5.5),
            },
        )
        .map_err(|e| e.to_string())?;
    let light = lit.as_element().first_element_child().ok_or("expected a light-source child")?;
    check_eq(light.tag_name(), "feSpotLight".to_owned())?;
    check_eq(light.get_attribute("x"), Some("600".into()))?;
    check_eq(light.get_attribute("y"), Some("600".into()))?;
    check_eq(light.get_attribute("z"), Some("400".into()))?;
    check_eq(light.get_attribute("pointsAtX"), Some("0".into()))?;
    check_eq(light.get_attribute("pointsAtY"), Some("0".into()))?;
    check_eq(light.get_attribute("pointsAtZ"), Some("0".into()))?;
    check_eq(light.get_attribute("specularExponent"), Some("1".into()))?;
    check_eq(light.get_attribute("limitingConeAngle"), Some("5.5".into()))
}

/// `LightSource::Spot` with `limiting_cone_angle: None` omits `limitingConeAngle` entirely — the SVG default
/// (no limiting cone at all), distinct from writing an explicit `0`.
#[wasm_bindgen_test]
fn should_omit_limiting_cone_angle_when_none() -> Result<(), String> {
    let svg = make_svg("filter-diffuse-lighting-spot-no-cone");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdlsn").map_err(|e| e.to_string())?;
    let lit = filter
        .diffuse_lighting(
            1.0,
            1.0,
            "white",
            LightSource::Spot {
                x: 600.0,
                y: 600.0,
                z: 400.0,
                points_at_x: 0.0,
                points_at_y: 0.0,
                points_at_z: 0.0,
                specular_exponent: 1.0,
                limiting_cone_angle: None,
            },
        )
        .map_err(|e| e.to_string())?;
    let light = lit.as_element().first_element_child().ok_or("expected a light-source child")?;
    check_eq(light.get_attribute("limitingConeAngle"), None)
}

/// The generic `SvgNode::set_attr` escape hatch on the returned primitive node covers attributes not yet wrapped
/// by a named parameter, such as `in`, `result`, and `kernelUnitLength`.
#[wasm_bindgen_test]
fn should_set_unwrapped_attrs_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-diffuse-lighting-escape-hatch");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdleh").map_err(|e| e.to_string())?;
    let lit = filter
        .diffuse_lighting(
            6.0,
            1.0,
            "white",
            LightSource::Distant {
                azimuth: 235.0,
                elevation: 55.0,
            },
        )
        .map_err(|e| e.to_string())?;
    lit.set_attr("in", "SourceAlpha").map_err(|e| e.to_string())?;
    lit.set_attr("result", "lit").map_err(|e| e.to_string())?;
    lit.set_attr("kernelUnitLength", "1 1").map_err(|e| e.to_string())?;
    let el = lit.as_element();
    check_eq(el.get_attribute("in"), Some("SourceAlpha".into()))?;
    check_eq(el.get_attribute("result"), Some("lit".into()))?;
    check_eq(el.get_attribute("kernelUnitLength"), Some("1 1".into()))
}
