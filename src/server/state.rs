use qdrant_client::Qdrant;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub qdrant: Arc<Qdrant>,
    pub http_client: reqwest::Client,
    pub mistral_api_key: String,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let qdrant_url =
            std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
        let mistral_api_key =
            std::env::var("MISTRAL_API_KEY").unwrap_or_default();

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        let mut qdrant_builder = Qdrant::from_url(&qdrant_url);
        if let Ok(api_key) = std::env::var("QDRANT_API_KEY") {
            if !api_key.is_empty() {
                qdrant_builder = qdrant_builder.api_key(api_key);
            }
        }
        let qdrant = qdrant_builder.build()?;

        let http_client = reqwest::Client::new();

        Ok(Self {
            pool,
            qdrant: Arc::new(qdrant),
            http_client,
            mistral_api_key,
        })
    }
}

impl axum::extract::FromRef<AppState> for leptos::config::LeptosOptions {
    fn from_ref(_state: &AppState) -> Self {
        leptos::prelude::get_configuration(None)
            .unwrap()
            .leptos_options
    }
}

pub fn provide_server_context(state: AppState) {
    leptos::prelude::provide_context(state);
}
