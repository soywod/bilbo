use leptos::prelude::*;

#[component]
pub fn SearchBar(
    query: RwSignal<String>,
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
        </div>
    }
}
