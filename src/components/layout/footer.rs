use crate::i18n::{i18n_context, LocaleEnum};
use leptos::*;
use leptos_i18n::t;

pub fn footer(cx: Scope) -> impl IntoView {
    let i18n = i18n_context(cx);
    view! { cx,
        <footer>
            <div class="locales">
                <img
                    on:click=move |_| i18n.set_locale(LocaleEnum::fr)
                    class="locale_icon"
                    alt=t!(i18n, english_flag_icon_alt)
                    src="/assets/icons/french_flag.svg"
                    />
                <img
                    on:click=move |_| i18n.set_locale(LocaleEnum::en)
                    class="locale_icon"
                    alt=t!(i18n, french_flag_icon_alt)
                    src="/assets/icons/english_flag.svg"
                />
            </div>
        </footer>
    }
}
