use leptos::prelude::*;

use crate::api::{list_authors, list_tags, search_books, semantic_search};
use crate::components::book_table::BookTable;
use crate::components::search_bar::{SearchBar, SearchMode};

#[component]
pub fn HomePage() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let mode = RwSignal::new(SearchMode::Keyword);
    let page = RwSignal::new(0_i64);
    let selected_tags = RwSignal::new(Vec::<String>::new());
    let selected_author = RwSignal::new(Option::<String>::None);
    let page_size = 20_i64;

    let search_trigger = RwSignal::new(0_u32);

    let tags_resource = Resource::new(|| (), |_| list_tags());
    let authors_resource = Resource::new(|| (), |_| list_authors());

    // Main search resource: runs on SSR and re-runs when trigger/page changes
    let search_resource = Resource::new(
        move || (search_trigger.get(), page.get()),
        move |(_trigger, p)| {
            let q = query.get();
            let tags = selected_tags.get();
            let author = selected_author.get();
            let current_mode = mode.get();
            async move {
                match current_mode {
                    SearchMode::Keyword => {
                        search_books(q, tags, author, p, page_size)
                            .await
                            .unwrap_or_default()
                    }
                    SearchMode::Semantic => {
                        let results = semantic_search(q, tags, 20)
                            .await
                            .unwrap_or_default();
                        let count = results.len() as i64;
                        (results, count)
                    }
                }
            }
        },
    );

    let do_search = Callback::new(move |()| {
        page.set(0);
        search_trigger.update(|n| *n += 1);
    });

    view! {
        <h1>"Rechercher des livres"</h1>

        <SearchBar query=query mode=mode on_search=do_search />

        <div class="search-filters">
            <Suspense fallback=|| view! { <span>"Chargement tags..."</span> }>
                {move || {
                    tags_resource.get().map(|tags_result| {
                        let tags = tags_result.unwrap_or_default();
                        view! {
                            <select on:change=move |ev| {
                                let val = event_target_value(&ev);
                                if val.is_empty() {
                                    selected_tags.set(vec![]);
                                } else {
                                    selected_tags.set(vec![val]);
                                }
                            }>
                                <option value="">"Tous les tags"</option>
                                {tags.into_iter().map(|t| {
                                    let t2 = t.clone();
                                    view! { <option value=t>{t2}</option> }
                                }).collect_view()}
                            </select>
                        }
                    })
                }}
            </Suspense>

            <Suspense fallback=|| view! { <span>"Chargement auteurs..."</span> }>
                {move || {
                    authors_resource.get().map(|authors_result| {
                        let authors = authors_result.unwrap_or_default();
                        view! {
                            <select on:change=move |ev| {
                                let val = event_target_value(&ev);
                                if val.is_empty() {
                                    selected_author.set(None);
                                } else {
                                    selected_author.set(Some(val));
                                }
                            }>
                                <option value="">"Tous les auteurs"</option>
                                {authors.into_iter().map(|a| {
                                    let a2 = a.clone();
                                    view! { <option value=a>{a2}</option> }
                                }).collect_view()}
                            </select>
                        }
                    })
                }}
            </Suspense>
        </div>

        <Suspense fallback=|| view! { <p>"Chargement..."</p> }>
            {move || {
                search_resource.get().map(|(books, total)| {
                    let books_signal = Signal::derive(move || books.clone());
                    let total_signal = Signal::derive(move || total);
                    view! {
                        <BookTable books=books_signal total=total_signal page=page page_size=page_size />
                    }
                })
            }}
        </Suspense>
    }
}
