use leptos::*;

pub fn footer(cx: Scope) -> impl IntoView {
    view! { cx,
        <footer>
            <div class="locales">
                <img
                    on:click=move |_| leptos_i18n::set_locale(cx, "fr")
                    class="locale_icon"
                    src="/assets/icons/french_flag.svg"
                    />
                <img
                    on:click=move |_| leptos_i18n::set_locale(cx, "en")
                    class="locale_icon"
                    src="/assets/icons/english_flag.svg"
                />
            </div>
        </footer>
    }
}
