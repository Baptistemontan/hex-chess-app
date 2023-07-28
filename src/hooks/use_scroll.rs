use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollInfo {
    pub direction: ScrollDirection,
    pub top: bool,
}

#[cfg(feature = "hydrate")]
pub fn use_scroll(cx: Scope, top_offset: f64) -> ReadSignal<ScrollInfo> {
    use std::cell::RefCell;
    use std::time::Duration;

    use super::debounce_call;

    let (infos, set_infos) = create_signal(
        cx,
        ScrollInfo {
            direction: ScrollDirection::Down,
            top: true,
        },
    );

    let last_scroll_y = RefCell::new(window().scroll_y().unwrap_or_default());

    let cb = move |_| {
        let new_scroll_y = window().scroll_y().unwrap_or_default();

        let top = new_scroll_y < top_offset;

        let direction = if *last_scroll_y.borrow() < new_scroll_y {
            ScrollDirection::Down
        } else {
            ScrollDirection::Up
        };

        *last_scroll_y.borrow_mut() = new_scroll_y;

        set_infos.set(ScrollInfo { direction, top })
    };

    window_event_listener(ev::scroll, debounce_call(cb, Duration::from_millis(20)));

    infos
}

#[cfg(not(feature = "hydrate"))]
pub fn use_scroll(cx: Scope, _top_offset: f64) -> ReadSignal<ScrollInfo> {
    let (infos, _) = create_signal(
        cx,
        ScrollInfo {
            direction: ScrollDirection::Down,
            top: true,
        },
    );

    infos
}
