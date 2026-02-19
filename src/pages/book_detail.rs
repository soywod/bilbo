use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

use crate::api::get_book;
use crate::components::reseller_links::ResellerLinks;
use crate::components::tag_badge::TagBadge;

#[component]
pub fn BookDetailPage() -> impl IntoView {
    let params = use_params_map();
    let ref_id = move || params.read().get("ref_id").unwrap_or_default();

    let book_resource = Resource::new(ref_id, |ref_id| get_book(ref_id));

    view! {
        <Suspense fallback=|| view! { <p>"Chargement..."</p> }>
            {move || {
                book_resource.get().map(|result| {
                    match result {
                        Ok(Some(book)) => {
                            let description = book.summary.clone().unwrap_or_else(|| {
                                format!("{} par {}", book.title, book.authors.join(", "))
                            });
                            let json_ld = build_json_ld(&book);
                            let title = book.title.clone();
                            let og_title = book.title.clone();
                            let og_desc = description.clone();
                            let og_url = format!("https://bilbo.example.com/book/{}", book.ref_id);
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

                                        <ResellerLinks
                                            paper_urls=book.reseller_paper_urls.clone()
                                            digital_urls=book.reseller_digital_urls.clone()
                                        />

                                        {book.summary.clone().map(|s| view! {
                                            <div class="book-summary">
                                                <h2>"Résumé"</h2>
                                                <p>{s}</p>
                                            </div>
                                        })}

                                        {book.introduction.clone().map(|intro| view! {
                                            <div class="book-introduction">
                                                <h2>"Introduction"</h2>
                                                <p>{intro}</p>
                                            </div>
                                        })}

                                        {book.cover_text.clone().map(|ct| view! {
                                            <div class="book-cover-text">
                                                <h2>"Quatrième de couverture"</h2>
                                                <p>{ct}</p>
                                            </div>
                                        })}

                                        {(!book.chapter_summaries.is_empty()).then(|| {
                                            view! {
                                                <div class="chapter-summaries">
                                                    <h2>"Résumés des chapitres"</h2>
                                                    {book.chapter_summaries.iter().map(|cs| {
                                                        let title = cs.title.clone().unwrap_or_else(|| format!("Chapitre {}", cs.chapter_idx + 1));
                                                        let summary = cs.summary.clone();
                                                        view! {
                                                            <div class="chapter">
                                                                <h3>{title}</h3>
                                                                <p>{summary}</p>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }
                                        })}

                                        <div class="book-content">
                                            <h2>"Contenu"</h2>
                                            <div class="content-body">{book.content.clone()}</div>
                                        </div>
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
        "url": format!("https://bilbo.example.com/book/{}", book.ref_id),
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
        ld["description"] = serde_json::json!(summary);
    }

    let mut offers: Vec<serde_json::Value> = Vec::new();
    for url in &book.reseller_paper_urls {
        offers.push(serde_json::json!({
            "@type": "Offer",
            "url": url,
            "availability": "https://schema.org/InStock",
            "itemCondition": "https://schema.org/NewCondition"
        }));
    }
    for url in &book.reseller_digital_urls {
        offers.push(serde_json::json!({
            "@type": "Offer",
            "url": url,
            "availability": "https://schema.org/InStock"
        }));
    }
    if !offers.is_empty() {
        ld["offers"] = serde_json::json!(offers);
    }

    serde_json::to_string(&ld).unwrap_or_default()
}
