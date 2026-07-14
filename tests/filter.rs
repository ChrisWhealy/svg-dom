mod common;

use common::*;
use svg_dom::{
    Error,
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

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Region attributes via the generic escape hatch
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_attr` on `SvgFilter` covers region attributes not yet wrapped by a named setter (`x`, `y`, `width`,
/// `height`, `filterUnits`, `primitiveUnits`).
#[wasm_bindgen_test]
fn should_set_filter_region_via_generic_escape_hatch() -> Result<(), String> {
    let svg = make_svg("filter-region");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let filter = defs.filter("region").map_err(|e| e.to_string())?;
    filter.set_attr("x", "-20%").map_err(|e| e.to_string())?;
    filter.set_attr("filterUnits", "userSpaceOnUse").map_err(|e| e.to_string())?;
    check_eq(filter.as_element().get_attribute("x"), Some("-20%".into()))?;
    check_eq(filter.as_element().get_attribute("filterUnits"), Some("userSpaceOnUse".into()))
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
