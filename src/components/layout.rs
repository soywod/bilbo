use leptos::prelude::*;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <header class="site-header">
            <div class="header-inner">
                <a href="/" class="logo">"Bilbo"</a>
                <nav>
                    <a href="/">"Recherche"</a>
                    <a href="/chat">"Chat"</a>
                </nav>
            </div>
        </header>
        <main class="site-main">
            {children()}
        </main>
        <footer class="site-footer">
            <p>"Bilbo — Bibliothèque numérique publique"</p>
        </footer>
    }
}
