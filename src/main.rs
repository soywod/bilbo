#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use bilbo::app::App;
    use bilbo::server::state::AppState;
    use clap::Parser;
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

    let site_root = std::env::var("LEPTOS_SITE_ROOT").unwrap_or_else(|_| "target/site".into());
    let site_addr: std::net::SocketAddr = std::env::var("LEPTOS_SITE_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".into())
        .parse()
        .expect("invalid LEPTOS_SITE_ADDR");
    let leptos_options = leptos::config::LeptosOptions::builder()
        .output_name(std::sync::Arc::<str>::from("bilbo"))
        .site_root(std::sync::Arc::<str>::from(site_root))
        .site_pkg_dir(std::sync::Arc::<str>::from("pkg"))
        .env(leptos::config::Env::PROD)
        .site_addr(site_addr)
        .reload_port(3001)
        .build();
    let addr = leptos_options.site_addr;

    let state = AppState::new(leptos_options.clone()).await.expect("failed to initialize app state");

    if cli.import {
        tracing::info!("starting import pipeline");
        bilbo::server::import::run_import(&state)
            .await
            .expect("import failed");
        tracing::info!("import complete");
    }
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
