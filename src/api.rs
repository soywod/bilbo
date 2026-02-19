use leptos::prelude::*;

#[allow(unused_imports)]
use crate::model::book::{BookDetail, BookSearchResult, ChatMessage, ChatRole, ChatSource};

#[server]
pub async fn search_books(
    query: String,
    tags: Vec<String>,
    author: Option<String>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<BookSearchResult>, i64), ServerFnError> {
    let state = expect_context::<crate::server::state::AppState>();
    let tags_filter = if tags.is_empty() { vec![] } else { tags };
    let (books, total) = crate::server::db::search_books_fts(
        &state.pool,
        &query,
        &tags_filter,
        author.as_deref(),
        page,
        page_size,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok((books, total))
}

#[server]
pub async fn semantic_search(
    query: String,
    tags: Vec<String>,
    limit: u64,
) -> Result<Vec<BookSearchResult>, ServerFnError> {
    let state = expect_context::<crate::server::state::AppState>();

    if state.mistral_api_key.is_empty() {
        return Err(ServerFnError::new("Mistral API key not configured"));
    }

    let embeddings = crate::server::mistral::embed_texts(
        &state.http_client,
        &state.mistral_api_key,
        &[query],
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
        &tags,
        limit,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut seen = std::collections::HashSet::new();
    let mut books = Vec::new();

    for result in results {
        if seen.insert(result.ref_id.clone()) {
            let book = crate::server::db::get_book_by_ref_id(&state.pool, &result.ref_id)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;

            if let Some(b) = book {
                books.push(BookSearchResult {
                    id: b.id,
                    ref_id: b.ref_id,
                    title: b.title,
                    authors: b.authors,
                    tags: b.tags,
                    editor: b.editor,
                    edition_date: b.edition_date,
                    summary: b.summary,
                });
            }
        }
    }

    Ok(books)
}

#[server]
pub async fn get_book(ref_id: String) -> Result<Option<BookDetail>, ServerFnError> {
    let state = expect_context::<crate::server::state::AppState>();
    let book = crate::server::db::get_book_by_ref_id(&state.pool, &ref_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(book)
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
                r.ref_id,
                r.chunk_text
            )
        })
        .collect();

    let sources: Vec<ChatSource> = results
        .iter()
        .map(|r| ChatSource {
            ref_id: r.ref_id.clone(),
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

    Ok(ChatMessage {
        role: ChatRole::Assistant,
        content: response,
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
