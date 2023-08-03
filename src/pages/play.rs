use crate::components::auth::CheckLoggedIn;
use crate::components::board::MultiBoard;
use crate::server::board::GameEvent;
use leptos::*;
use leptos_i18n::t;
use leptos_router::*;

#[component]
pub fn WaitingRandom(cx: Scope) -> impl IntoView {
    let navigate = leptos_router::use_navigate(cx);
    let events = GameEventStream::new(cx, &GameEventKind::Random);

    events.listen(cx, move |event| {
        if let GameEvent::GameStart { game_id, .. } = event {
            navigate(&format!("/play/{}", game_id), Default::default()).unwrap();
        }
    });

    view! { cx,
        <CheckLoggedIn>
            <p>{t!(cx, "waiting")}</p>
        </CheckLoggedIn>
    }
}

#[component]
pub fn WaitingCustom(cx: Scope) -> impl IntoView {
    let navigate = leptos_router::use_navigate(cx);
    let events = GameEventStream::new(cx, &GameEventKind::Custom);

    events.listen(cx, move |event| {
        if let GameEvent::CustomCreated { game_id } = event {
            navigate(&format!("/play/custom/{}", game_id), Default::default()).unwrap();
        }
    });

    view! { cx,
        <CheckLoggedIn>
            <p>{t!(cx, "creating")}</p>
        </CheckLoggedIn>
    }
}

#[component]
pub fn WaitingCustomWithId(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    let render = move |cx: Scope| {
        params.get().get("game_id").cloned().map(move |game_id| {
            let navigate = leptos_router::use_navigate(cx);
            let events = GameEventStream::new(cx, &GameEventKind::Join(&game_id));
            let game_id = store_value(cx, game_id);
            let on_copy = move |_| {
                use crate::utils::{clipboard::copy_to_clipboard, url::get_origin};
                if let Some(origin) = get_origin() {
                    copy_to_clipboard(&format!("{}/play/{}", origin, game_id.get_value()));
                }
            };
            events.listen(cx, move |event| {
                if let GameEvent::GameStart { game_id, .. } = event {
                    navigate(&format!("/play/{}", game_id), Default::default()).unwrap();
                }
            });

            view! { cx,
                <div on:click=on_copy class="big_button">
                    <p>{t!(cx, "copy_link")}</p>
                </div>
            }
        })
    };

    view! { cx,
        <CheckLoggedIn>
            <div class="custom_game_link">
                {render(cx)}
            </div>
        </CheckLoggedIn>
    }
}

type EventSignal = ReadSignal<Option<Result<GameEvent, EventError>>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GameEventKind<'a> {
    Custom,
    Join(&'a str),
    Random,
}

#[derive(Debug, Clone, Copy)]
pub struct GameEventStream(EventSignal);

impl GameEventStream {
    pub fn new(cx: Scope, game_kind: &GameEventKind) -> Self {
        GameEventStream(Self::subscribe_to_events(cx, game_kind))
    }

    #[cfg(all(feature = "hydrate", not(feature = "ssr")))]
    fn subscribe_to_events(cx: Scope, game_kind: &GameEventKind) -> EventSignal {
        use futures::StreamExt;
        log!("{:?}", game_kind);
        let url = match game_kind {
            GameEventKind::Custom => "/api/board/new_custom_game".into(),
            GameEventKind::Random => "/api/board/new_random_game".into(),
            GameEventKind::Join(id) => format!("/api/board/join_game/{}", id),
        };
        let mut source = gloo_net::eventsource::futures::EventSource::new(&url).unwrap();
        let stream = source.subscribe("message").unwrap().map(|value| {
            let (_, event) = value?;
            let data = event.data().as_string().unwrap();
            let event: GameEvent = serde_json::from_str(&data).unwrap();
            Ok(event)
        });
        let events = create_signal_from_stream(cx, stream);
        on_cleanup(cx, move || source.close());
        events
    }

    #[cfg(feature = "ssr")]
    fn subscribe_to_events(cx: Scope, _game_kind: &GameEventKind) -> EventSignal {
        create_signal(cx, None).0
    }

    pub fn listen(self, cx: Scope, listener: impl Fn(GameEvent) + 'static) {
        create_effect(cx, move |_: Option<Option<()>>| {
            let event = match self.0.get()? {
                Ok(ev) => ev,
                Err(err) => {
                    log!("err: {:?}", err);
                    return None;
                }
            };

            listener(event);

            None
        });
    }
}

#[component]
pub fn Play(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    let render = move |cx: Scope| {
        params.get().get("game_id").map(|game_id| {
            let events = GameEventStream::new(cx, &GameEventKind::Join(game_id));
            view! { cx,
                <MultiBoard events=events/>
            }
        })
    };

    view! { cx,
        <CheckLoggedIn>
            <div class="board">
                {render(cx)}
            </div>
        </CheckLoggedIn>
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
pub struct EventError;

#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
#[derive(Debug, Clone)]
pub struct EventError(gloo_net::eventsource::EventSourceError);

#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
impl From<gloo_net::eventsource::EventSourceError> for EventError {
    fn from(value: gloo_net::eventsource::EventSourceError) -> Self {
        Self(value)
    }
}
