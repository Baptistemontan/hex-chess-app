use crate::hooks::{use_scroll, ScrollDirection};

use super::super::auth::{LoggedIn, NotLoggedIn};
use crate::i18n::i18n_context;
use leptos::*;
use leptos_i18n::t;

pub fn header(cx: Scope) -> impl IntoView {
    let scroll_infos = use_scroll(cx, 50.0);

    create_effect(cx, move |_| {
        log!("{:?}", scroll_infos.get());
    });

    let scrolled_down =
        move || scroll_infos.with(|infos| infos.direction == ScrollDirection::Down && !infos.top);

    let scrolled_up =
        move || scroll_infos.with(|infos| infos.direction == ScrollDirection::Up && !infos.top);

    view! { cx,
        <header
            class=("headerScrollDown", scrolled_down)
            class=("headerScrollUp", scrolled_up)
        >
            <nav>
                <a href="/">
                    <h1>"Hex Chess"</h1>
                </a>
                <div class="topLinks">
                    {navigation_list(cx)}
                    {login_button(cx)}
                </div>
            </nav>
        </header>
    }
}

pub fn navigation_list(cx: Scope) -> impl IntoView {
    let i18n = i18n_context(cx);
    view! { cx,
        <ol>
            <li class="link">
                <a href="/about">{t!(i18n, about)}</a>
            </li>
        </ol>
    }
}

pub fn login_button(cx: Scope) -> impl IntoView {
    let location = leptos_router::use_location(cx);

    let login_url = move || format!("/api/auth/login?origin={}", location.pathname.get());

    let i18n = i18n_context(cx);

    view! { cx,
        <div class="big_button">
            <LoggedIn>
                <a href="/api/auth/logout" rel="external">{t!(i18n, logout)}</a>
            </LoggedIn>
            <NotLoggedIn>
                <a href=login_url rel="external">{t!(i18n, login)}</a>
            </NotLoggedIn>
        </div>
    }
}
