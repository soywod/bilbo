use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchMode {
    Keyword,
    Semantic,
}

#[component]
pub fn SearchBar(
    query: RwSignal<String>,
    mode: RwSignal<SearchMode>,
    on_search: Callback<()>,
) -> impl IntoView {
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        on_search.run(());
    };

    view! {
        <div class="search-section">
            <form class="search-bar" on:submit=on_submit>
                <input
                    type="text"
                    placeholder="Rechercher un livre..."
                    prop:value=move || query.get()
                    on:input=move |ev| {
                        query.set(event_target_value(&ev));
                    }
                />
                <button type="submit">"Rechercher"</button>
            </form>
            <div class="search-mode-toggle">
                <label>
                    <input
                        type="radio"
                        name="search_mode"
                        checked=move || mode.get() == SearchMode::Keyword
                        on:change=move |_| mode.set(SearchMode::Keyword)
                    />
                    " Mots-clefs"
                </label>
                <label>
                    <input
                        type="radio"
                        name="search_mode"
                        checked=move || mode.get() == SearchMode::Semantic
                        on:change=move |_| mode.set(SearchMode::Semantic)
                    />
                    " Sémantique"
                </label>
            </div>
        </div>
    }
}
