use crate::{common, helpers::make_svg};
use svg_dom::root::utils::{Point, Size};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_attr / attr
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// An attribute written with `set_attr` is immediately readable back via `attr`.
#[wasm_bindgen_test]
fn should_read_new_value_after_calling_set_attr() -> Result<(), String> {
    let rect = make_svg("node-set-attr")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_attr("opacity", "0.5").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("opacity"), Some("0.5".into()))
}

/// `attr` returns `None` for an attribute that has never been set.
#[wasm_bindgen_test]
fn should_return_none_for_absent_attribute() -> Result<(), String> {
    let rect = make_svg("node-attr-absent")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("nonexistent"), None)
}

/// `set_attrs` writes each supplied attribute and all values are immediately readable back.
#[wasm_bindgen_test]
fn should_set_multiple_attributes_in_one_call() -> Result<(), String> {
    let rect = make_svg("node-set-attrs")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;

    rect.set_attrs([("fill", "steelblue"), ("stroke", "white"), ("stroke-width", "2")])
        .map_err(|e| e.to_string())?;

    common::check_eq(rect.attr("fill"), Some("steelblue".into()))?;
    common::check_eq(rect.attr("stroke"), Some("white".into()))?;
    common::check_eq(rect.attr("stroke-width"), Some("2".into()))
}

/// `set_attrs` accepts owned String values as well as string literals.
#[wasm_bindgen_test]
fn should_set_multiple_owned_string_attributes_in_one_call() -> Result<(), String> {
    let line = make_svg("node-set-attrs-owned")
        .line(Point::origin(), Point::new(10.0, 20.0))
        .map_err(|e| e.to_string())?;

    line.set_attrs([
        ("x1", 1.0_f64.to_string()),
        ("y1", 2.0_f64.to_string()),
        ("x2", 3.0_f64.to_string()),
        ("y2", 4.0_f64.to_string()),
    ])
    .map_err(|e| e.to_string())?;

    common::check_eq(line.attr("x1"), Some("1".into()))?;
    common::check_eq(line.attr("y1"), Some("2".into()))?;
    common::check_eq(line.attr("x2"), Some("3".into()))?;
    common::check_eq(line.attr("y2"), Some("4".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_attr_if_changed
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_attr_if_changed` writes an attribute that was previously absent and the new value is readable back.
#[wasm_bindgen_test]
fn should_write_absent_attribute_with_set_attr_if_changed() -> Result<(), String> {
    let rect = make_svg("node-set-attr-if-changed-absent")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_attr_if_changed("style", "cursor:grab").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:grab".into()))
}

/// `set_attr_if_changed` updates the attribute when the new value differs from the current one.
#[wasm_bindgen_test]
fn should_update_attribute_when_value_differs() -> Result<(), String> {
    let rect = make_svg("node-set-attr-if-changed-update")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_attr_if_changed("style", "cursor:grab").map_err(|e| e.to_string())?;
    rect.set_attr_if_changed("style", "cursor:grabbing")
        .map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("style"), Some("cursor:grabbing".into()))
}

/// Setting the same value again via `set_attr_if_changed` leaves the attribute unchanged and reports success.
#[wasm_bindgen_test]
fn should_leave_attribute_unchanged_when_value_matches() -> Result<(), String> {
    let rect = make_svg("node-set-attr-if-changed-noop")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_attr_if_changed("opacity", "0.5").map_err(|e| e.to_string())?;
    rect.set_attr_if_changed("opacity", "0.5").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("opacity"), Some("0.5".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// set_attr_display
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_attr_display` formats a numeric value through the scratch buffer and writes it as the named attribute.
#[wasm_bindgen_test]
fn should_write_numeric_attribute_via_set_attr_display() -> Result<(), String> {
    let circle = make_svg("node-set-attr-display")
        .circle(Point::new(50.0, 50.0), 20.0)
        .map_err(|e| e.to_string())?;
    let mut buf = String::new();
    circle
        .set_attr_display(&mut buf, "stroke-width", 2.5)
        .map_err(|e| e.to_string())?;
    common::check_eq(circle.attr("stroke-width"), Some("2.5".into()))
}

/// The same scratch buffer can be reused across `set_attr_display` calls and the latest value wins.
#[wasm_bindgen_test]
fn should_reuse_buffer_across_set_attr_display_calls() -> Result<(), String> {
    let circle = make_svg("node-set-attr-display-reuse")
        .circle(Point::new(50.0, 50.0), 20.0)
        .map_err(|e| e.to_string())?;
    let mut buf = String::new();
    circle
        .set_attr_display(&mut buf, "stroke-width", 1)
        .map_err(|e| e.to_string())?;
    circle
        .set_attr_display(&mut buf, "stroke-width", 4)
        .map_err(|e| e.to_string())?;
    common::check_eq(circle.attr("stroke-width"), Some("4".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// remove_attr
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// After `remove_attr`, `attr` returns `None` for that attribute.
#[wasm_bindgen_test]
fn should_return_none_after_reading_removed_attribute() -> Result<(), String> {
    let rect = make_svg("node-remove-attr")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_attr("opacity", "0.5").map_err(|e| e.to_string())?;
    rect.remove_attr("opacity").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("opacity"), None)
}

/// Calling `remove_attr` twice on the same attribute is idempotent
#[wasm_bindgen_test]
fn should_succeed_after_calling_remove_attr_twice() -> Result<(), String> {
    let rect = make_svg("node-remove-attr-twice")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_attr("opacity", "0.5").map_err(|e| e.to_string())?;
    rect.remove_attr("opacity").map_err(|e| e.to_string())?;
    rect.remove_attr("opacity").map_err(|e| e.to_string())
}

/// Calling `remove_attr` on a non-existent attribute succeeds without error
#[wasm_bindgen_test]
fn should_successfully_remove_nonexistent_attribute() -> Result<(), String> {
    let rect = make_svg("node-remove-absent")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.remove_attr("nonexistent").map_err(|e| e.to_string())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// add_class / remove_class / toggle_class / has_class
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `has_class` returns `false` for a class that has never been added.
#[wasm_bindgen_test]
fn should_return_false_for_absent_class() -> Result<(), String> {
    let rect = make_svg("node-class-absent")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    common::check_eq(rect.has_class("highlighted"), false)
}

/// `add_class` makes a class visible to `has_class` and to the underlying `class` attribute.
#[wasm_bindgen_test]
fn should_add_class() -> Result<(), String> {
    let rect = make_svg("node-add-class")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("highlighted").map_err(|e| e.to_string())?;
    common::check_eq(rect.has_class("highlighted"), true)?;
    common::check_eq(rect.attr("class"), Some("highlighted".into()))
}

/// Adding the same class twice is idempotent: the class appears only once in the `class` attribute.
#[wasm_bindgen_test]
fn should_add_class_idempotently() -> Result<(), String> {
    let rect = make_svg("node-add-class-twice")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("highlighted").map_err(|e| e.to_string())?;
    rect.add_class("highlighted").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("class"), Some("highlighted".into()))
}

/// `add_class` can be called more than once with different classes; both remain present.
#[wasm_bindgen_test]
fn should_add_multiple_distinct_classes() -> Result<(), String> {
    let rect = make_svg("node-add-multiple-classes")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("highlighted").map_err(|e| e.to_string())?;
    rect.add_class("selected").map_err(|e| e.to_string())?;
    common::check_eq(rect.has_class("highlighted"), true)?;
    common::check_eq(rect.has_class("selected"), true)
}

/// `remove_class` clears a previously added class.
#[wasm_bindgen_test]
fn should_remove_class() -> Result<(), String> {
    let rect = make_svg("node-remove-class")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("highlighted").map_err(|e| e.to_string())?;
    rect.remove_class("highlighted").map_err(|e| e.to_string())?;
    common::check_eq(rect.has_class("highlighted"), false)
}

/// Calling `remove_class` on a class that was never added succeeds without error.
#[wasm_bindgen_test]
fn should_successfully_remove_nonexistent_class() -> Result<(), String> {
    let rect = make_svg("node-remove-absent-class")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.remove_class("nonexistent").map_err(|e| e.to_string())
}

/// `toggle_class` adds an absent class and reports it is now present.
#[wasm_bindgen_test]
fn should_toggle_class_on_when_absent() -> Result<(), String> {
    let rect = make_svg("node-toggle-class-on")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let now_present = rect.toggle_class("selected").map_err(|e| e.to_string())?;
    common::check_eq(now_present, true)?;
    common::check_eq(rect.has_class("selected"), true)
}

/// `toggle_class` removes a present class and reports it is now absent.
#[wasm_bindgen_test]
fn should_toggle_class_off_when_present() -> Result<(), String> {
    let rect = make_svg("node-toggle-class-off")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("selected").map_err(|e| e.to_string())?;
    let now_present = rect.toggle_class("selected").map_err(|e| e.to_string())?;
    common::check_eq(now_present, false)?;
    common::check_eq(rect.has_class("selected"), false)
}

/// `add_class` rejects an empty token (DOM `SyntaxError`), leaving the `class` attribute unchanged.
#[wasm_bindgen_test]
fn should_reject_empty_token_via_add_class() -> Result<(), String> {
    let rect = make_svg("node-add-class-empty")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("kept").map_err(|e| e.to_string())?;
    common::check(rect.add_class("").is_err(), "expected an empty class token to fail")?;
    common::check_eq(rect.attr("class"), Some("kept".into()))
}

/// `add_class` rejects a token containing ASCII whitespace (DOM `InvalidCharacterError`), leaving the `class`
/// attribute unchanged.
#[wasm_bindgen_test]
fn should_reject_whitespace_token_via_add_class() -> Result<(), String> {
    let rect = make_svg("node-add-class-whitespace")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("kept").map_err(|e| e.to_string())?;
    common::check(
        rect.add_class("two classes").is_err(),
        "expected a whitespace-containing class token to fail",
    )?;
    common::check_eq(rect.attr("class"), Some("kept".into()))
}

/// `remove_class` rejects an empty token (DOM `SyntaxError`), leaving the `class` attribute unchanged.
#[wasm_bindgen_test]
fn should_reject_empty_token_via_remove_class() -> Result<(), String> {
    let rect = make_svg("node-remove-class-empty")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("kept").map_err(|e| e.to_string())?;
    common::check(rect.remove_class("").is_err(), "expected an empty class token to fail")?;
    common::check_eq(rect.attr("class"), Some("kept".into()))
}

/// `remove_class` rejects a token containing ASCII whitespace (DOM `InvalidCharacterError`), leaving the `class`
/// attribute unchanged.
#[wasm_bindgen_test]
fn should_reject_whitespace_token_via_remove_class() -> Result<(), String> {
    let rect = make_svg("node-remove-class-whitespace")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("kept").map_err(|e| e.to_string())?;
    common::check(
        rect.remove_class("two classes").is_err(),
        "expected a whitespace-containing class token to fail",
    )?;
    common::check_eq(rect.attr("class"), Some("kept".into()))
}

/// `toggle_class` rejects an empty token (DOM `SyntaxError`), leaving the `class` attribute unchanged.
#[wasm_bindgen_test]
fn should_reject_empty_token_via_toggle_class() -> Result<(), String> {
    let rect = make_svg("node-toggle-class-empty")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("kept").map_err(|e| e.to_string())?;
    common::check(rect.toggle_class("").is_err(), "expected an empty class token to fail")?;
    common::check_eq(rect.attr("class"), Some("kept".into()))
}

/// `toggle_class` rejects a token containing ASCII whitespace (DOM `InvalidCharacterError`), leaving the `class`
/// attribute unchanged.
#[wasm_bindgen_test]
fn should_reject_whitespace_token_via_toggle_class() -> Result<(), String> {
    let rect = make_svg("node-toggle-class-whitespace")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("kept").map_err(|e| e.to_string())?;
    common::check(
        rect.toggle_class("two classes").is_err(),
        "expected a whitespace-containing class token to fail",
    )?;
    common::check_eq(rect.attr("class"), Some("kept".into()))
}

/// `set_class_enabled(class, true)` adds an absent class.
#[wasm_bindgen_test]
fn should_add_class_via_set_class_enabled_true() -> Result<(), String> {
    let rect = make_svg("node-set-class-enabled-add")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_class_enabled("selected", true).map_err(|e| e.to_string())?;
    common::check_eq(rect.has_class("selected"), true)
}

/// `set_class_enabled(class, true)` on an already-present class is idempotent.
#[wasm_bindgen_test]
fn should_leave_class_present_via_set_class_enabled_true_when_already_present() -> Result<(), String> {
    let rect = make_svg("node-set-class-enabled-add-idempotent")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("selected").map_err(|e| e.to_string())?;
    rect.set_class_enabled("selected", true).map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("class"), Some("selected".into()))
}

/// `set_class_enabled(class, false)` removes a present class.
#[wasm_bindgen_test]
fn should_remove_class_via_set_class_enabled_false() -> Result<(), String> {
    let rect = make_svg("node-set-class-enabled-remove")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("selected").map_err(|e| e.to_string())?;
    rect.set_class_enabled("selected", false).map_err(|e| e.to_string())?;
    common::check_eq(rect.has_class("selected"), false)
}

/// `set_class_enabled(class, false)` on an already-absent class is idempotent.
#[wasm_bindgen_test]
fn should_leave_class_absent_via_set_class_enabled_false_when_already_absent() -> Result<(), String> {
    let rect = make_svg("node-set-class-enabled-remove-idempotent")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_class_enabled("selected", false).map_err(|e| e.to_string())?;
    common::check_eq(rect.has_class("selected"), false)
}

/// `set_class_enabled` rejects an empty token (DOM `SyntaxError`), leaving the `class` attribute unchanged.
#[wasm_bindgen_test]
fn should_reject_empty_token_via_set_class_enabled() -> Result<(), String> {
    let rect = make_svg("node-set-class-enabled-empty")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("kept").map_err(|e| e.to_string())?;
    common::check(
        rect.set_class_enabled("", true).is_err(),
        "expected an empty class token to fail",
    )?;
    common::check_eq(rect.attr("class"), Some("kept".into()))
}

/// `set_class_enabled` rejects a token containing ASCII whitespace (DOM `InvalidCharacterError`), leaving the
/// `class` attribute unchanged.
#[wasm_bindgen_test]
fn should_reject_whitespace_token_via_set_class_enabled() -> Result<(), String> {
    let rect = make_svg("node-set-class-enabled-whitespace")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.add_class("kept").map_err(|e| e.to_string())?;
    common::check(
        rect.set_class_enabled("two classes", true).is_err(),
        "expected a whitespace-containing class token to fail",
    )?;
    common::check_eq(rect.attr("class"), Some("kept".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Convenience attribute setters
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_fill` writes the `fill` attribute.
#[wasm_bindgen_test]
fn should_update_fill_attribute_after_calling_set_fill() -> Result<(), String> {
    let rect = make_svg("node-fill")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_fill("steelblue").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("fill"), Some("steelblue".into()))
}

/// `set_stroke` writes the `stroke` attribute.
#[wasm_bindgen_test]
fn should_update_stroke_attribute_after_calling_set_stroke() -> Result<(), String> {
    let rect = make_svg("node-stroke")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.set_stroke("black").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("stroke"), Some("black".into()))
}

/// `set_stroke_width` writes the `stroke-width` attribute.
#[wasm_bindgen_test]
fn should_update_stroke_width_attribute_after_calling_set_stroke_width() -> Result<(), String> {
    let line = make_svg("node-stroke-width")
        .line(Point::origin(), Point::new(100.0, 100.0))
        .map_err(|e| e.to_string())?;
    line.set_stroke_width(3.0).map_err(|e| e.to_string())?;
    common::check_eq(line.attr("stroke-width"), Some("3".into()))
}

/// `set_d` replaces the `d` attribute of a `<path>` element.
#[wasm_bindgen_test]
fn should_update_path_data_after_calling_set_d() -> Result<(), String> {
    let path = make_svg("node-set-d").path("M 0 0 L 50 50").map_err(|e| e.to_string())?;
    path.set_d("M 10 10 Q 50 0 90 10").map_err(|e| e.to_string())?;
    common::check_eq(path.attr("d"), Some("M 10 10 Q 50 0 90 10".into()))
}
