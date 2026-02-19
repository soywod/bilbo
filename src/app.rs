use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::layout::Layout;
use crate::pages::{book_detail::BookDetailPage, chat::ChatPage, home::HomePage};

#[cfg(feature = "ssr")]
pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    use leptos::prelude::*;

    view! {
        <!DOCTYPE html>
        <html lang="fr">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HashedStylesheet id="leptos" options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Bilbo - Bibliothèque numérique" />
        <Meta name="description" content="Bibliothèque numérique publique en ligne. Recherchez et explorez des livres numérisés." />

        <Router>
            <Layout>
                <Routes fallback=|| view! { <p>"Page non trouvée."</p> }>
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/book/:ref_id") view=BookDetailPage />
                    <Route path=path!("/chat") view=ChatPage />
                </Routes>
            </Layout>
        </Router>
    }
}
