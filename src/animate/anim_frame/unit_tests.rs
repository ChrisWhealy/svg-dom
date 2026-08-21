#[test]
fn animation_frame_constructors_have_initial_capacity() -> Result<(), String> {
    let new_cap = super::AnimationFrame::new().scratch.capacity();
    if new_cap < 16 {
        return Err(format!("expected new buffer capacity to be 16. Got {new_cap} instead"));
    };

    let default_cap = super::AnimationFrame::default().scratch.capacity();
    if default_cap < 16 {
        return Err(format!("expected default buffer capacity to be 16. Got {default_cap} instead"));
    };

    Ok(())
}
