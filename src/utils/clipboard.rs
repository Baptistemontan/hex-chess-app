#[cfg(feature = "hydrate")]
pub fn copy_to_clipboard(to_copy: &str) -> Option<()> {
    let window = leptos::window();
    let nav = window.navigator();
    let clip = nav.clipboard()?;
    let _ = clip.write_text(to_copy);
    Some(())
}

#[cfg(not(feature = "hydrate"))]
pub fn copy_to_clipboard(_to_copy: &str) -> Option<()> {
    None
}
