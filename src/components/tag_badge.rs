use leptos::prelude::*;

#[component]
pub fn TagBadge(
    tag: String,
    #[prop(optional)] on_click: Option<Callback<String>>,
) -> impl IntoView {
    let tag_clone = tag.clone();
    let on_click_handler = move |_| {
        if let Some(cb) = &on_click {
            cb.run(tag_clone.clone());
        }
    };

    view! {
        <span class="tag-badge" on:click=on_click_handler>
            {tag}
        </span>
    }
}
