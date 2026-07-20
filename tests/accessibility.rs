mod common;

use common::*;
use svg_dom::{
    Error,
    root::utils::{Point, Size},
};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// `set_title` creates a `<title>` child element.
#[wasm_bindgen_test]
fn should_create_title_element() -> Result<(), String> {
    let svg = make_svg("a11y-title-create");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_title("Close dialog").map_err(|e| e.to_string())?;
    let title = rect.as_element().first_element_child().ok_or("expected a <title> child")?;
    check_eq(title.tag_name(), "title".to_owned())
}

/// `set_desc` creates a `<desc>` child element.
#[wasm_bindgen_test]
fn should_create_desc_element() -> Result<(), String> {
    let svg = make_svg("a11y-desc-create");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_desc("Discards unsaved changes.").map_err(|e| e.to_string())?;
    let desc = rect.as_element().first_element_child().ok_or("expected a <desc> child")?;
    check_eq(desc.tag_name(), "desc".to_owned())
}

/// `title()` reads back the text set via `set_title`.
#[wasm_bindgen_test]
fn should_read_back_title() -> Result<(), String> {
    let svg = make_svg("a11y-title-read");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_title("Close dialog").map_err(|e| e.to_string())?;
    check_eq(rect.title(), Some("Close dialog".to_owned()))
}

/// `desc()` reads back the text set via `set_desc`.
#[wasm_bindgen_test]
fn should_read_back_desc() -> Result<(), String> {
    let svg = make_svg("a11y-desc-read");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_desc("Discards unsaved changes.").map_err(|e| e.to_string())?;
    check_eq(rect.desc(), Some("Discards unsaved changes.".to_owned()))
}

/// `title()` returns `None` when no `<title>` child has been set.
#[wasm_bindgen_test]
fn should_return_none_for_missing_title() -> Result<(), String> {
    let svg = make_svg("a11y-title-missing");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    check_eq(rect.title(), None)
}

/// `desc()` returns `None` when no `<desc>` child has been set.
#[wasm_bindgen_test]
fn should_return_none_for_missing_desc() -> Result<(), String> {
    let svg = make_svg("a11y-desc-missing");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    check_eq(rect.desc(), None)
}

/// Calling `set_title` a second time updates the existing `<title>` in place rather than creating a duplicate.
#[wasm_bindgen_test]
fn should_update_existing_title_not_duplicate() -> Result<(), String> {
    let svg = make_svg("a11y-title-update");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_title("First").map_err(|e| e.to_string())?;
    rect.set_title("Second").map_err(|e| e.to_string())?;
    check_eq(rect.as_element().child_element_count(), 1)?;
    check_eq(rect.title(), Some("Second".to_owned()))
}

/// Calling `set_desc` a second time updates the existing `<desc>` in place rather than creating a duplicate.
#[wasm_bindgen_test]
fn should_update_existing_desc_not_duplicate() -> Result<(), String> {
    let svg = make_svg("a11y-desc-update");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_desc("First").map_err(|e| e.to_string())?;
    rect.set_desc("Second").map_err(|e| e.to_string())?;
    check_eq(rect.as_element().child_element_count(), 1)?;
    check_eq(rect.desc(), Some("Second".to_owned()))
}

/// A new `<title>` is inserted as the first child even when the element already has other children.
#[wasm_bindgen_test]
fn should_insert_title_as_first_child() -> Result<(), String> {
    let svg = make_svg("a11y-title-first");
    let group = svg.group().map_err(|e| e.to_string())?;
    svg.build_batch_into(&group, |b| {
        b.circle(Point::new(10.0, 10.0), 5.0)?;
        Ok(())
    })
    .map_err(|e| e.to_string())?;
    group.set_title("A circle").map_err(|e| e.to_string())?;
    let first = group.as_element().first_element_child().ok_or("expected a first child")?;
    check_eq(first.tag_name(), "title".to_owned())
}

/// `title` and `desc` are independent: setting one does not affect the other.
#[wasm_bindgen_test]
fn should_keep_title_and_desc_independent() -> Result<(), String> {
    let svg = make_svg("a11y-independent");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_title("A title").map_err(|e| e.to_string())?;
    rect.set_desc("A description").map_err(|e| e.to_string())?;
    check_eq(rect.as_element().child_element_count(), 2)?;
    check_eq(rect.title(), Some("A title".to_owned()))?;
    check_eq(rect.desc(), Some("A description".to_owned()))
}

/// `<title>` always precedes `<desc>` in the DOM when both are set, regardless of call order — matching the SVG
/// specification's own examples and preventing `<desc>`'s "insert as first child when absent" rule from landing it
/// ahead of an already-present `<title>`.
#[wasm_bindgen_test]
fn should_order_title_before_desc_when_title_set_first() -> Result<(), String> {
    let svg = make_svg("a11y-order-title-first");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_title("A title").map_err(|e| e.to_string())?;
    rect.set_desc("A description").map_err(|e| e.to_string())?;
    let first = rect.as_element().first_element_child().ok_or("expected a first child")?;
    check_eq(first.tag_name(), "title".to_owned())
}

/// Same as above, but with `set_desc` called *before* `set_title` — `<title>` should still end up first.
#[wasm_bindgen_test]
fn should_order_title_before_desc_when_desc_set_first() -> Result<(), String> {
    let svg = make_svg("a11y-order-desc-first");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_desc("A description").map_err(|e| e.to_string())?;
    rect.set_title("A title").map_err(|e| e.to_string())?;
    let first = rect.as_element().first_element_child().ok_or("expected a first child")?;
    check_eq(first.tag_name(), "title".to_owned())
}

/// `remove_title` removes the `<title>` child; `title()` then returns `None`.
#[wasm_bindgen_test]
fn should_remove_title() -> Result<(), String> {
    let svg = make_svg("a11y-title-remove");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_title("Close dialog").map_err(|e| e.to_string())?;
    rect.remove_title();
    check_eq(rect.title(), None)?;
    check_eq(rect.as_element().child_element_count(), 0)
}

/// `remove_desc` removes the `<desc>` child; `desc()` then returns `None`.
#[wasm_bindgen_test]
fn should_remove_desc() -> Result<(), String> {
    let svg = make_svg("a11y-desc-remove");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_desc("Discards unsaved changes.").map_err(|e| e.to_string())?;
    rect.remove_desc();
    check_eq(rect.desc(), None)?;
    check_eq(rect.as_element().child_element_count(), 0)
}

/// `remove_title` on an element with no `<title>` is a harmless no-op.
#[wasm_bindgen_test]
fn should_no_op_remove_title_when_absent() -> Result<(), String> {
    let svg = make_svg("a11y-title-remove-noop");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.remove_title();
    check_eq(rect.as_element().child_element_count(), 0)
}

/// `remove_desc` on an element with no `<desc>` is a harmless no-op.
#[wasm_bindgen_test]
fn should_no_op_remove_desc_when_absent() -> Result<(), String> {
    let svg = make_svg("a11y-desc-remove-noop");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.remove_desc();
    check_eq(rect.as_element().child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Empty / whitespace-only rejection (SVG 2 forbids blank <title>/<desc>)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `set_title("")` is rejected and no `<title>` child is created.
#[wasm_bindgen_test]
fn should_reject_empty_title() -> Result<(), String> {
    let svg = make_svg("a11y-title-empty");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    let result = rect.set_title("");
    check(
        matches!(result, Err(Error::InvalidAccessibleText("title"))),
        "expected InvalidAccessibleText(\"title\") for an empty string",
    )?;
    check_eq(rect.as_element().child_element_count(), 0)
}

/// `set_title(" ")` (spaces only) is rejected.
#[wasm_bindgen_test]
fn should_reject_space_only_title() -> Result<(), String> {
    let svg = make_svg("a11y-title-spaces");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    let result = rect.set_title("   ");
    check(
        matches!(result, Err(Error::InvalidAccessibleText("title"))),
        "expected InvalidAccessibleText(\"title\") for a space-only string",
    )?;
    check_eq(rect.as_element().child_element_count(), 0)
}

/// `set_title("\n\t")` (line-break/tab whitespace only) is rejected.
#[wasm_bindgen_test]
fn should_reject_line_break_only_title() -> Result<(), String> {
    let svg = make_svg("a11y-title-linebreak");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    let result = rect.set_title("\n\t \n");
    check(
        matches!(result, Err(Error::InvalidAccessibleText("title"))),
        "expected InvalidAccessibleText(\"title\") for a line-break/tab-only string",
    )?;
    check_eq(rect.as_element().child_element_count(), 0)
}

/// `set_desc("")` is rejected and no `<desc>` child is created.
#[wasm_bindgen_test]
fn should_reject_empty_desc() -> Result<(), String> {
    let svg = make_svg("a11y-desc-empty");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    let result = rect.set_desc("");
    check(
        matches!(result, Err(Error::InvalidAccessibleText("desc"))),
        "expected InvalidAccessibleText(\"desc\") for an empty string",
    )?;
    check_eq(rect.as_element().child_element_count(), 0)
}

/// `set_desc(" ")` (spaces only) is rejected.
#[wasm_bindgen_test]
fn should_reject_space_only_desc() -> Result<(), String> {
    let svg = make_svg("a11y-desc-spaces");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    let result = rect.set_desc("   ");
    check(
        matches!(result, Err(Error::InvalidAccessibleText("desc"))),
        "expected InvalidAccessibleText(\"desc\") for a space-only string",
    )?;
    check_eq(rect.as_element().child_element_count(), 0)
}

/// `set_desc("\n\t")` (line-break/tab whitespace only) is rejected.
#[wasm_bindgen_test]
fn should_reject_line_break_only_desc() -> Result<(), String> {
    let svg = make_svg("a11y-desc-linebreak");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    let result = rect.set_desc("\n\t \n");
    check(
        matches!(result, Err(Error::InvalidAccessibleText("desc"))),
        "expected InvalidAccessibleText(\"desc\") for a line-break/tab-only string",
    )?;
    check_eq(rect.as_element().child_element_count(), 0)
}

/// Rejecting a blank update leaves a *pre-existing* `<title>` untouched, rather than blanking it out.
#[wasm_bindgen_test]
fn should_not_blank_existing_title_on_rejected_update() -> Result<(), String> {
    let svg = make_svg("a11y-title-reject-preserves");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_title("Original").map_err(|e| e.to_string())?;
    let result = rect.set_title("   ");
    check(
        matches!(result, Err(Error::InvalidAccessibleText("title"))),
        "expected rejection",
    )?;
    check_eq(rect.title(), Some("Original".to_owned()))
}

/// Rejecting a blank update leaves a *pre-existing* `<desc>` untouched, rather than blanking it out.
#[wasm_bindgen_test]
fn should_not_blank_existing_desc_on_rejected_update() -> Result<(), String> {
    let svg = make_svg("a11y-desc-reject-preserves");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_desc("Original").map_err(|e| e.to_string())?;
    let result = rect.set_desc("\n");
    check(
        matches!(result, Err(Error::InvalidAccessibleText("desc"))),
        "expected rejection",
    )?;
    check_eq(rect.desc(), Some("Original".to_owned()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Singularity scope: pre-existing extra siblings (e.g. externally authored multilingual markup) are left untouched.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// A pre-existing second `<title>` sibling (simulating externally authored, multilingual markup this crate did not
/// build) is left completely untouched by `set_title`/`title()`/`remove_title` — they only ever act on the first
/// direct `<title>` child, per `set_title`'s `# Scope` doc section.
#[wasm_bindgen_test]
fn should_leave_other_title_siblings_untouched() -> Result<(), String> {
    let svg = make_svg("a11y-title-multi-sibling");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_title("English").map_err(|e| e.to_string())?;

    let doc = rect.as_element().owner_document().ok_or("expected an owner document")?;
    let fr_title = doc
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "title")
        .map_err(|_| "create_element_ns failed")?;
    fr_title.set_attribute("lang", "fr").map_err(|_| "set_attribute failed")?;
    fr_title.set_text_content(Some("Français"));
    rect.as_element().append_child(&fr_title).map_err(|_| "append_child failed")?;
    check_eq(rect.as_element().child_element_count(), 2)?;

    // set_title/title() only ever touch the first <title> — the English one.
    rect.set_title("Updated English").map_err(|e| e.to_string())?;
    check_eq(rect.title(), Some("Updated English".to_owned()))?;
    check_eq(fr_title.text_content(), Some("Français".to_owned()))?;

    // remove_title() removes only the first <title>, leaving the French sibling in place.
    rect.remove_title();
    check_eq(rect.as_element().child_element_count(), 1)?;
    check_eq(fr_title.text_content(), Some("Français".to_owned()))?;
    // With the English <title> gone, the French one is now the sole/first match.
    check_eq(rect.title(), Some("Français".to_owned()))
}

/// Same as `should_leave_other_title_siblings_untouched`, but for `<desc>`.
#[wasm_bindgen_test]
fn should_leave_other_desc_siblings_untouched() -> Result<(), String> {
    let svg = make_svg("a11y-desc-multi-sibling");
    let rect = svg.rect(Point::origin(), Size::new(20.0, 20.0)).map_err(|e| e.to_string())?;
    rect.set_desc("English description").map_err(|e| e.to_string())?;

    let doc = rect.as_element().owner_document().ok_or("expected an owner document")?;
    let fr_desc = doc
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "desc")
        .map_err(|_| "create_element_ns failed")?;
    fr_desc.set_attribute("lang", "fr").map_err(|_| "set_attribute failed")?;
    fr_desc.set_text_content(Some("Description française"));
    rect.as_element().append_child(&fr_desc).map_err(|_| "append_child failed")?;
    check_eq(rect.as_element().child_element_count(), 2)?;

    rect.set_desc("Updated English description").map_err(|e| e.to_string())?;
    check_eq(rect.desc(), Some("Updated English description".to_owned()))?;
    check_eq(fr_desc.text_content(), Some("Description française".to_owned()))?;

    rect.remove_desc();
    check_eq(rect.as_element().child_element_count(), 1)?;
    check_eq(fr_desc.text_content(), Some("Description française".to_owned()))?;
    check_eq(rect.desc(), Some("Description française".to_owned()))
}
