use std::collections::HashSet;

use crate::model::book::BookSearchResult;
use crate::server::state::AppState;

pub async fn semantic_boost(
    state: &AppState,
    query: &str,
    tags: &[String],
    author: Option<&str>,
    limit: u64,
) -> Vec<BookSearchResult> {
    let embeddings = match crate::server::mistral::embed_texts(
        &state.http_client,
        &state.mistral_api_key,
        &[query.to_string()],
    )
    .await
    {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let query_embedding = match embeddings.into_iter().next() {
        Some(e) => e,
        None => return vec![],
    };

    let results = match crate::server::qdrant::search_similar(
        &state.qdrant,
        query_embedding,
        tags,
        author,
        limit,
    )
    .await
    {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let mut seen = HashSet::new();
    let mut books = Vec::new();

    for result in results {
        if !seen.insert(result.reference.clone()) {
            continue;
        }

        let book = match crate::server::db::get_book_by_reference(&state.pool, &result.reference)
            .await
        {
            Ok(Some(b)) => b,
            _ => continue,
        };

        books.push(BookSearchResult {
            id: book.id,
            reference: book.reference,
            title: book.title,
            authors: book.authors,
            tags: book.tags,
            editor: book.editor,
            edition_date: book.edition_date,
            summary: book.summary,
        });
    }

    books
}
