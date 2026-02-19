use leptos::prelude::*;
use leptos_meta::*;

use crate::api::chat;
use crate::components::chat_widget::ChatWidget;
use crate::model::book::{ChatMessage, ChatRole};

#[component]
pub fn ChatPage() -> impl IntoView {
    let messages = RwSignal::new(Vec::<ChatMessage>::new());
    let loading = RwSignal::new(false);

    let on_send = Callback::new(move |content: String| {
        let user_msg = ChatMessage {
            role: ChatRole::User,
            content,
            sources: vec![],
        };

        messages.update(|msgs| msgs.push(user_msg));
        loading.set(true);

        let current_messages = messages.get();

        leptos::task::spawn_local(async move {
            match chat(current_messages).await {
                Ok(response) => {
                    messages.update(|msgs| msgs.push(response));
                }
                Err(e) => {
                    messages.update(|msgs| {
                        msgs.push(ChatMessage {
                            role: ChatRole::Assistant,
                            content: format!("Erreur : {e}"),
                            sources: vec![],
                        });
                    });
                }
            }
            loading.set(false);
        });
    });

    let messages_signal = Signal::derive(move || messages.get());
    let loading_signal = Signal::derive(move || loading.get());

    view! {
        <Title text="Chat — Bilbo" />
        <Meta name="description" content="Posez vos questions sur les livres de la bibliothèque numérique Bilbo." />

        <div class="chat-page">
            <h1>"Chat avec Bilbo"</h1>
            <p>"Posez vos questions sur les livres de la bibliothèque. Bilbo cherchera les passages pertinents pour vous répondre."</p>
            <ChatWidget messages=messages_signal on_send=on_send loading=loading_signal />
        </div>
    }
}
