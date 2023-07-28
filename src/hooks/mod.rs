mod use_scroll;

pub use use_scroll::*;

#[cfg(feature = "hydrate")]
use std::time::Duration;
#[cfg(feature = "hydrate")]
pub fn debounce_call<T>(fun: impl Fn(T) + 'static, delay: Duration) -> impl Fn(T) + 'static {
    use std::{cell::Cell, rc::Rc};

    use leptos::set_timeout;

    let can_call = Rc::new(Cell::new(true));

    move |arg: T| {
        if !can_call.get() {
            return;
        }
        can_call.set(false);

        let can_call = Rc::clone(&can_call);

        set_timeout(
            move || {
                can_call.set(true);
            },
            delay,
        );

        fun(arg)
    }
}
