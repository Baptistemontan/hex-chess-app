use leptos::*;
use leptos_i18n::{set_locale, t};

pub fn footer(cx: Scope) -> impl IntoView {
    view! { cx,
        <footer>
            <div class="locales">
                <img
                    on:click=move |_| set_locale(cx, "fr")
                    class="locale_icon"
                    alt=t!(cx, "english_flag_icon_alt")
                    src="/assets/icons/french_flag.svg"
                    />
                <img
                    on:click=move |_| set_locale(cx, "en")
                    class="locale_icon"
                    alt=t!(cx, "french_flag_icon_alt")
                    src="/assets/icons/english_flag.svg"
                />
            </div>
        </footer>
    }
}
