use crate::i18n::{LocaleEnum, Locales};
use crate::t;
use leptos::*;
use leptos_i18n::set_locale;

pub fn footer(cx: Scope) -> impl IntoView {
    let set_locale = set_locale::<Locales>(cx);
    view! { cx,
        <footer>
            <div class="locales">
                <img
                    on:click=move |_| set_locale(LocaleEnum::fr)
                    class="locale_icon"
                    alt=t!(cx, english_flag_icon_alt)
                    src="/assets/icons/french_flag.svg"
                    />
                <img
                    on:click=move |_| set_locale(LocaleEnum::en)
                    class="locale_icon"
                    alt=t!(cx, french_flag_icon_alt)
                    src="/assets/icons/english_flag.svg"
                />
            </div>
        </footer>
    }
}
