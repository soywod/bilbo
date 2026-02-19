#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use bilbo::app::App;
    use bilbo::server::state::AppState;
    use clap::Parser;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower_http::services::ServeDir;
    use tracing_subscriber;

    tracing_subscriber::fmt::init();

    #[derive(Parser)]
    #[command(name = "bilbo")]
    struct Cli {
        /// Run the import pipeline on files in data/ directory
        #[arg(long)]
        import: bool,
    }

    let cli = Cli::parse();

    let state = AppState::new().await.expect("failed to initialize app state");

    if cli.import {
        tracing::info!("starting import pipeline");
        bilbo::server::import::run_import(&state)
            .await
            .expect("import failed");
        tracing::info!("import complete");
    }

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .route(
            "/sitemap.xml",
            axum::routing::get(bilbo::server::api::sitemap_handler),
        )
        .leptos_routes_with_context(
            &state,
            routes,
            {
                let state = state.clone();
                move || bilbo::server::state::provide_server_context(state.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || {
                    use bilbo::app::shell;
                    shell(leptos_options.clone())
                }
            },
        )
        .fallback(axum::routing::get_service(ServeDir::new(
            leptos_options.site_root.as_ref(),
        )))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
