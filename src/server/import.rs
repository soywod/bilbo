use std::path::Path;

use crate::server::{db, markdown, mistral, qdrant, state::AppState};

/// Run the import pipeline on all `.md` files in `data/`.
pub async fn run_import(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("data");
    let processed_dir = data_dir.join("processed");
    let failed_dir = data_dir.join("failed");

    tokio::fs::create_dir_all(&processed_dir).await?;
    tokio::fs::create_dir_all(&failed_dir).await?;

    // Ensure Qdrant collection exists
    qdrant::ensure_collection(&state.qdrant).await?;

    let mut entries = tokio::fs::read_dir(data_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        tracing::info!("processing {filename}");

        match import_file(state, &path).await {
            Ok(()) => {
                let dest = processed_dir.join(&filename);
                tokio::fs::rename(&path, &dest).await?;
                tracing::info!("{filename} -> processed/");
            }
            Err(e) => {
                tracing::error!("failed to import {filename}: {e}");
                let dest = failed_dir.join(&filename);
                tokio::fs::rename(&path, &dest).await?;
            }
        }
    }

    Ok(())
}

async fn import_file(state: &AppState, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let raw = tokio::fs::read_to_string(path).await?;
    let parsed = markdown::parse_markdown(&raw).map_err(|e| e.to_string())?;
    let fm = &parsed.frontmatter;

    // Check if book already exists
    let existing = db::find_book_by_ref_id(&state.pool, &fm.id).await?;

    match existing {
        Some((_id, ref existing_hash)) if existing_hash == &parsed.hash => {
            tracing::info!("book {} unchanged, skipping", fm.id);
            return Ok(());
        }
        _ => {}
    }

    // Generate summary via Mistral if not in frontmatter
    let summary = if fm.summary.is_some() {
        fm.summary.clone()
    } else if !state.mistral_api_key.is_empty() {
        let s = mistral::generate_summary(
            &state.http_client,
            &state.mistral_api_key,
            &parsed.content[..parsed.content.len().min(6000)],
        )
        .await
        .ok();
        s
    } else {
        None
    };

    // Upsert book in PostgreSQL
    let book_id = match existing {
        Some(_) => {
            db::update_book(
                &state.pool,
                &fm.id,
                &parsed.hash,
                &fm.title,
                &fm.authors,
                fm.editor.as_deref(),
                &fm.tags,
                fm.edition_date.as_deref(),
                summary.as_deref(),
                fm.introduction.as_deref(),
                fm.cover_text.as_deref(),
                fm.ean.as_deref(),
                fm.isbn.as_deref(),
                &parsed.content,
                &fm.reseller_paper_urls,
                &fm.reseller_digital_urls,
            )
            .await?
        }
        None => {
            db::insert_book(
                &state.pool,
                &fm.id,
                &parsed.hash,
                &fm.title,
                &fm.authors,
                fm.editor.as_deref(),
                &fm.tags,
                fm.edition_date.as_deref(),
                summary.as_deref(),
                fm.introduction.as_deref(),
                fm.cover_text.as_deref(),
                fm.ean.as_deref(),
                fm.isbn.as_deref(),
                &parsed.content,
                &fm.reseller_paper_urls,
                &fm.reseller_digital_urls,
            )
            .await?
        }
    };

    // Extract chapters and generate chapter summaries
    let chapters = markdown::extract_chapters(&parsed.content);

    if !state.mistral_api_key.is_empty() {
        let chapter_inputs: Vec<(Option<String>, String)> = chapters
            .iter()
            .map(|c| (c.title.clone(), c.text.clone()))
            .collect();

        let chapter_summaries = mistral::generate_chapter_summaries(
            &state.http_client,
            &state.mistral_api_key,
            &chapter_inputs,
        )
        .await
        .unwrap_or_default();

        let summaries_data: Vec<(i32, Option<&str>, &str)> = chapter_summaries
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_empty())
            .map(|(i, s)| {
                let title = chapters.get(i).and_then(|c| c.title.as_deref());
                (i as i32, title, s.as_str())
            })
            .collect();

        db::upsert_chapter_summaries(&state.pool, book_id, &summaries_data).await?;
    }

    // Chunk content and generate embeddings
    let chunks = markdown::chunk_text(&chapters);

    if !state.mistral_api_key.is_empty() && !chunks.is_empty() {
        let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();

        let embeddings = mistral::embed_texts(
            &state.http_client,
            &state.mistral_api_key,
            &texts,
        )
        .await
        .map_err(|e| format!("embedding error: {e}"))?;

        // Delete old points if updating
        if existing.is_some() {
            qdrant::delete_book_points(&state.qdrant, book_id).await?;
        }

        let chunk_data: Vec<(usize, Option<String>, usize, String)> = chunks
            .iter()
            .map(|c| {
                (
                    c.chapter_idx,
                    c.chapter_title.clone(),
                    c.chunk_index,
                    c.text.clone(),
                )
            })
            .collect();

        qdrant::upsert_chunks(
            &state.qdrant,
            book_id,
            &fm.id,
            &fm.title,
            &fm.authors,
            &fm.tags,
            &chunk_data,
            &embeddings,
        )
        .await?;
    }

    tracing::info!("imported book {} ({})", fm.id, fm.title);
    Ok(())
}
