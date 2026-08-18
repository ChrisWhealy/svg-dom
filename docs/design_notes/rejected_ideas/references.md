# Reference-attribute (`url(#id)`) design

[← Back to rejected ideas](README.md)

See [References](../references.md) for the design notes on how this crate actually caches and validates reference-attribute ids.
This rejection is evaluated against those notes.

## Unifying marker references on handles and making marker IDs immutable

The external review raised two concerns:

1. Two marker-reference styles exist: `line.set_marker_end("arrow")?` (a string id) and `line.set_marker_end_ref(&marker)?` (a handle).
   This reintroduces string-typed ids alongside the safer handle form.

2. `SvgMarker::set_id` allows renaming a marker after it has been referenced.
   This leaves all previously written `url(#...)` attributes stale.
   The recommendation was to make ids immutable after construction.
   It also recommended making the handle form the only reference path.

### String-based references cannot be removed

The string form (`set_marker_start/mid/end`) exists for markers that are not created through this crate.
Examples: a pre-existing `<marker>` defined in inline HTML, a CSS `<defs>` block written by hand, or a third-party library.
In those cases the caller has only the marker's id — there is no `SvgMarker` handle to supply.
Removing the string form would make those markers unreferenceable through the crate's API, forcing callers back to raw `element.set_attribute`.

### Staleness is inherent to the SVG reference model, not a Rust API problem

Both the string and handle forms ultimately write a DOM attribute string: `marker-end="url(#arrow)"`.
SVG marker references are not live pointers.
They are strings that the browser resolves by id when it paints the element.
Even if the string form were removed and the handle form were the only path, the written attribute would still be a plain string.
That string becomes stale if the marker is later renamed.
Tracking live references would require the crate to maintain a list of every element that references each marker.
It would also have to update all of them on rename.
That is essentially a live reference-update system, akin to the canonical registry rejected in [Restricting or removing `SvgNode::parent()` to prevent split listener state](node_and_tree.md#restricting-or-removing-svgnodeparent-to-prevent-split-listener-state).
That is far outside the scope of a minimal wrapper crate.

### `set_id` with `&mut self` already provides the strongest practical protection

`set_id` takes `&mut self`.
Rust's borrow checker therefore prevents any code from holding a shared `&SvgMarker` reference while a rename is in progress.
The remaining staleness risk is in the DOM, not in Rust.
A caller can call `set_id` after the marker has already been referenced.
The previously written DOM attributes will not be updated.
This is fully documented on both `set_id` and the `_ref` methods ("reapply the reference after renaming if needed").

### Removing `set_id` would limit legitimate use cases without closing the hole

A marker whose id must be computed at runtime (a generated unique name, a user-provided label, an id derived from data) needs to be renamed after initial construction.
Removing `set_id` would force callers to delete and recreate the marker and reapply all references (a more error-prone and DOM-intensive path) rather than a single guarded rename.
Staleness is inherent to DOM string references.
Making ids immutable would not eliminate that risk.
It would only hide it.

### The genuine inconsistency: `set_marker_start` and `set_marker_mid` lacked the "prefer `_ref`" recommendation

`set_marker_end` already carried the note "Prefer `set_marker_end_ref` when you have the `SvgMarker` handle available."
`set_marker_start` and `set_marker_mid` did not.
This asymmetry was a real inconsistency.
The doc comments on `set_marker_start` and `set_marker_mid` have been updated to match `set_marker_end`.
All three string-id setters now consistently recommend the `_ref` form when a handle is available.
