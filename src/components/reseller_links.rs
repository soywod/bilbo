use leptos::prelude::*;

use crate::model::book::ResellerUrl;

#[component]
pub fn ResellerLinks(urls: Vec<ResellerUrl>) -> impl IntoView {
    if urls.is_empty() {
        return view! { <div></div> }.into_any();
    }

    let paper_urls: Vec<_> = urls.iter().filter(|u| u.kind == "paper").cloned().collect();
    let digital_urls: Vec<_> = urls.iter().filter(|u| u.kind == "digital").cloned().collect();

    view! {
        <div class="reseller-links">
            <h3>"Acheter ce livre"</h3>
            {if !paper_urls.is_empty() {
                Some(view! {
                    <div class="link-group">
                        <span>"Format papier : "</span>
                        {paper_urls.into_iter().enumerate().map(|(i, ru)| {
                            let label = format!("Revendeur {}", i + 1);
                            view! {
                                <a href=ru.url target="_blank" rel="noopener noreferrer">{label}</a>
                            }
                        }).collect_view()}
                    </div>
                })
            } else {
                None
            }}
            {if !digital_urls.is_empty() {
                Some(view! {
                    <div class="link-group">
                        <span>"Format numérique : "</span>
                        {digital_urls.into_iter().enumerate().map(|(i, ru)| {
                            let label = format!("Revendeur {}", i + 1);
                            view! {
                                <a href=ru.url target="_blank" rel="noopener noreferrer">{label}</a>
                            }
                        }).collect_view()}
                    </div>
                })
            } else {
                None
            }}
        </div>
    }
    .into_any()
}
