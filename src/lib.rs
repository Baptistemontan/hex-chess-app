mod components;
pub mod hooks;
pub mod pages;
pub mod server;
pub mod utils;
use cfg_if::cfg_if;
pub use utils::i18n;

cfg_if! {
if #[cfg(feature = "hydrate")] {

  use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn hydrate() {
      use pages::App;
      use leptos::*;

      console_error_panic_hook::set_once();

      leptos::mount_to_body(move |cx| {
          view! { cx, <App/> }
      });
    }
}
}
