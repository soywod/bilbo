use leptos::prelude::*;

#[allow(unused_imports)]
use crate::model::book::{BookDetail, BookSearchResult, ChatMessage, ChatRole, ChatSource};

#[server]
pub async fn search_books(
    #[server(default)] query: String,
    #[server(default)] tags: Vec<String>,
    #[server(default)] author: Option<String>,
    #[server(default)] page: i64,
    #[server(default)] page_size: i64,
) -> Result<(Vec<BookSearchResult>, i64), ServerFnError> {
    let state = expect_context::<crate::server::state::AppState>();
    let tags_filter = if tags.is_empty() { vec![] } else { tags };
    let (mut books, total) = crate::server::db::search_books_fts(
        &state.pool,
        &query,
        &tags_filter,
        author.as_deref(),
        page,
        page_size,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // On page 0, boost with semantic results if query is non-empty and Mistral is configured
    let query_trimmed = query.trim();
    if page == 0 && !query_trimmed.is_empty() && !state.mistral_api_key.is_empty() {
        let semantic_results = crate::server::search::semantic_boost(
            &state,
            query_trimmed,
            &tags_filter,
            author.as_deref(),
            10,
        )
        .await;

        let fts_refs: std::collections::HashSet<String> =
            books.iter().map(|b| b.reference.clone()).collect();

        for book in semantic_results {
            if !fts_refs.contains(&book.reference) {
                books.push(book);
            }
        }
    }

    Ok((books, total))
}

#[server]
pub async fn get_book(reference: String) -> Result<Option<BookDetail>, ServerFnError> {
    use crate::server::markdown::markdown_to_html;

    let state = expect_context::<crate::server::state::AppState>();
    let book = crate::server::db::get_book_by_reference(&state.pool, &reference)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(book.map(|mut b| {
        b.summary = b.summary.map(|s| markdown_to_html(&s));
        b.introduction = b.introduction.map(|s| markdown_to_html(&s));
        b.cover_text = b.cover_text.map(|s| markdown_to_html(&s));
        for cs in &mut b.chapter_summaries {
            cs.summary = markdown_to_html(&cs.summary);
        }
        b
    }))
}

#[server]
pub async fn chat(messages: Vec<ChatMessage>) -> Result<ChatMessage, ServerFnError> {
    let state = expect_context::<crate::server::state::AppState>();

    if state.mistral_api_key.is_empty() {
        return Err(ServerFnError::new("Mistral API key not configured"));
    }

    let last_user_msg = messages
        .iter()
        .rev()
        .find(|m| m.role == ChatRole::User)
        .ok_or_else(|| ServerFnError::new("no user message"))?;

    let embeddings = crate::server::mistral::embed_texts(
        &state.http_client,
        &state.mistral_api_key,
        &[last_user_msg.content.clone()],
    )
    .await
    .map_err(|e| ServerFnError::new(e))?;

    let query_embedding = embeddings
        .into_iter()
        .next()
        .ok_or_else(|| ServerFnError::new("no embedding returned"))?;

    let results = crate::server::qdrant::search_similar(
        &state.qdrant,
        query_embedding,
        &[],
        None,
        5,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let context: String = results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            format!(
                "[Source {}: {} - {}]\n{}\n",
                i + 1,
                r.title,
                r.reference,
                r.chunk_text
            )
        })
        .collect();

    let sources: Vec<ChatSource> = results
        .iter()
        .map(|r| ChatSource {
            reference: r.reference.clone(),
            title: r.title.clone(),
            chunk_text: r.chunk_text.chars().take(200).collect(),
        })
        .collect();

    let msg_pairs: Vec<(String, String)> = messages
        .iter()
        .map(|m| {
            let role = match m.role {
                ChatRole::User => "user".to_string(),
                ChatRole::Assistant => "assistant".to_string(),
            };
            (role, m.content.clone())
        })
        .collect();

    let response = crate::server::mistral::rag_chat(
        &state.http_client,
        &state.mistral_api_key,
        &context,
        &msg_pairs,
    )
    .await
    .map_err(|e| ServerFnError::new(e))?;

    let html_response = crate::server::markdown::markdown_to_html(&response);

    Ok(ChatMessage {
        role: ChatRole::Assistant,
        content: html_response,
        sources,
    })
}

#[server]
pub async fn list_tags() -> Result<Vec<String>, ServerFnError> {
    let state = expect_context::<crate::server::state::AppState>();
    let tags = crate::server::db::list_all_tags(&state.pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(tags)
}

#[server]
pub async fn list_authors() -> Result<Vec<String>, ServerFnError> {
    let state = expect_context::<crate::server::state::AppState>();
    let authors = crate::server::db::list_all_authors(&state.pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(authors)
}
