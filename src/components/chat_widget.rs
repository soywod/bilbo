use leptos::prelude::*;

use crate::model::book::{ChatMessage, ChatRole};

#[component]
pub fn ChatWidget(
    messages: Signal<Vec<ChatMessage>>,
    on_send: Callback<String>,
    loading: Signal<bool>,
) -> impl IntoView {
    let input = RwSignal::new(String::new());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let msg = input.get().trim().to_string();
        if !msg.is_empty() {
            on_send.run(msg);
            input.set(String::new());
        }
    };

    view! {
        <div class="chat-messages">
            {move || {
                messages.get().into_iter().map(|msg| {
                    let role_class = match msg.role {
                        ChatRole::User => "chat-message user",
                        ChatRole::Assistant => "chat-message assistant",
                    };
                    let role_label = match msg.role {
                        ChatRole::User => "Vous",
                        ChatRole::Assistant => "Bilbo",
                    };
                    let content = msg.content.clone();
                    let is_assistant = msg.role == ChatRole::Assistant;
                    let sources = msg.sources.clone();

                    view! {
                        <div class=role_class>
                            <div class="message-role">{role_label}</div>
                            {if is_assistant {
                                view! { <div class="message-content" inner_html=content></div> }.into_any()
                            } else {
                                view! { <div class="message-content">{content}</div> }.into_any()
                            }}
                            {(!sources.is_empty()).then(|| {
                                view! {
                                    <div class="message-sources">
                                        <strong>"Sources : "</strong>
                                        {sources.iter().map(|s| {
                                            let href = format!("/book/{}", s.reference);
                                            let title = s.title.clone();
                                            view! {
                                                <a href=href>{title}</a>
                                            }
                                        }).collect_view()}
                                    </div>
                                }
                            })}
                        </div>
                    }
                }).collect_view()
            }}
            {move || loading.get().then(|| view! {
                <div class="chat-message assistant">
                    <div class="message-role">"Bilbo"</div>
                    <div class="message-content">"Réflexion en cours..."</div>
                </div>
            })}
        </div>
        <form class="chat-input" on:submit=on_submit>
            <input
                type="text"
                placeholder="Posez une question sur les livres..."
                prop:value=move || input.get()
                on:input=move |ev| input.set(event_target_value(&ev))
            />
            <button type="submit" disabled=move || loading.get()>
                "Envoyer"
            </button>
        </form>
    }
}
