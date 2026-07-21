use crate::common::{self, *};
use svg_dom::{Error, root::utils::Point};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// build_defs / build_marker — deferred-append variants
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `build_defs` appends `<defs>` to the SVG root after the closure succeeds.
#[wasm_bindgen_test]
fn should_build_defs_and_append_on_success() -> Result<(), String> {
    let svg = make_svg("build-defs-ok");
    let defs = svg.build_defs(|_| Ok(())).map_err(|e| e.to_string())?;
    // The defs element must now be a child of the SVG.
    let parent = defs
        .as_element()
        .parent_element()
        .ok_or("defs has no parent after build_defs")?;
    common::check_eq(parent.tag_name(), "svg".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `build_defs` returns `Err` when the closure does, and the `<defs>` element is NOT appended.
#[wasm_bindgen_test]
fn should_not_append_defs_when_build_closure_fails() -> Result<(), String> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let svg = make_svg("build-defs-err");
    let captured: Rc<RefCell<Option<web_sys::SvgElement>>> = Rc::new(RefCell::new(None));
    let cap2 = captured.clone();

    let result = svg.build_defs(move |defs| {
        *cap2.borrow_mut() = Some(defs.as_element().clone());
        Err(Error::Dom("deliberate failure".into()))
    });

    common::check(result.is_err(), "build_defs must return Err when closure fails")?;

    let borrow = captured.borrow();
    let el = borrow.as_ref().ok_or("closure was never called")?;
    common::check(
        el.parent_element().is_none(),
        "defs must not be appended to SVG when closure fails",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `build_marker` appends `<marker>` to `<defs>` after the closure succeeds, with all attributes set.
#[wasm_bindgen_test]
fn should_build_marker_and_append_on_success() -> Result<(), String> {
    let svg = make_svg("build-marker-ok");
    let defs = svg.defs().map_err(|e| e.to_string())?;
    let marker = defs
        .build_marker("bm-arrow", |m| {
            m.set_ref_x(10.0)?;
            m.set_ref_y(3.5)?;
            m.set_marker_width(10.0)?;
            m.set_marker_height(7.0)?;
            m.set_orient("auto")?;
            m.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
            Ok(())
        })
        .map_err(|e| e.to_string())?;

    // Marker must be inside defs.
    let parent = marker.as_element().parent_element().ok_or("marker has no parent")?;
    common::check_eq(parent.tag_name(), "defs".to_owned())?;

    // All attributes must be present.
    let el = marker.as_element();
    common::check_eq(el.get_attribute("id"), Some("bm-arrow".into()))?;
    common::check_eq(el.get_attribute("refX"), Some("10".into()))?;
    common::check_eq(el.get_attribute("refY"), Some("3.5".into()))?;
    common::check_eq(el.get_attribute("markerWidth"), Some("10".into()))?;
    common::check_eq(el.get_attribute("markerHeight"), Some("7".into()))?;
    common::check_eq(el.get_attribute("orient"), Some("auto".into()))?;

    // Polygon child must be inside the marker.
    common::check_eq(el.child_element_count(), 1_u32)?;
    let child = el.first_element_child().ok_or("marker has no child")?;
    common::check_eq(child.tag_name(), "polygon".to_owned())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `build_marker` returns `Err` when the closure does, and the `<marker>` element is NOT appended to `<defs>`.
#[wasm_bindgen_test]
fn should_not_append_marker_when_build_closure_fails() -> Result<(), String> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let svg = make_svg("build-marker-err");
    let defs = svg.defs().map_err(|e| e.to_string())?;

    let captured: Rc<RefCell<Option<web_sys::SvgElement>>> = Rc::new(RefCell::new(None));
    let cap2 = captured.clone();

    let result = defs.build_marker("bm-err", move |marker| {
        *cap2.borrow_mut() = Some(marker.as_element().clone());
        Err(Error::Dom("deliberate failure".into()))
    });

    common::check(result.is_err(), "build_marker must return Err when closure fails")?;

    let borrow = captured.borrow();
    let el = borrow.as_ref().ok_or("closure was never called")?;
    common::check(
        el.parent_element().is_none(),
        "marker must not be appended to defs when closure fails",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
/// `build_defs` nesting `build_marker` — the fully built subtree lands in the SVG in one append.
#[wasm_bindgen_test]
fn should_build_nested_defs_and_marker_atomically() -> Result<(), String> {
    let svg = make_svg("build-nested");
    svg.build_defs(|defs| {
        defs.build_marker("nested-arrow", |m| {
            m.set_ref_x(10.0)?;
            m.set_ref_y(3.5)?;
            m.set_orient("auto")?;
            m.polygon(&[Point::new(0.0, 0.0), Point::new(10.0, 3.5), Point::new(0.0, 7.0)])?;
            Ok(())
        })?;
        Ok(())
    })
    .map_err(|e| e.to_string())?;

    // Verify the structure through the live SVG via the common div we can query.
    let doc = web_sys::window().unwrap().document().unwrap();
    let marker_el = doc
        .query_selector("#build-nested svg defs marker")
        .map_err(|_| "query_selector failed".to_owned())?
        .ok_or("marker not found in live DOM")?;

    common::check_eq(marker_el.get_attribute("id"), Some("nested-arrow".into()))?;
    common::check_eq(marker_el.child_element_count(), 1_u32)
}
