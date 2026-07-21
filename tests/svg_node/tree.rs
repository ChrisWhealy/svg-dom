use crate::{common, helpers::make_svg};
use svg_dom::root::utils::{Point, Size};
use wasm_bindgen_test::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Clone semantics
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Cloning a node produces a second handle to the same underlying DOM element: a mutation made through the clone is
/// immediately visible via the original.
#[wasm_bindgen_test]
fn should_refer_to_same_dom_node_after_clone() -> Result<(), String> {
    let rect = make_svg("node-clone")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    let clone = rect.clone();
    clone.set_fill("crimson").map_err(|e| e.to_string())?;
    common::check_eq(rect.attr("fill"), Some("crimson".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// append
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `append` transfers an element from one parent so it becomes the child of the new target node.
#[wasm_bindgen_test]
fn should_append_element_to_group() -> Result<(), String> {
    let svg = make_svg("node-append");
    let group = svg.group().map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    group.append(&rect).map_err(|e| e.to_string())?;
    common::check_eq(group.as_element().child_element_count(), 1)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// remove
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `remove` detaches a node so its former parent no longer counts it as a child.
#[wasm_bindgen_test]
fn should_detach_node_after_remove() -> Result<(), String> {
    let svg = make_svg("node-remove");
    let group = svg.group().map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(50.0, 50.0)).map_err(|e| e.to_string())?;
    group.append(&rect).map_err(|e| e.to_string())?;
    common::check_eq(group.as_element().child_element_count(), 1)?;

    rect.remove();
    common::check_eq(group.as_element().child_element_count(), 0)
}

/// Calling `remove` on an already-detached node is idempotent.
#[wasm_bindgen_test]
fn should_succeed_when_removing_already_detached_node() -> Result<(), String> {
    let rect = make_svg("node-remove-detached")
        .rect(Point::origin(), Size::new(50.0, 50.0))
        .map_err(|e| e.to_string())?;
    rect.remove();
    rect.remove(); // a node with no parent: still no panic, still no error
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// insert_before
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `insert_before` places the new child ahead of the reference child in document (paint) order.
#[wasm_bindgen_test]
fn should_insert_child_before_reference() -> Result<(), String> {
    let svg = make_svg("node-insert-before");
    let group = svg.group().map_err(|e| e.to_string())?;

    let front = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    front.set_attr("id", "front").map_err(|e| e.to_string())?;
    group.append(&front).map_err(|e| e.to_string())?;

    let behind = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    behind.set_attr("id", "behind").map_err(|e| e.to_string())?;
    group.insert_before(&behind, &front).map_err(|e| e.to_string())?;

    common::check_eq(group.as_element().child_element_count(), 2)?;
    let first_id = group.as_element().first_element_child().map(|e| e.id());
    common::check_eq(first_id, Some("behind".into()))
}

/// `insert_before` fails when the reference node is not a child of the target node.
#[wasm_bindgen_test]
fn should_error_when_reference_is_not_a_child() -> Result<(), String> {
    let svg = make_svg("node-insert-before-bad-ref");
    let group = svg.group().map_err(|e| e.to_string())?;
    // `stranger` is appended to the <svg> root by the factory, not to `group`.
    let stranger = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    let newcomer = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    common::check(
        group.insert_before(&newcomer, &stranger).is_err(),
        "insert_before should error when the reference is not a child of the target",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// clear
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `clear` removes every child of a node, leaving it empty.
#[wasm_bindgen_test]
fn should_remove_all_children_after_clear() -> Result<(), String> {
    let svg = make_svg("node-clear");
    let group = svg.group().map_err(|e| e.to_string())?;

    let rect = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    let circle = svg.circle(Point::new(20.0, 20.0), 5.0).map_err(|e| e.to_string())?;
    group.append(&rect).map_err(|e| e.to_string())?;
    group.append(&circle).map_err(|e| e.to_string())?;
    common::check_eq(group.as_element().child_element_count(), 2)?;

    group.clear();
    common::check_eq(group.as_element().child_element_count(), 0)
}

/// Calling `clear` on a node that has no children is idempotent.
#[wasm_bindgen_test]
fn should_succeed_when_clearing_empty_node() -> Result<(), String> {
    let group = make_svg("node-clear-empty").group().map_err(|e| e.to_string())?;
    group.clear();
    common::check_eq(group.as_element().child_element_count(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// replace_with
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `replace_with` swaps a node for another in the same position; the replacement keeps the old node's place and the
/// child count is unchanged.
#[wasm_bindgen_test]
fn should_swap_node_for_replacement() -> Result<(), String> {
    let svg = make_svg("node-replace-with");
    let group = svg.group().map_err(|e| e.to_string())?;

    let placeholder = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    placeholder.set_attr("id", "placeholder").map_err(|e| e.to_string())?;
    group.append(&placeholder).map_err(|e| e.to_string())?;

    let circle = svg.circle(Point::new(5.0, 5.0), 5.0).map_err(|e| e.to_string())?;
    circle.set_attr("id", "circle").map_err(|e| e.to_string())?;
    group.append(&circle).map_err(|e| e.to_string())?;

    // Swap the placeholder (first child) for a brand-new rect.
    let replacement = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    replacement.set_attr("id", "replacement").map_err(|e| e.to_string())?;
    placeholder.replace_with(&replacement).map_err(|e| e.to_string())?;

    common::check_eq(group.as_element().child_element_count(), 2)?;
    let first_id = group.as_element().first_element_child().map(|e| e.id());
    common::check_eq(first_id, Some("replacement".into()))
}

/// `replace_with` fails on a node that has no parent (nothing to replace it within).
#[wasm_bindgen_test]
fn should_error_when_replacing_parentless_node() -> Result<(), String> {
    let svg = make_svg("node-replace-with-parentless");
    let orphan = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    orphan.remove(); // detach it, so it now has no parent

    let replacement = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    common::check(
        orphan.replace_with(&replacement).is_err(),
        "replace_with should error when the node has no parent",
    )
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// parent
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `parent` returns a handle to the containing element after a node is appended into a group.
#[wasm_bindgen_test]
fn should_return_parent_after_append() -> Result<(), String> {
    let svg = make_svg("node-parent");
    let group = svg.group().map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    group.append(&rect).map_err(|e| e.to_string())?;

    let parent = rect.parent().ok_or("expected a parent handle")?;
    common::check_eq(parent.as_element().tag_name(), "g".to_string())
}

/// `parent` returns `None` for a node that has been detached from the DOM.
#[wasm_bindgen_test]
fn should_return_none_parent_for_detached_node() -> Result<(), String> {
    let rect = make_svg("node-parent-detached")
        .rect(Point::origin(), Size::new(10.0, 10.0))
        .map_err(|e| e.to_string())?;
    rect.remove();
    common::check(rect.parent().is_none(), "a detached node should have no parent")
}

/// The handle returned by `parent` points at the same DOM element: a mutation made through it is visible via a
/// separately held handle to that same parent. (It is, however, an independent handle with its own listener storage.)
#[wasm_bindgen_test]
fn should_return_parent_handle_to_same_element() -> Result<(), String> {
    let svg = make_svg("node-parent-same");
    let group = svg.group().map_err(|e| e.to_string())?;
    let rect = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    group.append(&rect).map_err(|e| e.to_string())?;

    let parent = rect.parent().ok_or("expected a parent handle")?;
    parent.set_attr("data-tag", "via-parent").map_err(|e| e.to_string())?;

    // Observed through the original `group` handle — same underlying element.
    common::check_eq(group.attr("data-tag"), Some("via-parent".into()))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// first_child / last_child
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `first_child` returns a handle to the first appended child.
#[wasm_bindgen_test]
fn should_return_first_child_after_appends() -> Result<(), String> {
    let svg = make_svg("node-first-child");
    let group = svg.group().map_err(|e| e.to_string())?;
    let first = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    first.set_attr("id", "first").map_err(|e| e.to_string())?;
    let second = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    second.set_attr("id", "second").map_err(|e| e.to_string())?;
    group.append(&first).map_err(|e| e.to_string())?;
    group.append(&second).map_err(|e| e.to_string())?;

    let found = group.first_child().ok_or("expected a first child")?;
    common::check_eq(found.attr("id"), Some("first".into()))
}

/// `first_child` returns `None` for a childless node.
#[wasm_bindgen_test]
fn should_return_none_first_child_for_empty_node() -> Result<(), String> {
    let group = make_svg("node-first-child-empty").group().map_err(|e| e.to_string())?;
    common::check(group.first_child().is_none(), "a childless node should have no first child")
}

/// `last_child` returns a handle to the most recently appended child.
#[wasm_bindgen_test]
fn should_return_last_child_after_appends() -> Result<(), String> {
    let svg = make_svg("node-last-child");
    let group = svg.group().map_err(|e| e.to_string())?;
    let first = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    first.set_attr("id", "first").map_err(|e| e.to_string())?;
    let second = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    second.set_attr("id", "second").map_err(|e| e.to_string())?;
    group.append(&first).map_err(|e| e.to_string())?;
    group.append(&second).map_err(|e| e.to_string())?;

    let found = group.last_child().ok_or("expected a last child")?;
    common::check_eq(found.attr("id"), Some("second".into()))
}

/// `last_child` returns `None` for a childless node.
#[wasm_bindgen_test]
fn should_return_none_last_child_for_empty_node() -> Result<(), String> {
    let group = make_svg("node-last-child-empty").group().map_err(|e| e.to_string())?;
    common::check(group.last_child().is_none(), "a childless node should have no last child")
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// next_sibling / previous_sibling
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `next_sibling` returns the element that immediately follows this one in document order.
#[wasm_bindgen_test]
fn should_return_next_sibling() -> Result<(), String> {
    let svg = make_svg("node-next-sibling");
    let group = svg.group().map_err(|e| e.to_string())?;
    let first = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    let second = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    second.set_attr("id", "second").map_err(|e| e.to_string())?;
    group.append(&first).map_err(|e| e.to_string())?;
    group.append(&second).map_err(|e| e.to_string())?;

    let next = first.next_sibling().ok_or("expected a next sibling")?;
    common::check_eq(next.attr("id"), Some("second".into()))
}

/// `next_sibling` returns `None` for the last child.
#[wasm_bindgen_test]
fn should_return_none_next_sibling_for_last_child() -> Result<(), String> {
    let svg = make_svg("node-next-sibling-last");
    let group = svg.group().map_err(|e| e.to_string())?;
    let only = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    group.append(&only).map_err(|e| e.to_string())?;
    common::check(only.next_sibling().is_none(), "the last child should have no next sibling")
}

/// `previous_sibling` returns the element that immediately precedes this one in document order.
#[wasm_bindgen_test]
fn should_return_previous_sibling() -> Result<(), String> {
    let svg = make_svg("node-prev-sibling");
    let group = svg.group().map_err(|e| e.to_string())?;
    let first = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    first.set_attr("id", "first").map_err(|e| e.to_string())?;
    let second = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    group.append(&first).map_err(|e| e.to_string())?;
    group.append(&second).map_err(|e| e.to_string())?;

    let prev = second.previous_sibling().ok_or("expected a previous sibling")?;
    common::check_eq(prev.attr("id"), Some("first".into()))
}

/// `previous_sibling` returns `None` for the first child.
#[wasm_bindgen_test]
fn should_return_none_previous_sibling_for_first_child() -> Result<(), String> {
    let svg = make_svg("node-prev-sibling-first");
    let group = svg.group().map_err(|e| e.to_string())?;
    let only = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    group.append(&only).map_err(|e| e.to_string())?;
    common::check(
        only.previous_sibling().is_none(),
        "the first child should have no previous sibling",
    )
}

/// Walking a chain of children via `first_child` + repeated `next_sibling` visits every child in document order,
/// without the caller having kept a handle to any of them.
#[wasm_bindgen_test]
fn should_walk_all_children_via_first_child_and_next_sibling() -> Result<(), String> {
    let svg = make_svg("node-walk-siblings");
    let group = svg.group().map_err(|e| e.to_string())?;
    for i in 0..4 {
        let rect = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
        rect.set_attr("data-index", &i.to_string()).map_err(|e| e.to_string())?;
        group.append(&rect).map_err(|e| e.to_string())?;
    }

    let mut seen = Vec::new();
    let mut current = group.first_child();
    while let Some(node) = current {
        seen.push(node.attr("data-index").unwrap_or_default());
        current = node.next_sibling();
    }
    common::check_eq(seen, vec!["0".to_string(), "1".to_string(), "2".to_string(), "3".to_string()])
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// children
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `children` returns a handle to every child element, in document order.
#[wasm_bindgen_test]
fn should_return_all_children_in_order() -> Result<(), String> {
    let svg = make_svg("node-children");
    let group = svg.group().map_err(|e| e.to_string())?;
    for i in 0..3 {
        let rect = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
        rect.set_attr("data-index", &i.to_string()).map_err(|e| e.to_string())?;
        group.append(&rect).map_err(|e| e.to_string())?;
    }

    let ids: Vec<String> = group
        .children()
        .iter()
        .map(|c| c.attr("data-index").unwrap_or_default())
        .collect();
    common::check_eq(ids, vec!["0".to_string(), "1".to_string(), "2".to_string()])
}

/// `children` returns an empty `Vec` for a childless node.
#[wasm_bindgen_test]
fn should_return_empty_children_for_empty_node() -> Result<(), String> {
    let group = make_svg("node-children-empty").group().map_err(|e| e.to_string())?;
    common::check_eq(group.children().len(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// query_selector / query_selector_all
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `query_selector` finds a descendant anywhere in the subtree, not just a direct child, using an attribute
/// selector.
#[wasm_bindgen_test]
fn should_find_nested_descendant_via_attribute_selector() -> Result<(), String> {
    let svg = make_svg("node-query-selector");
    let outer = svg.group().map_err(|e| e.to_string())?;
    let inner = svg.group().map_err(|e| e.to_string())?;
    outer.append(&inner).map_err(|e| e.to_string())?;
    let target = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
    target.set_attr("data-role", "target").map_err(|e| e.to_string())?;
    inner.append(&target).map_err(|e| e.to_string())?;

    let found = outer
        .query_selector("[data-role='target']")
        .map_err(|e| e.to_string())?
        .ok_or("expected a match")?;
    common::check_eq(found.as_element().tag_name(), "rect".to_string())
}

/// `query_selector` returns `Ok(None)` when nothing matches.
#[wasm_bindgen_test]
fn should_return_none_query_selector_when_no_match() -> Result<(), String> {
    let group = make_svg("node-query-selector-none").group().map_err(|e| e.to_string())?;
    let result = group.query_selector("[data-role='missing']").map_err(|e| e.to_string())?;
    common::check(result.is_none(), "expected no match")
}

/// `query_selector` returns `Err(Error::Dom)` for invalid CSS selector syntax.
#[wasm_bindgen_test]
fn should_error_on_invalid_selector_syntax() -> Result<(), String> {
    let group = make_svg("node-query-selector-invalid").group().map_err(|e| e.to_string())?;
    common::check(
        group.query_selector(":::not-valid").is_err(),
        "expected an error for invalid selector syntax",
    )
}

/// `query_selector_all` finds every matching descendant, in document order.
#[wasm_bindgen_test]
fn should_find_all_matching_descendants() -> Result<(), String> {
    let svg = make_svg("node-query-selector-all");
    let group = svg.group().map_err(|e| e.to_string())?;
    for i in 0..3 {
        let rect = svg.rect(Point::origin(), Size::new(10.0, 10.0)).map_err(|e| e.to_string())?;
        rect.set_attr("data-tag", "bar").map_err(|e| e.to_string())?;
        rect.set_attr("data-index", &i.to_string()).map_err(|e| e.to_string())?;
        group.append(&rect).map_err(|e| e.to_string())?;
    }
    // A non-matching sibling should not appear in the result.
    let other = svg.circle(Point::new(5.0, 5.0), 5.0).map_err(|e| e.to_string())?;
    group.append(&other).map_err(|e| e.to_string())?;

    let matches = group.query_selector_all("[data-tag='bar']").map_err(|e| e.to_string())?;
    let ids: Vec<String> = matches.iter().map(|n| n.attr("data-index").unwrap_or_default()).collect();
    common::check_eq(ids, vec!["0".to_string(), "1".to_string(), "2".to_string()])
}

/// `query_selector_all` returns an empty `Vec` when nothing matches.
#[wasm_bindgen_test]
fn should_return_empty_query_selector_all_when_no_match() -> Result<(), String> {
    let group = make_svg("node-query-selector-all-none").group().map_err(|e| e.to_string())?;
    let matches = group.query_selector_all("[data-tag='missing']").map_err(|e| e.to_string())?;
    common::check_eq(matches.len(), 0)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Non-SVG content (HTML inside a <foreignObject>)
//
// Every method below documents what happens when traversal or selector matching lands on a non-SVG element; these
// tests build that situation for real with a `<foreignObject>` rather than only asserting it from the doc comments.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// `first_child` returns `None` when the first (and only) child is HTML, rather than reporting the `<foreignObject>`
/// wrapper's HTML content as if it were an `SvgNode`.
#[wasm_bindgen_test]
fn should_return_none_first_child_when_first_child_is_html() -> Result<(), String> {
    let svg = make_svg("node-first-child-html");
    let group = svg.group().map_err(|e| e.to_string())?;
    let fo = common::foreign_object(group.as_element());
    common::html_div(&fo);

    // The <foreignObject> itself is a genuine SvgElement, so it is reachable as the group's first child...
    let fo_node = group
        .first_child()
        .ok_or("expected the <foreignObject> as the group's first child")?;
    common::check_eq(fo_node.as_element().tag_name(), "foreignObject".to_string())?;
    // ...but its own first child is the HTML <div>, which is not.
    common::check(
        fo_node.first_child().is_none(),
        "expected None: the foreignObject's first child is HTML",
    )
}

/// `last_child` returns `None` when the last child is HTML.
#[wasm_bindgen_test]
fn should_return_none_last_child_when_last_child_is_html() -> Result<(), String> {
    let svg = make_svg("node-last-child-html");
    let group = svg.group().map_err(|e| e.to_string())?;
    let fo = common::foreign_object(group.as_element());
    common::svg_rect(&fo);
    common::html_div(&fo);

    let fo_node = group
        .first_child()
        .ok_or("expected the <foreignObject> as the group's first child")?;
    common::check(
        fo_node.last_child().is_none(),
        "expected None: the foreignObject's last child is HTML",
    )
}

/// `next_sibling` returns `None` when the immediately following element is HTML, even though it is not itself the
/// last child of its parent.
#[wasm_bindgen_test]
fn should_return_none_next_sibling_when_next_is_html() -> Result<(), String> {
    let svg = make_svg("node-next-sibling-html");
    let group = svg.group().map_err(|e| e.to_string())?;
    let fo = common::foreign_object(group.as_element());
    common::svg_rect(&fo); // first child: genuine SVG
    common::html_div(&fo); // second child: HTML, immediately follows the rect

    let fo_node = group.first_child().ok_or("expected the <foreignObject>")?;
    let rect_node = fo_node
        .first_child()
        .ok_or("expected the <rect> as the foreignObject's first child")?;
    common::check_eq(rect_node.as_element().tag_name(), "rect".to_string())?;
    common::check(
        rect_node.next_sibling().is_none(),
        "expected None: the rect's next sibling is HTML",
    )
}

/// `previous_sibling` returns `None` when the immediately preceding element is HTML.
#[wasm_bindgen_test]
fn should_return_none_previous_sibling_when_previous_is_html() -> Result<(), String> {
    let svg = make_svg("node-prev-sibling-html");
    let group = svg.group().map_err(|e| e.to_string())?;
    let fo = common::foreign_object(group.as_element());
    common::html_div(&fo); // first child: HTML
    common::svg_rect(&fo); // second (last) child: genuine SVG, immediately follows the div

    let fo_node = group.first_child().ok_or("expected the <foreignObject>")?;
    let rect_node = fo_node
        .last_child()
        .ok_or("expected the <rect> as the foreignObject's last child")?;
    common::check_eq(rect_node.as_element().tag_name(), "rect".to_string())?;
    common::check(
        rect_node.previous_sibling().is_none(),
        "expected None: the rect's previous sibling is HTML",
    )
}

/// `children` silently skips HTML children while retaining the document order of the SVG ones.
#[wasm_bindgen_test]
fn should_skip_html_children_while_retaining_svg_order() -> Result<(), String> {
    let svg = make_svg("node-children-html");
    let group = svg.group().map_err(|e| e.to_string())?;
    let fo = common::foreign_object(group.as_element());

    let r0 = common::svg_rect(&fo);
    r0.set_attribute("data-index", "0").map_err(|e| format!("{e:?}"))?;
    common::html_div(&fo);
    let r1 = common::svg_rect(&fo);
    r1.set_attribute("data-index", "1").map_err(|e| format!("{e:?}"))?;
    common::html_div(&fo);

    let fo_node = group.first_child().ok_or("expected the <foreignObject>")?;
    let ids: Vec<String> = fo_node
        .children()
        .iter()
        .map(|c| c.attr("data-index").unwrap_or_default())
        .collect();
    common::check_eq(ids, vec!["0".to_string(), "1".to_string()])
}

/// `query_selector` returns `Ok(None)` when the browser's first document-order match is HTML, even though a later
/// SVG element also matches the same selector — it does not keep looking for a usable match.
#[wasm_bindgen_test]
fn should_return_none_query_selector_when_first_match_is_html() -> Result<(), String> {
    let svg = make_svg("node-query-selector-html-first");
    let group = svg.group().map_err(|e| e.to_string())?;
    let fo = common::foreign_object(group.as_element());

    let div = common::html_div(&fo); // matches, and comes first in document order
    div.set_attribute("data-role", "target").map_err(|e| format!("{e:?}"))?;
    let rect = common::svg_rect(&fo); // also matches, but comes second
    rect.set_attribute("data-role", "target").map_err(|e| format!("{e:?}"))?;

    let result = group.query_selector("[data-role='target']").map_err(|e| e.to_string())?;
    common::check(result.is_none(), "expected None: the first document-order match is HTML")
}

/// `query_selector_all` omits HTML matches, returning only the SVG ones.
#[wasm_bindgen_test]
fn should_omit_html_matches_from_query_selector_all() -> Result<(), String> {
    let svg = make_svg("node-query-selector-all-html");
    let group = svg.group().map_err(|e| e.to_string())?;
    let fo = common::foreign_object(group.as_element());

    let div = common::html_div(&fo);
    div.set_attribute("data-role", "target").map_err(|e| format!("{e:?}"))?;
    let rect = common::svg_rect(&fo);
    rect.set_attribute("data-role", "target").map_err(|e| format!("{e:?}"))?;

    let matches = group.query_selector_all("[data-role='target']").map_err(|e| e.to_string())?;
    common::check_eq(matches.len(), 1)?;
    common::check_eq(matches[0].as_element().tag_name(), "rect".to_string())
}
