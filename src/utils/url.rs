pub fn get_origin() -> Option<String> {
    if cfg!(feature = "hydrate") {
        let window = leptos::window();
        let location = window.location();
        location.origin().ok()
    } else {
        None
    }
}
