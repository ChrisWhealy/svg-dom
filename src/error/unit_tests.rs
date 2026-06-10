use super::Error;

macro_rules! ensure_eq {
    ($left:expr, $right:expr) => {{
        let left  = $left;
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
fn display_element_not_found() -> Result<(), String> {
    let err = Error::ElementNotFound("my-svg".into());
    ensure_eq!(err.to_string(), "element not found: #my-svg");
    Ok(())
}

#[test]
fn display_element_not_found_empty_id() -> Result<(), String> {
    let err = Error::ElementNotFound(String::new());
    ensure_eq!(err.to_string(), "element not found: #");
    Ok(())
}

#[test]
fn display_dom_error() -> Result<(), String> {
    let err = Error::Dom("operation failed".into());
    ensure_eq!(err.to_string(), "DOM error: operation failed");
    Ok(())
}

#[test]
fn display_dom_error_empty_message() -> Result<(), String> {
    let err = Error::Dom(String::new());
    ensure_eq!(err.to_string(), "DOM error: ");
    Ok(())
}

#[test]
fn display_cast_failed() -> Result<(), String> {
    let err = Error::CastFailed("SvgsvgElement");
    ensure_eq!(err.to_string(), "JsCast to SvgsvgElement failed");
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Debug (derived — verify the inner values are preserved and readable)
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn debug_element_not_found() -> Result<(), String> {
    let err = Error::ElementNotFound("canvas".into());
    ensure_eq!(format!("{err:?}"), r#"ElementNotFound("canvas")"#);
    Ok(())
}

#[test]
fn debug_dom_error() -> Result<(), String> {
    let err = Error::Dom("no window".into());
    ensure_eq!(format!("{err:?}"), r#"Dom("no window")"#);
    Ok(())
}

#[test]
fn debug_cast_failed() -> Result<(), String> {
    let err = Error::CastFailed("SvgElement");
    ensure_eq!(format!("{err:?}"), r#"CastFailed("SvgElement")"#);
    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
// Inner values are accessible via pattern matching
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn element_not_found_inner_value() -> Result<(), String> {
    let id = "diagram";
    let Error::ElementNotFound(inner) = Error::ElementNotFound(id.into()) else {
        return Err("expected ElementNotFound variant".into());
    };
    ensure_eq!(inner, id);
    Ok(())
}

#[test]
fn dom_inner_value() -> Result<(), String> {
    let msg = "appendChild failed";
    let Error::Dom(inner) = Error::Dom(msg.into()) else {
        return Err("expected Dom variant".into());
    };
    ensure_eq!(inner, msg);
    Ok(())
}

#[test]
fn cast_failed_inner_value() -> Result<(), String> {
    let ty = "SvgsvgElement";
    let Error::CastFailed(inner) = Error::CastFailed(ty) else {
        return Err("expected CastFailed variant".into());
    };
    ensure_eq!(inner, ty);
    Ok(())
}
