use crate::hooks::{use_scroll, ScrollDirection};

use super::super::auth::{LoggedIn, NotLoggedIn};
use leptos::*;

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
                    <h1 class="title">"Hex Chess"</h1>
                </a>
                <div class="topLinks">
                    {navigation_list(cx)}
                    {login_button(cx)}
                </div>
            </nav>
        </header>
    }
}

const LINKS: &[(&str, &str)] = &[("About", "/about")];

pub fn navigation_list(cx: Scope) -> impl IntoView {
    view! { cx,
        <ol>
            {LINKS.iter().copied().map(move |(text, path)| {
                view! { cx,
                    <li class="link">
                        <a href=path>{text}</a>
                    </li>
                }
            }).collect_view(cx)}
        </ol>
    }
}

pub fn login_button(cx: Scope) -> impl IntoView {
    let location = leptos_router::use_location(cx);

    let login_url = move || format!("/api/auth/login?origin={}", location.pathname.get());

    view! { cx,
        <div class="big_button">
            <LoggedIn>
                <a href="/api/auth/logout" rel="external">"Logout"</a>
            </LoggedIn>
            <NotLoggedIn>
                <a href=login_url rel="external">"Login"</a>
            </NotLoggedIn>
        </div>
    }
}
