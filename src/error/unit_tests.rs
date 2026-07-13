use super::Error;

macro_rules! ensure_eq {
    ($left:expr, $right:expr) => {{
        let left = $left;
        let right = $right;
        if left != right {
            return Err(format!("expected {:?}, got {:?}", right, left));
        }
    }};
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Display
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_error_on_display_element_not_found() -> Result<(), String> {
    let err = Error::ElementNotFound("my-svg".into());
    ensure_eq!(err.to_string(), "element not found: #my-svg");
    Ok(())
}

#[test]
fn should_error_on_display_element_not_found_empty_id() -> Result<(), String> {
    let err = Error::ElementNotFound(String::new());
    ensure_eq!(err.to_string(), "element not found: #");
    Ok(())
}

#[test]
fn should_error_on_display_dom_error() -> Result<(), String> {
    let err = Error::Dom("operation failed".into());
    ensure_eq!(err.to_string(), "DOM error: operation failed");
    Ok(())
}

#[test]
fn should_error_on_display_dom_error_empty_message() -> Result<(), String> {
    let err = Error::Dom(String::new());
    ensure_eq!(err.to_string(), "DOM error: ");
    Ok(())
}

#[test]
fn should_error_on_display_cast_failed() -> Result<(), String> {
    let err = Error::CastFailed("SvgsvgElement");
    ensure_eq!(err.to_string(), "JsCast to SvgsvgElement failed");
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Debug (derived — verify the inner values are preserved and readable)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_error_on_debug_element_not_found() -> Result<(), String> {
    let err = Error::ElementNotFound("canvas".into());
    ensure_eq!(format!("{err:?}"), r#"ElementNotFound("canvas")"#);
    Ok(())
}

#[test]
fn should_error_on_debug_dom_error() -> Result<(), String> {
    let err = Error::Dom("no window".into());
    ensure_eq!(format!("{err:?}"), r#"Dom("no window")"#);
    Ok(())
}

#[test]
fn should_error_on_debug_cast_failed() -> Result<(), String> {
    let err = Error::CastFailed("SvgElement");
    ensure_eq!(format!("{err:?}"), r#"CastFailed("SvgElement")"#);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Inner values are accessible via pattern matching
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_error_on_element_not_found_inner_value() -> Result<(), String> {
    let id = "diagram";
    let Error::ElementNotFound(inner) = Error::ElementNotFound(id.into()) else {
        return Err("expected ElementNotFound variant".into());
    };
    ensure_eq!(inner, id);
    Ok(())
}

#[test]
fn should_error_on_dom_inner_value() -> Result<(), String> {
    let msg = "appendChild failed";
    let Error::Dom(inner) = Error::Dom(msg.into()) else {
        return Err("expected Dom variant".into());
    };
    ensure_eq!(inner, msg);
    Ok(())
}

#[test]
fn should_error_on_bad_inner_value_cast() -> Result<(), String> {
    let ty = "SvgsvgElement";
    let Error::CastFailed(inner) = Error::CastFailed(ty) else {
        return Err("expected CastFailed variant".into());
    };
    ensure_eq!(inner, ty);
    Ok(())
}

#[test]
fn should_error_on_display_invalid_marker_id() -> Result<(), String> {
    let err = Error::InvalidMarkerId("url(#arrow)".into());
    ensure_eq!(err.to_string(), r#"invalid svg marker id: "url(#arrow)""#);
    Ok(())
}

#[test]
fn should_error_on_debug_invalid_marker_id() -> Result<(), String> {
    let err = Error::InvalidMarkerId("bad id".into());
    ensure_eq!(format!("{err:?}"), r#"InvalidMarkerId("bad id")"#);
    Ok(())
}

#[test]
fn should_error_on_invalid_marker_id_inner_value() -> Result<(), String> {
    let id = "url(#x)";
    let Error::InvalidMarkerId(inner) = Error::InvalidMarkerId(id.into()) else {
        return Err("expected InvalidMarkerId variant".into());
    };
    ensure_eq!(inner, id);
    Ok(())
}

#[test]
fn should_error_on_display_invalid_clip_path_id() -> Result<(), String> {
    let err = Error::InvalidClipPathId("url(#clip)".into());
    ensure_eq!(err.to_string(), r#"invalid svg clip-path id: "url(#clip)""#);
    Ok(())
}

#[test]
fn should_error_on_debug_invalid_clip_path_id() -> Result<(), String> {
    let err = Error::InvalidClipPathId("bad id".into());
    ensure_eq!(format!("{err:?}"), r#"InvalidClipPathId("bad id")"#);
    Ok(())
}

#[test]
fn should_error_on_invalid_clip_path_id_inner_value() -> Result<(), String> {
    let id = "url(#x)";
    let Error::InvalidClipPathId(inner) = Error::InvalidClipPathId(id.into()) else {
        return Err("expected InvalidClipPathId variant".into());
    };
    ensure_eq!(inner, id);
    Ok(())
}

#[test]
fn should_error_on_display_invalid_symbol_id() -> Result<(), String> {
    let err = Error::InvalidSymbolId("url(#sym)".into());
    ensure_eq!(err.to_string(), r#"invalid svg symbol id: "url(#sym)""#);
    Ok(())
}

#[test]
fn should_error_on_debug_invalid_symbol_id() -> Result<(), String> {
    let err = Error::InvalidSymbolId("bad id".into());
    ensure_eq!(format!("{err:?}"), r#"InvalidSymbolId("bad id")"#);
    Ok(())
}

#[test]
fn should_error_on_invalid_symbol_id_inner_value() -> Result<(), String> {
    let id = "url(#x)";
    let Error::InvalidSymbolId(inner) = Error::InvalidSymbolId(id.into()) else {
        return Err("expected InvalidSymbolId variant".into());
    };
    ensure_eq!(inner, id);
    Ok(())
}

#[test]
fn should_error_on_display_reserved_attribute() -> Result<(), String> {
    let err = Error::ReservedAttribute("id");
    ensure_eq!(err.to_string(), r#"attribute "id" is reserved; use the dedicated setter"#);
    Ok(())
}

#[test]
fn should_error_on_debug_reserved_attribute() -> Result<(), String> {
    let err = Error::ReservedAttribute("id");
    ensure_eq!(format!("{err:?}"), r#"ReservedAttribute("id")"#);
    Ok(())
}

#[test]
fn should_error_on_reserved_attribute_inner_value() -> Result<(), String> {
    let Error::ReservedAttribute(inner) = Error::ReservedAttribute("id") else {
        return Err("expected ReservedAttribute variant".into());
    };
    ensure_eq!(inner, "id");
    Ok(())
}
