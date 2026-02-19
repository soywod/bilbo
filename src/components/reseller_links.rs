use leptos::prelude::*;

#[component]
pub fn ResellerLinks(
    paper_urls: Vec<String>,
    digital_urls: Vec<String>,
) -> impl IntoView {
    let has_links = !paper_urls.is_empty() || !digital_urls.is_empty();

    if !has_links {
        return view! { <div></div> }.into_any();
    }

    view! {
        <div class="reseller-links">
            <h3>"Acheter ce livre"</h3>
            {if !paper_urls.is_empty() {
                Some(view! {
                    <div class="link-group">
                        <span>"Format papier : "</span>
                        {paper_urls.into_iter().enumerate().map(|(i, url)| {
                            let label = format!("Revendeur {}", i + 1);
                            view! {
                                <a href=url target="_blank" rel="noopener noreferrer">{label}</a>
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
                        {digital_urls.into_iter().enumerate().map(|(i, url)| {
                            let label = format!("Revendeur {}", i + 1);
                            view! {
                                <a href=url target="_blank" rel="noopener noreferrer">{label}</a>
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
