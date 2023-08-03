mod footer;
mod header;

use leptos::*;

#[component]
pub fn Layout(cx: Scope, children: ChildrenFn) -> impl IntoView {
    view! { cx,
        <>
            {header::header(cx)}
            <main>
                {children(cx)}
            </main>
            {footer::footer(cx)}
        </>
    }
}
