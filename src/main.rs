#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use hex_chess_app::{pages::App, server::*};
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};

    fn get_secret_key() -> cookie::Key {
        cookie::Key::generate()
    }

    dotenvy::dotenv().ok();

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> });

    let auth_client = auth::auth_client::AuthClient::new().unwrap();
    let auth_client = web::Data::new(auth_client);

    let secret_key = get_secret_key();

    let base_url = web::Data::new(BaseUrl::new());
    let leptos_options = web::Data::new(conf.leptos_options);

    HttpServer::new(move || {
        App::new()
            .service(web::scope("/api/board").configure(board::server::config))
            .service(web::scope("/api/auth").configure(auth::config))
            .configure(leptos_i18n::config)
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new(
                "/pkg",
                format!("{}/pkg", leptos_options.site_root),
            ))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", &leptos_options.site_root))
            // serve the favicon from /favicon.ico
            .service(favicon)
            .service(manifest)
            .leptos_routes(
                leptos_options.get_ref().clone(),
                routes.clone(),
                |cx| view! { cx, <App/> },
            )
            .app_data(web::Data::clone(&leptos_options))
            .app_data(auth_client.clone())
            .app_data(base_url.clone())
            .wrap(actix_session::SessionMiddleware::new(
                actix_session::storage::CookieSessionStore::default(),
                secret_key.clone(),
            ))
        //.wrap(middleware::Compress::default())
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon.ico")]
async fn favicon(
    leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}
#[cfg(feature = "ssr")]
#[actix_web::get("manifest.json")]
async fn manifest(
    leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/manifest.json"
    ))?)
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `ssg` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features ssg`
    use hex_chess_app::app::*;
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(move |cx| {
        // note: for testing it may be preferrable to replace this with a
        // more specific component, although leptos_router should still work
        view! {cx, <App/> }
    });
}
