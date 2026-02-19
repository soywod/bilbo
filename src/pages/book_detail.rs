use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

use crate::api::get_book;
use crate::components::reseller_links::ResellerLinks;
use crate::components::tag_badge::TagBadge;

#[component]
pub fn BookDetailPage() -> impl IntoView {
    let params = use_params_map();
    let reference = move || params.read().get("reference").unwrap_or_default();

    let book_resource = Resource::new(reference, |reference| get_book(reference));

    view! {
        <Suspense fallback=|| view! { <p>"Chargement..."</p> }>
            {move || {
                book_resource.get().map(|result| {
                    match result {
                        Ok(Some(book)) => {
                            let description = book.summary.clone()
                                .map(|s| strip_html_tags(&s))
                                .unwrap_or_else(|| {
                                    format!("{} par {}", book.title, book.authors.join(", "))
                                });
                            let json_ld = build_json_ld(&book);
                            let title = book.title.clone();
                            let og_title = book.title.clone();
                            let og_desc = description.clone();
                            let og_url = format!("https://bilbo.example.com/book/{}", book.reference);
                            let authors_str = book.authors.join(", ");
                            let editor_str = book.editor.clone().unwrap_or_default();
                            let edition_date_str = book.edition_date.clone().unwrap_or_default();

                            view! {
                                <Title text=title />
                                <Meta name="description" content=description />
                                <Meta property="og:title" content=og_title />
                                <Meta property="og:description" content=og_desc />
                                <Meta property="og:type" content="book" />
                                <Meta property="og:url" content=og_url />
                                <Script type_="application/ld+json">{json_ld}</Script>

                                <div class="book-detail">
                                    <article>
                                        <h1>{book.title.clone()}</h1>
                                        <div class="book-meta">
                                            <span>{authors_str}</span>
                                            <span>{editor_str}</span>
                                            <span>{edition_date_str}</span>
                                        </div>

                                        <div>
                                            {book.tags.iter().map(|t| view! { <TagBadge tag=t.clone() /> }).collect_view()}
                                        </div>

                                        {book.isbn.clone().map(|isbn| view! { <p><strong>"ISBN : "</strong>{isbn}</p> })}
                                        {book.ean.clone().map(|ean| view! { <p><strong>"EAN : "</strong>{ean}</p> })}

                                        <ResellerLinks urls=book.reseller_urls.clone() />

                                        {book.summary.clone().map(|s| view! {
                                            <div class="book-summary">
                                                <h2>"Résumé"</h2>
                                                <div inner_html=s></div>
                                            </div>
                                        })}

                                        {book.introduction.clone().map(|intro| view! {
                                            <div class="book-introduction">
                                                <h2>"Introduction"</h2>
                                                <div inner_html=intro></div>
                                            </div>
                                        })}

                                        {book.cover_text.clone().map(|ct| view! {
                                            <div class="book-cover-text">
                                                <h2>"Quatrième de couverture"</h2>
                                                <div inner_html=ct></div>
                                            </div>
                                        })}

                                        {(!book.chapter_summaries.is_empty()).then(|| {
                                            view! {
                                                <div class="chapter-summaries">
                                                    <h2>"Résumés des chapitres"</h2>
                                                    {book.chapter_summaries.iter().map(|cs| {
                                                        let title = cs.title.clone().unwrap_or_else(|| format!("Chapitre {}", cs.chapter_idx + 1));
                                                        let summary_html = cs.summary.clone();
                                                        view! {
                                                            <div class="chapter">
                                                                <h3>{title}</h3>
                                                                <div inner_html=summary_html></div>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }
                                        })}
                                    </article>
                                </div>
                            }.into_any()
                        }
                        Ok(None) => {
                            view! { <p>"Livre non trouvé."</p> }.into_any()
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            view! { <p>"Erreur : " {msg}</p> }.into_any()
                        }
                    }
                })
            }}
        </Suspense>
    }
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}

fn build_json_ld(book: &crate::model::book::BookDetail) -> String {
    let authors_json: Vec<serde_json::Value> = book
        .authors
        .iter()
        .map(|a| {
            serde_json::json!({
                "@type": "Person",
                "name": a
            })
        })
        .collect();

    let mut ld = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Book",
        "name": book.title,
        "author": authors_json,
        "url": format!("https://bilbo.example.com/book/{}", book.reference),
    });

    if let Some(isbn) = &book.isbn {
        ld["isbn"] = serde_json::json!(isbn);
    }
    if let Some(editor) = &book.editor {
        ld["publisher"] = serde_json::json!({
            "@type": "Organization",
            "name": editor
        });
    }
    if let Some(summary) = &book.summary {
        ld["description"] = serde_json::json!(strip_html_tags(summary));
    }

    let mut offers: Vec<serde_json::Value> = Vec::new();
    for ru in &book.reseller_urls {
        let mut offer = serde_json::json!({
            "@type": "Offer",
            "url": ru.url,
            "availability": "https://schema.org/InStock",
        });
        if ru.kind == "paper" {
            offer["itemCondition"] = serde_json::json!("https://schema.org/NewCondition");
        }
        offers.push(offer);
    }
    if !offers.is_empty() {
        ld["offers"] = serde_json::json!(offers);
    }

    serde_json::to_string(&ld).unwrap_or_default()
}
