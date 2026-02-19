use leptos::prelude::*;

use crate::components::tag_badge::TagBadge;
use crate::model::book::BookSearchResult;

#[component]
pub fn BookTable(
    books: Signal<Vec<BookSearchResult>>,
    total: Signal<i64>,
    page: RwSignal<i64>,
    page_size: i64,
) -> impl IntoView {
    let total_pages = move || {
        let t = total.get();
        if t == 0 {
            1
        } else {
            (t + page_size - 1) / page_size
        }
    };

    view! {
        <table class="book-table">
            <thead>
                <tr>
                    <th>"Titre"</th>
                    <th>"Auteurs"</th>
                    <th>"Éditeur"</th>
                    <th>"Tags"</th>
                </tr>
            </thead>
            <tbody>
                {move || {
                    books.get().into_iter().map(|book| {
                        let reference = book.reference.clone();
                        view! {
                            <tr>
                                <td>
                                    <a href=format!("/book/{reference}")>{book.title}</a>
                                </td>
                                <td>{book.authors.join(", ")}</td>
                                <td>{book.editor.unwrap_or_default()}</td>
                                <td>
                                    {book.tags.into_iter().map(|t| view! { <TagBadge tag=t /> }).collect_view()}
                                </td>
                            </tr>
                        }
                    }).collect_view()
                }}
            </tbody>
        </table>
        <div class="pagination">
            <button
                on:click=move |_| page.update(|p| *p = (*p - 1).max(0))
                disabled=move || page.get() == 0
            >
                "Précédent"
            </button>
            <span>{move || format!("Page {} / {}", page.get() + 1, total_pages())}</span>
            <button
                on:click=move |_| page.update(|p| *p += 1)
                disabled=move || { page.get() + 1 >= total_pages() }
            >
                "Suivant"
            </button>
        </div>
    }
}
