mod common;

use common::*;
use svg_dom::{
    BlendMode, Channel, ColorMatrixType, CompositeOperator, Error, FilterUnits, TransferFunction,
    root::utils::{Point, Size},
};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::filter — eager variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `defs.filter(id)` creates an element with tag name `"filter"`.
#[wasm_bindgen_test]
fn should_create_filter_element() -> Result<(), String> {
    let svg = make_svg("filter-create");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("blur").map_err(|e| e.to_string())?;
    check_eq(filter.as_element().tag_name(), "filter".to_owned())
}

/// The `<filter>` element is a child of `<defs>`, not of the root `<svg>`.
#[wasm_bindgen_test]
fn should_append_filter_to_defs() -> Result<(), String> {
    let svg = make_svg("filter-parent");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("soft").map_err(|e| e.to_string())?;
    let parent = filter.as_element().parent_element().ok_or("filter has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// `defs.filter(id)` sets the `id` attribute on the element.
#[wasm_bindgen_test]
fn should_set_filter_id_attribute() -> Result<(), String> {
    let svg = make_svg("filter-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("my-filter").map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("id"), Some("my-filter".into()))
}

/// `SvgFilter::id()` returns the cached id without a DOM round-trip.
#[wasm_bindgen_test]
fn should_return_cached_filter_id() -> Result<(), String> {
    let svg = make_svg("filter-cached-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("cached").map_err(|e| e.to_string())?;
    check_eq(filter.id(), "cached")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgDefs::build_filter — deferred/transactional variant
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `build_filter` appends the element to `<defs>` only after the closure returns `Ok`.
#[wasm_bindgen_test]
fn should_build_filter_and_append_to_defs() -> Result<(), String> {
    let svg = make_svg("filter-build");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("built", |f| {
            f.gaussian_blur(3.0)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    let parent = filter.as_element().parent_element().ok_or("filter has no parent")?;
    check_eq(parent.tag_name(), "defs".to_owned())
}

/// When `build_filter`'s closure returns `Err`, the filter element is not attached to `<defs>`.
#[wasm_bindgen_test]
fn should_not_append_filter_on_closure_error() -> Result<(), String> {
    let svg = make_svg("filter-build-err");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.build_filter("failing", |_f| Err(Error::Dom("deliberate".into())));
    check(result.is_err(), "expected build_filter to return Err")?;
    // The <defs> element should have no children since the closure failed.
    let defs_el = defs.as_element();
    check_eq(defs_el.child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// gaussian_blur primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `gaussian_blur` appends a `<feGaussianBlur>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_gaussian_blur_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-blur-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fb").map_err(|e| e.to_string())?;
    filter.gaussian_blur(4.0).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feGaussianBlur"`.
#[wasm_bindgen_test]
fn should_create_fe_gaussian_blur_element() -> Result<(), String> {
    let svg = make_svg("filter-blur-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbt").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur(4.0).map_err(|e| e.to_string())?;
    check_eq(blur.as_element().tag_name(), "feGaussianBlur".to_owned())
}

/// `gaussian_blur` writes the `stdDeviation` attribute.
#[wasm_bindgen_test]
fn should_set_std_deviation() -> Result<(), String> {
    let svg = make_svg("filter-blur-std-dev");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fsd").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur(6.5).map_err(|e| e.to_string())?;
    check_eq(blur.as_element().get_attribute("stdDeviation"), Some("6.5".into()))
}

/// Multiple primitives can be added to the same filter, in document order.
#[wasm_bindgen_test]
fn should_add_multiple_primitives_in_order() -> Result<(), String> {
    let svg = make_svg("filter-multi");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fm").map_err(|e| e.to_string())?;
    filter.gaussian_blur(2.0).map_err(|e| e.to_string())?;
    filter.gaussian_blur(8.0).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 2)
}

/// The generic `SvgNode::set_attr` escape hatch on the returned primitive node covers attributes not yet wrapped by
/// a named parameter, such as `in` and `result`.
#[wasm_bindgen_test]
fn should_set_result_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-blur-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fr").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur(4.0).map_err(|e| e.to_string())?;
    blur.set_attr("result", "blurred").map_err(|e| e.to_string())?;
    check_eq(blur.as_element().get_attribute("result"), Some("blurred".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// gaussian_blur_xy primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `gaussian_blur_xy` appends a `<feGaussianBlur>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_gaussian_blur_xy_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-blur-xy-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbxy").map_err(|e| e.to_string())?;
    filter.gaussian_blur_xy(3.0, 6.5).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feGaussianBlur"`, the same element `gaussian_blur` produces.
#[wasm_bindgen_test]
fn should_create_fe_gaussian_blur_element_via_xy() -> Result<(), String> {
    let svg = make_svg("filter-blur-xy-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbxyt").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur_xy(3.0, 6.5).map_err(|e| e.to_string())?;
    check_eq(blur.as_element().tag_name(), "feGaussianBlur".to_owned())
}

/// `gaussian_blur_xy(3.0, 6.5)` writes the two-number `stdDeviation="3 6.5"` form in a single attribute, exactly
/// as the SVG `<number-optional-number>` grammar for `stdDeviation` requires.
#[wasm_bindgen_test]
fn should_set_std_deviation_as_two_numbers() -> Result<(), String> {
    let svg = make_svg("filter-blur-xy-std-dev");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbxysd").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur_xy(3.0, 6.5).map_err(|e| e.to_string())?;
    check_eq(blur.as_element().get_attribute("stdDeviation"), Some("3 6.5".into()))
}

/// Passing `0.0` for one axis blurs only along the other, per the SVG grammar's documented use case.
#[wasm_bindgen_test]
fn should_allow_zero_on_one_axis() -> Result<(), String> {
    let svg = make_svg("filter-blur-xy-zero-axis");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbxyz").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur_xy(0.0, 8.0).map_err(|e| e.to_string())?;
    check_eq(blur.as_element().get_attribute("stdDeviation"), Some("0 8".into()))
}

/// The generic `SvgNode::set_attr` escape hatch works identically on a `gaussian_blur_xy` node as on a
/// `gaussian_blur` one, since both return the same kind of handle around the same element.
#[wasm_bindgen_test]
fn should_set_result_on_gaussian_blur_xy_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-blur-xy-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbxyr").map_err(|e| e.to_string())?;
    let blur = filter.gaussian_blur_xy(3.0, 6.5).map_err(|e| e.to_string())?;
    blur.set_attr("result", "blurred").map_err(|e| e.to_string())?;
    check_eq(blur.as_element().get_attribute("result"), Some("blurred".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// offset primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `offset` appends a `<feOffset>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_offset_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-offset-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fo").map_err(|e| e.to_string())?;
    filter.offset(4.0, 4.0).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feOffset"`.
#[wasm_bindgen_test]
fn should_create_fe_offset_element() -> Result<(), String> {
    let svg = make_svg("filter-offset-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fot").map_err(|e| e.to_string())?;
    let offset = filter.offset(4.0, 4.0).map_err(|e| e.to_string())?;
    check_eq(offset.as_element().tag_name(), "feOffset".to_owned())
}

/// `offset` writes the `dx` and `dy` attributes.
#[wasm_bindgen_test]
fn should_set_dx_dy() -> Result<(), String> {
    let svg = make_svg("filter-offset-dxdy");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdd").map_err(|e| e.to_string())?;
    let offset = filter.offset(3.5, -2.0).map_err(|e| e.to_string())?;
    check_eq(offset.as_element().get_attribute("dx"), Some("3.5".into()))?;
    check_eq(offset.as_element().get_attribute("dy"), Some("-2".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// merge primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `merge` appends a single `<feMerge>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_merge_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-merge-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmg").map_err(|e| e.to_string())?;
    filter.merge(&["a", "b"]).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feMerge"`.
#[wasm_bindgen_test]
fn should_create_fe_merge_element() -> Result<(), String> {
    let svg = make_svg("filter-merge-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmt").map_err(|e| e.to_string())?;
    let merge = filter.merge(&["a", "b"]).map_err(|e| e.to_string())?;
    check_eq(merge.as_element().tag_name(), "feMerge".to_owned())
}

/// `merge` appends one `<feMergeNode>` per input, in order.
#[wasm_bindgen_test]
fn should_add_one_merge_node_per_input() -> Result<(), String> {
    let svg = make_svg("filter-merge-nodes");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fmn").map_err(|e| e.to_string())?;
    let merge = filter.merge(&["offset-blur", "SourceGraphic"]).map_err(|e| e.to_string())?;
    let el = merge.as_element();
    check_eq(el.child_element_count(), 2)?;
    let first = el.first_element_child().ok_or("missing first feMergeNode")?;
    let second = first.next_element_sibling().ok_or("missing second feMergeNode")?;
    check_eq(first.tag_name(), "feMergeNode".to_owned())?;
    check_eq(first.get_attribute("in"), Some("offset-blur".into()))?;
    check_eq(second.tag_name(), "feMergeNode".to_owned())?;
    check_eq(second.get_attribute("in"), Some("SourceGraphic".into()))
}

/// `merge` with an empty input slice appends a `<feMerge>` with no children.
#[wasm_bindgen_test]
fn should_add_empty_merge_for_no_inputs() -> Result<(), String> {
    let svg = make_svg("filter-merge-empty");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fme").map_err(|e| e.to_string())?;
    let merge = filter.merge(&[]).map_err(|e| e.to_string())?;
    check_eq(merge.as_element().child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// flood primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `flood` appends a `<feFlood>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_flood_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-flood-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ffl").map_err(|e| e.to_string())?;
    filter.flood("black", 0.5).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feFlood"`.
#[wasm_bindgen_test]
fn should_create_fe_flood_element() -> Result<(), String> {
    let svg = make_svg("filter-flood-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fflt").map_err(|e| e.to_string())?;
    let flood = filter.flood("black", 0.5).map_err(|e| e.to_string())?;
    check_eq(flood.as_element().tag_name(), "feFlood".to_owned())
}

/// `flood` writes the `flood-color` and `flood-opacity` attributes.
#[wasm_bindgen_test]
fn should_set_flood_color_and_opacity() -> Result<(), String> {
    let svg = make_svg("filter-flood-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ffa").map_err(|e| e.to_string())?;
    let flood = filter.flood("crimson", 0.65).map_err(|e| e.to_string())?;
    check_eq(flood.as_element().get_attribute("flood-color"), Some("crimson".into()))?;
    check_eq(flood.as_element().get_attribute("flood-opacity"), Some("0.65".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// composite primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `composite` appends a `<feComposite>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_composite_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-composite-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcp").map_err(|e| e.to_string())?;
    filter.composite("blur", CompositeOperator::In).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feComposite"`.
#[wasm_bindgen_test]
fn should_create_fe_composite_element() -> Result<(), String> {
    let svg = make_svg("filter-composite-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcpt").map_err(|e| e.to_string())?;
    let composite = filter.composite("blur", CompositeOperator::In).map_err(|e| e.to_string())?;
    check_eq(composite.as_element().tag_name(), "feComposite".to_owned())
}

/// `composite` writes the `in2` and `operator` attributes.
#[wasm_bindgen_test]
fn should_set_in2_and_operator() -> Result<(), String> {
    let svg = make_svg("filter-composite-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fca").map_err(|e| e.to_string())?;
    let composite = filter.composite("blur", CompositeOperator::In).map_err(|e| e.to_string())?;
    check_eq(composite.as_element().get_attribute("in2"), Some("blur".into()))?;
    check_eq(composite.as_element().get_attribute("operator"), Some("in".into()))
}

/// Every `CompositeOperator` variant writes its exact SVG keyword.
#[wasm_bindgen_test]
fn should_write_every_composite_operator_keyword() -> Result<(), String> {
    let svg = make_svg("filter-composite-operators");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fco").map_err(|e| e.to_string())?;
    let cases = [
        (CompositeOperator::Over, "over"),
        (CompositeOperator::In, "in"),
        (CompositeOperator::Out, "out"),
        (CompositeOperator::Atop, "atop"),
        (CompositeOperator::Xor, "xor"),
        (CompositeOperator::Lighter, "lighter"),
        (CompositeOperator::Arithmetic, "arithmetic"),
    ];
    for (operator, expected) in cases {
        let composite = filter.composite("blur", operator).map_err(|e| e.to_string())?;
        check_eq(composite.as_element().get_attribute("operator"), Some(expected.into()))?;
    }
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// blend primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `blend` appends a `<feBlend>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_blend_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-blend-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbl").map_err(|e| e.to_string())?;
    filter.blend("colour", BlendMode::Multiply).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feBlend"`.
#[wasm_bindgen_test]
fn should_create_fe_blend_element() -> Result<(), String> {
    let svg = make_svg("filter-blend-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fblt").map_err(|e| e.to_string())?;
    let blend = filter.blend("colour", BlendMode::Multiply).map_err(|e| e.to_string())?;
    check_eq(blend.as_element().tag_name(), "feBlend".to_owned())
}

/// `blend` writes the `in2` and `mode` attributes.
#[wasm_bindgen_test]
fn should_set_in2_and_mode() -> Result<(), String> {
    let svg = make_svg("filter-blend-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fbla").map_err(|e| e.to_string())?;
    let blend = filter.blend("colour", BlendMode::Screen).map_err(|e| e.to_string())?;
    check_eq(blend.as_element().get_attribute("in2"), Some("colour".into()))?;
    check_eq(blend.as_element().get_attribute("mode"), Some("screen".into()))
}

/// Every `BlendMode` variant writes its exact SVG/CSS keyword.
#[wasm_bindgen_test]
fn should_write_every_blend_mode_keyword() -> Result<(), String> {
    let svg = make_svg("filter-blend-modes");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fblm").map_err(|e| e.to_string())?;
    let cases = [
        (BlendMode::Normal, "normal"),
        (BlendMode::Multiply, "multiply"),
        (BlendMode::Screen, "screen"),
        (BlendMode::Darken, "darken"),
        (BlendMode::Lighten, "lighten"),
        (BlendMode::Overlay, "overlay"),
        (BlendMode::ColorDodge, "color-dodge"),
        (BlendMode::ColorBurn, "color-burn"),
        (BlendMode::HardLight, "hard-light"),
        (BlendMode::SoftLight, "soft-light"),
        (BlendMode::Difference, "difference"),
        (BlendMode::Exclusion, "exclusion"),
        (BlendMode::Hue, "hue"),
        (BlendMode::Saturation, "saturation"),
        (BlendMode::Color, "color"),
        (BlendMode::Luminosity, "luminosity"),
    ];
    for (mode, expected) in cases {
        let blend = filter.blend("colour", mode).map_err(|e| e.to_string())?;
        check_eq(blend.as_element().get_attribute("mode"), Some(expected.into()))?;
    }
    Ok(())
}

/// The generic `SvgNode::set_attr` escape hatch works on a `blend` node the same as on every other primitive, for
/// attributes like `in`/`result` not wrapped by a named parameter.
#[wasm_bindgen_test]
fn should_set_result_on_blend_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-blend-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fblr").map_err(|e| e.to_string())?;
    let blend = filter.blend("colour", BlendMode::Multiply).map_err(|e| e.to_string())?;
    blend.set_attr("result", "tinted").map_err(|e| e.to_string())?;
    check_eq(blend.as_element().get_attribute("result"), Some("tinted".into()))
}

/// `blend` composes with `flood` and a final `composite(In)` into a working, alpha-preserving tint filter: flood a
/// colour, blend it over the source graphic, then clip the result back to the source's own alpha coverage — the
/// example from `SvgFilter::blend`'s doc comment. The final `composite` step is not cosmetic: without it, `flood`'s
/// opaque colour would leak through wherever the source graphic was itself transparent, since `blend` alone only
/// changes colour, not alpha.
#[wasm_bindgen_test]
fn should_build_tint_filter_chain() -> Result<(), String> {
    let svg = make_svg("filter-blend-tint-chain");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("tint", |f| {
            f.flood("steelblue", 1.0)?.set_attr("result", "colour")?;
            f.blend("colour", BlendMode::Multiply)?
                .set_attrs([("in", "SourceGraphic"), ("result", "tinted")])?;
            f.composite("SourceGraphic", CompositeOperator::In)?.set_attr("in", "tinted")?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 3)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// drop_shadow primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `drop_shadow` appends a `<feDropShadow>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_drop_shadow_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fds").map_err(|e| e.to_string())?;
    filter.drop_shadow(4.0, 4.0, 4.0, "black", 0.5).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feDropShadow"`.
#[wasm_bindgen_test]
fn should_create_fe_drop_shadow_element() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdst").map_err(|e| e.to_string())?;
    let shadow = filter.drop_shadow(4.0, 4.0, 4.0, "black", 0.5).map_err(|e| e.to_string())?;
    check_eq(shadow.as_element().tag_name(), "feDropShadow".to_owned())
}

/// `drop_shadow` writes `stdDeviation`, `dx`, `dy`, `flood-color`, and `flood-opacity`.
#[wasm_bindgen_test]
fn should_set_drop_shadow_attributes() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow-attrs");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdsa").map_err(|e| e.to_string())?;
    let shadow = filter.drop_shadow(3.5, 6.0, 8.0, "crimson", 0.65).map_err(|e| e.to_string())?;
    let el = shadow.as_element();
    check_eq(el.get_attribute("stdDeviation"), Some("3.5".into()))?;
    check_eq(el.get_attribute("dx"), Some("6".into()))?;
    check_eq(el.get_attribute("dy"), Some("8".into()))?;
    check_eq(el.get_attribute("flood-color"), Some("crimson".into()))?;
    check_eq(el.get_attribute("flood-opacity"), Some("0.65".into()))
}

/// A `<filter>` containing only `drop_shadow` is already a complete shadow effect — the primitive's own SVG
/// semantics merge the original graphic back on top, so this filter needs exactly one child, not a further
/// `merge` call.
#[wasm_bindgen_test]
fn should_need_no_further_merge_after_drop_shadow() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow-standalone");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("standalone-shadow", |f| {
            f.drop_shadow(4.0, 4.0, 4.0, "black", 0.5)?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The generic `SvgNode::set_attr` escape hatch works on a `drop_shadow` node the same as on every other
/// primitive, for attributes like `in`/`result` not wrapped by a named parameter.
#[wasm_bindgen_test]
fn should_set_result_on_drop_shadow_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fdsr").map_err(|e| e.to_string())?;
    let shadow = filter.drop_shadow(4.0, 4.0, 4.0, "black", 0.5).map_err(|e| e.to_string())?;
    shadow.set_attr("result", "shadow").map_err(|e| e.to_string())?;
    check_eq(shadow.as_element().get_attribute("result"), Some("shadow".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// A full blur + offset + merge drop-shadow chain
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// The three primitives compose into a working drop-shadow filter: blur the source alpha, offset it, then merge
/// it underneath the original graphic.
#[wasm_bindgen_test]
fn should_build_drop_shadow_filter_chain() -> Result<(), String> {
    let svg = make_svg("filter-drop-shadow");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("shadow", |f| {
            f.gaussian_blur(4.0)?.set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
            f.offset(4.0, 4.0)?.set_attrs([("in", "blur"), ("result", "offset-blur")])?;
            f.merge(&["offset-blur", "SourceGraphic"])?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 3)
}

/// Adding `flood` + `composite` composes into a working *tinted* drop-shadow filter: blur the source alpha,
/// composite a flood colour into the blurred mask, offset it, then merge it underneath the original graphic.
#[wasm_bindgen_test]
fn should_build_tinted_drop_shadow_filter_chain() -> Result<(), String> {
    let svg = make_svg("filter-tinted-drop-shadow");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs
        .build_filter("tinted-shadow", |f| {
            f.gaussian_blur(4.0)?.set_attrs([("in", "SourceAlpha"), ("result", "blur")])?;
            f.flood("black", 0.5)?.set_attr("result", "colour")?;
            f.composite("blur", CompositeOperator::In)?
                .set_attrs([("in", "colour"), ("result", "tinted")])?;
            f.offset(4.0, 4.0)?.set_attrs([("in", "tinted"), ("result", "offset-shadow")])?;
            f.merge(&["offset-shadow", "SourceGraphic"])?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 5)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// color_matrix primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `color_matrix` appends a `<feColorMatrix>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_color_matrix_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcm").map_err(|e| e.to_string())?;
    filter.color_matrix(ColorMatrixType::Saturate(0.0)).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feColorMatrix"`.
#[wasm_bindgen_test]
fn should_create_fe_color_matrix_element() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmt").map_err(|e| e.to_string())?;
    let cm = filter.color_matrix(ColorMatrixType::Saturate(0.0)).map_err(|e| e.to_string())?;
    check_eq(cm.as_element().tag_name(), "feColorMatrix".to_owned())
}

/// `ColorMatrixType::Saturate` writes `type="saturate"` and `values` as the single number.
#[wasm_bindgen_test]
fn should_set_saturate_type_and_values() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-saturate");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcms").map_err(|e| e.to_string())?;
    let cm = filter
        .color_matrix(ColorMatrixType::Saturate(0.25))
        .map_err(|e| e.to_string())?;
    check_eq(cm.as_element().get_attribute("type"), Some("saturate".into()))?;
    check_eq(cm.as_element().get_attribute("values"), Some("0.25".into()))
}

/// `ColorMatrixType::HueRotate` writes `type="hueRotate"` and `values` as the single number (degrees).
#[wasm_bindgen_test]
fn should_set_hue_rotate_type_and_values() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-hue");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmh").map_err(|e| e.to_string())?;
    let cm = filter
        .color_matrix(ColorMatrixType::HueRotate(90.0))
        .map_err(|e| e.to_string())?;
    check_eq(cm.as_element().get_attribute("type"), Some("hueRotate".into()))?;
    check_eq(cm.as_element().get_attribute("values"), Some("90".into()))
}

/// `ColorMatrixType::LuminanceToAlpha` writes `type="luminanceToAlpha"` and omits `values` entirely, since the
/// SVG spec defines `values` as not applicable for this type.
#[wasm_bindgen_test]
fn should_set_luminance_to_alpha_type_and_omit_values() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-luminance");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcml").map_err(|e| e.to_string())?;
    let cm = filter
        .color_matrix(ColorMatrixType::LuminanceToAlpha)
        .map_err(|e| e.to_string())?;
    check_eq(cm.as_element().get_attribute("type"), Some("luminanceToAlpha".into()))?;
    check_eq(cm.as_element().get_attribute("values"), None)
}

/// `ColorMatrixType::Matrix` writes `type="matrix"` and `values` as all 20 numbers, space-separated, in order.
#[wasm_bindgen_test]
fn should_set_matrix_type_and_all_twenty_values() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-matrix");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmm").map_err(|e| e.to_string())?;
    #[rustfmt::skip]
    let identity: [f64; 20] = [
        1.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0, 0.0,
    ];
    let cm = filter
        .color_matrix(ColorMatrixType::Matrix(identity))
        .map_err(|e| e.to_string())?;
    check_eq(cm.as_element().get_attribute("type"), Some("matrix".into()))?;
    check_eq(
        cm.as_element().get_attribute("values"),
        Some("1 0 0 0 0 0 1 0 0 0 0 0 1 0 0 0 0 0 1 0".into()),
    )
}

/// The generic `SvgNode::set_attr` escape hatch works on a `color_matrix` node the same as on every other
/// primitive, for attributes like `in`/`result` not wrapped by a named parameter.
#[wasm_bindgen_test]
fn should_set_result_on_color_matrix_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-color-matrix-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcmr").map_err(|e| e.to_string())?;
    let cm = filter.color_matrix(ColorMatrixType::Saturate(0.0)).map_err(|e| e.to_string())?;
    cm.set_attr("result", "grey").map_err(|e| e.to_string())?;
    check_eq(cm.as_element().get_attribute("result"), Some("grey".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// component_transfer primitive
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `component_transfer` appends a `<feComponentTransfer>` child to the `<filter>` element.
#[wasm_bindgen_test]
fn should_add_component_transfer_to_filter() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-child");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fct").map_err(|e| e.to_string())?;
    filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Linear { slope: 0.5, intercept: 0.0 })])
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().child_element_count(), 1)
}

/// The appended child has tag name `"feComponentTransfer"`.
#[wasm_bindgen_test]
fn should_create_fe_component_transfer_element() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-tag");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctt").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Linear { slope: 0.5, intercept: 0.0 })])
        .map_err(|e| e.to_string())?;
    check_eq(ct.as_element().tag_name(), "feComponentTransfer".to_owned())
}

/// `component_transfer` creates exactly one `<feFuncX>` child per entry in `funcs`, in the order given, with a
/// tag name matching the requested `Channel`.
#[wasm_bindgen_test]
fn should_create_one_fe_func_child_per_channel_in_order() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-children");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctc").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[
            (Channel::Alpha, TransferFunction::Identity),
            (Channel::Red, TransferFunction::Identity),
            (Channel::Green, TransferFunction::Identity),
            (Channel::Blue, TransferFunction::Identity),
        ])
        .map_err(|e| e.to_string())?;
    let el = ct.as_element();
    check_eq(el.child_element_count(), 4)?;
    let children = el.children();
    check_eq(children.item(0).map(|c| c.tag_name()), Some("feFuncA".to_owned()))?;
    check_eq(children.item(1).map(|c| c.tag_name()), Some("feFuncR".to_owned()))?;
    check_eq(children.item(2).map(|c| c.tag_name()), Some("feFuncG".to_owned()))?;
    check_eq(children.item(3).map(|c| c.tag_name()), Some("feFuncB".to_owned()))
}

/// Naming the same `Channel` twice in `funcs` is not deduplicated: both `<feFuncX>` children are created, in the
/// order given. (Per the SVG spec only the last one has any rendered effect, but that is a browser-side rendering
/// rule, not something this crate enforces or hides — see `Channel::Alpha`'s doc comment for the analogous
/// f(0) > 0 caveat.)
#[wasm_bindgen_test]
fn should_create_both_children_for_a_duplicated_channel_in_order() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-duplicate-channel");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctdup").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[
            (Channel::Red, TransferFunction::Linear { slope: 1.0, intercept: 0.0 }),
            (Channel::Red, TransferFunction::Linear { slope: 0.5, intercept: 0.1 }),
        ])
        .map_err(|e| e.to_string())?;
    let el = ct.as_element();
    check_eq(el.child_element_count(), 2)?;
    let children = el.children();
    let first = children.item(0).ok_or("expected a first feFuncR child")?;
    let second = children.item(1).ok_or("expected a second feFuncR child")?;
    check_eq(first.tag_name(), "feFuncR".to_owned())?;
    check_eq(second.tag_name(), "feFuncR".to_owned())?;
    check_eq(first.get_attribute("slope"), Some("1".into()))?;
    check_eq(first.get_attribute("intercept"), Some("0".into()))?;
    check_eq(second.get_attribute("slope"), Some("0.5".into()))?;
    check_eq(second.get_attribute("intercept"), Some("0.1".into()))
}

/// A channel not named in `funcs` gets no `<feFuncX>` child at all.
#[wasm_bindgen_test]
fn should_omit_fe_func_child_for_unmentioned_channel() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-omit");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcto").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Identity)])
        .map_err(|e| e.to_string())?;
    check_eq(ct.as_element().child_element_count(), 1)
}

/// `TransferFunction::Identity` writes `type="identity"` and no other attribute.
#[wasm_bindgen_test]
fn should_set_identity_type_and_no_other_attribute() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-identity");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcti").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Identity)])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncA> child")?;
    check_eq(func.get_attribute("type"), Some("identity".into()))?;
    check_eq(func.get_attribute("tableValues"), None)?;
    check_eq(func.get_attribute("slope"), None)
}

/// `TransferFunction::Table` writes `type="table"` and `tableValues` as the space-separated values, in order.
#[wasm_bindgen_test]
fn should_set_table_type_and_values() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-table");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctt2").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Red, TransferFunction::Table(vec![0.0, 0.5, 1.0]))])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncR> child")?;
    check_eq(func.get_attribute("type"), Some("table".into()))?;
    check_eq(func.get_attribute("tableValues"), Some("0 0.5 1".into()))
}

/// `TransferFunction::Table` with an empty list writes `type="table"` and an empty `tableValues` — valid SVG syntax
/// (per spec, equivalent to identity), not a crate-level error.
#[wasm_bindgen_test]
fn should_accept_empty_table_values() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-table-empty");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcte").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Red, TransferFunction::Table(vec![]))])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncR> child")?;
    check_eq(func.get_attribute("type"), Some("table".into()))?;
    check_eq(func.get_attribute("tableValues"), Some("".into()))
}

/// A `TransferFunction::Table` with exactly one value has no defined SVG semantics (the `n+1`-values-describe-`n`-
/// regions formula leaves zero regions to interpolate across) and is rejected before reaching the DOM.
#[wasm_bindgen_test]
fn should_reject_single_value_table() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-table-single");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcts1").map_err(|e| e.to_string())?;
    let result = filter.component_transfer(&[(Channel::Red, TransferFunction::Table(vec![0.5]))]);
    check(
        matches!(result, Err(Error::InvalidTransferTable)),
        "expected InvalidTransferTable error for a single-value Table",
    )
}

/// Two equal values is the documented, portable way to write a constant `Table` transfer function.
#[wasm_bindgen_test]
fn should_accept_two_equal_table_values_as_constant_workaround() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-table-constant");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctc").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Red, TransferFunction::Table(vec![0.5, 0.5]))])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncR> child")?;
    check_eq(func.get_attribute("tableValues"), Some("0.5 0.5".into()))
}

/// Unlike `Table`, `TransferFunction::Discrete` with a single value is well-defined by the SVG "discrete" stepping
/// formula (every input maps to the one entry), so it is accepted rather than rejected.
#[wasm_bindgen_test]
fn should_accept_single_value_discrete() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-discrete-single");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fcds1").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Red, TransferFunction::Discrete(vec![0.5]))])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncR> child")?;
    check_eq(func.get_attribute("tableValues"), Some("0.5".into()))
}

/// `TransferFunction::Discrete` writes `type="discrete"` and `tableValues` as the space-separated values.
#[wasm_bindgen_test]
fn should_set_discrete_type_and_values() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-discrete");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctd").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Green, TransferFunction::Discrete(vec![0.0, 0.25, 0.5, 0.75, 1.0]))])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncG> child")?;
    check_eq(func.get_attribute("type"), Some("discrete".into()))?;
    check_eq(func.get_attribute("tableValues"), Some("0 0.25 0.5 0.75 1".into()))
}

/// `TransferFunction::Linear` writes `type="linear"`, `slope`, and `intercept`.
#[wasm_bindgen_test]
fn should_set_linear_type_slope_and_intercept() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-linear");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctl").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Linear { slope: 0.6, intercept: 0.1 })])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncA> child")?;
    check_eq(func.get_attribute("type"), Some("linear".into()))?;
    check_eq(func.get_attribute("slope"), Some("0.6".into()))?;
    check_eq(func.get_attribute("intercept"), Some("0.1".into()))
}

/// `TransferFunction::Gamma` writes `type="gamma"`, `amplitude`, `exponent`, and `offset`.
#[wasm_bindgen_test]
fn should_set_gamma_type_amplitude_exponent_and_offset() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-gamma");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctg").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(
            Channel::Red,
            TransferFunction::Gamma {
                amplitude: 1.0,
                exponent: 0.45,
                offset: 0.0,
            },
        )])
        .map_err(|e| e.to_string())?;
    let func = ct.as_element().first_element_child().ok_or("expected a <feFuncR> child")?;
    check_eq(func.get_attribute("type"), Some("gamma".into()))?;
    check_eq(func.get_attribute("amplitude"), Some("1".into()))?;
    check_eq(func.get_attribute("exponent"), Some("0.45".into()))?;
    check_eq(func.get_attribute("offset"), Some("0".into()))
}

/// The generic `SvgNode::set_attr` escape hatch works on a `component_transfer` node the same as on every other
/// primitive, for attributes like `in`/`result` not wrapped by a named parameter.
#[wasm_bindgen_test]
fn should_set_result_on_component_transfer_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-component-transfer-result");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fctr").map_err(|e| e.to_string())?;
    let ct = filter
        .component_transfer(&[(Channel::Alpha, TransferFunction::Identity)])
        .map_err(|e| e.to_string())?;
    ct.set_attr("result", "faded").map_err(|e| e.to_string())?;
    check_eq(ct.as_element().get_attribute("result"), Some("faded".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Region attributes with explicit SVG units via the generic escape hatch
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_attr` remains available when percentage or other explicit length syntax cannot be represented by the numeric
/// named setters.
#[wasm_bindgen_test]
fn should_set_filter_region_with_explicit_units_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-region");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("region").map_err(|e| e.to_string())?;
    filter.set_attr("x", "-20%").map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("x"), Some("-20%".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Id validation
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// An invalid id is rejected with `Error::InvalidFilterId` before any DOM call.
#[wasm_bindgen_test]
fn should_reject_invalid_filter_id() -> Result<(), String> {
    let svg = make_svg("filter-invalid-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let result = defs.filter("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidFilterId(_))),
        "expected InvalidFilterId error for an id containing '(' and ')'",
    )
}

/// `set_id` updates both the DOM attribute and the cached value.
#[wasm_bindgen_test]
fn should_set_filter_id_via_set_id() -> Result<(), String> {
    let svg = make_svg("filter-set-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut filter = defs.filter("old-id").map_err(|e| e.to_string())?;
    filter.set_id("new-id").map_err(|e| e.to_string())?;
    check_eq(filter.id(), "new-id")?;
    check_eq(filter.as_element().get_attribute("id"), Some("new-id".into()))
}

/// Renaming to a longer id still round-trips correctly (exercises the cached `url(#id)` reference `String`
/// growing past whatever capacity the previous id left it with).
#[wasm_bindgen_test]
fn should_set_filter_id_longer_than_previous() -> Result<(), String> {
    let svg = make_svg("filter-set-id-longer");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let mut filter = defs.filter("a").map_err(|e| e.to_string())?;
    filter.set_id("a-much-longer-replacement-id").map_err(|e| e.to_string())?;
    check_eq(filter.id(), "a-much-longer-replacement-id")?;
    check_eq(
        filter.as_element().get_attribute("id"),
        Some("a-much-longer-replacement-id".into()),
    )
}

/// `set_attr("id", ...)` is rejected to protect the cached id.
#[wasm_bindgen_test]
fn should_reject_set_attr_id() -> Result<(), String> {
    let svg = make_svg("filter-reserved-id");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("safe").map_err(|e| e.to_string())?;
    let result = filter.set_attr("id", "injected");
    check(
        matches!(result, Err(Error::ReservedAttribute("id"))),
        "expected ReservedAttribute error for set_attr(\"id\", ...)",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// SvgNode::set_filter / set_filter_ref / remove_filter
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_filter` writes `filter="url(#id)"`.
#[wasm_bindgen_test]
fn should_set_filter_attribute() -> Result<(), String> {
    let svg = make_svg("filter-apply");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    defs.filter("my-blur").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_filter("my-blur").map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("filter"), Some("url(#my-blur)".into()))
}

/// `set_filter_ref` produces the same result as `set_filter` using the id from the handle.
#[wasm_bindgen_test]
fn should_set_filter_ref() -> Result<(), String> {
    let svg = make_svg("filter-apply-ref");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("ref-blur").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_filter_ref(&filter).map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("filter"), Some("url(#ref-blur)".into()))
}

/// `set_filter` with an invalid id returns `Error::InvalidFilterId`.
#[wasm_bindgen_test]
fn should_reject_invalid_set_filter_id() -> Result<(), String> {
    let svg = make_svg("filter-invalid-apply");
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    let result = rect.set_filter("url(#bad)");
    check(
        matches!(result, Err(Error::InvalidFilterId(_))),
        "expected InvalidFilterId from set_filter with bad id",
    )
}

/// `remove_filter` removes the `filter` attribute.
#[wasm_bindgen_test]
fn should_remove_filter_attribute() -> Result<(), String> {
    let svg = make_svg("filter-remove");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("removable").map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(100.0, 100.0)).map_err(|e| e.to_string())?;
    rect.set_filter_ref(&filter).map_err(|e| e.to_string())?;
    rect.remove_filter().map_err(|e| e.to_string())?;
    check_eq(rect.as_element().get_attribute("filter"), None)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Filter region and coordinate-space attributes
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_x` writes the `x` attribute.
#[wasm_bindgen_test]
fn should_set_filter_x() -> Result<(), String> {
    let svg = make_svg("filter-x");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fx").map_err(|e| e.to_string())?;
    filter.set_x(-0.2).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("x"), Some("-0.2".into()))
}

/// `set_y` writes the `y` attribute.
#[wasm_bindgen_test]
fn should_set_filter_y() -> Result<(), String> {
    let svg = make_svg("filter-y");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fy").map_err(|e| e.to_string())?;
    filter.set_y(-0.2).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("y"), Some("-0.2".into()))
}

/// `set_width` writes the `width` attribute.
#[wasm_bindgen_test]
fn should_set_filter_width() -> Result<(), String> {
    let svg = make_svg("filter-width");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fw").map_err(|e| e.to_string())?;
    filter.set_width(1.4).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("width"), Some("1.4".into()))
}

/// `set_height` writes the `height` attribute.
#[wasm_bindgen_test]
fn should_set_filter_height() -> Result<(), String> {
    let svg = make_svg("filter-height");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fh").map_err(|e| e.to_string())?;
    filter.set_height(1.4).map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("height"), Some("1.4".into()))
}

/// `set_filter_units(UserSpaceOnUse)` writes `filterUnits="userSpaceOnUse"`.
#[wasm_bindgen_test]
fn should_set_filter_units_user_space() -> Result<(), String> {
    let svg = make_svg("filter-units-user");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fu-user").map_err(|e| e.to_string())?;
    filter
        .set_filter_units(FilterUnits::UserSpaceOnUse)
        .map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("filterUnits"), Some("userSpaceOnUse".into()))
}

/// `set_filter_units(ObjectBoundingBox)` writes `filterUnits="objectBoundingBox"`.
#[wasm_bindgen_test]
fn should_set_filter_units_object_bounding_box() -> Result<(), String> {
    let svg = make_svg("filter-units-obb");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("fu-obb").map_err(|e| e.to_string())?;
    filter
        .set_filter_units(FilterUnits::ObjectBoundingBox)
        .map_err(|e| e.to_string())?;
    check_eq(
        filter.as_element().get_attribute("filterUnits"),
        Some("objectBoundingBox".into()),
    )
}

/// `set_primitive_units` writes the `primitiveUnits` attribute.
#[wasm_bindgen_test]
fn should_set_primitive_units() -> Result<(), String> {
    let svg = make_svg("filter-primitive-units");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("pu").map_err(|e| e.to_string())?;
    filter
        .set_primitive_units(FilterUnits::ObjectBoundingBox)
        .map_err(|e| e.to_string())?;
    check_eq(
        filter.as_element().get_attribute("primitiveUnits"),
        Some("objectBoundingBox".into()),
    )
}
