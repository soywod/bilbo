use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::model::book::{BookDetail, BookSearchResult, ChapterSummary};

pub async fn insert_book(
    pool: &PgPool,
    ref_id: &str,
    hash: &str,
    title: &str,
    authors: &[String],
    editor: Option<&str>,
    tags: &[String],
    edition_date: Option<&str>,
    summary: Option<&str>,
    introduction: Option<&str>,
    cover_text: Option<&str>,
    ean: Option<&str>,
    isbn: Option<&str>,
    content: &str,
    reseller_paper_urls: &[String],
    reseller_digital_urls: &[String],
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO books (ref_id, hash, title, authors, editor, tags, edition_date, summary, introduction, cover_text, ean, isbn, content, reseller_paper_urls, reseller_digital_urls)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id
        "#,
    )
    .bind(ref_id)
    .bind(hash)
    .bind(title)
    .bind(authors)
    .bind(editor)
    .bind(tags)
    .bind(edition_date)
    .bind(summary)
    .bind(introduction)
    .bind(cover_text)
    .bind(ean)
    .bind(isbn)
    .bind(content)
    .bind(reseller_paper_urls)
    .bind(reseller_digital_urls)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

pub async fn update_book(
    pool: &PgPool,
    ref_id: &str,
    hash: &str,
    title: &str,
    authors: &[String],
    editor: Option<&str>,
    tags: &[String],
    edition_date: Option<&str>,
    summary: Option<&str>,
    introduction: Option<&str>,
    cover_text: Option<&str>,
    ean: Option<&str>,
    isbn: Option<&str>,
    content: &str,
    reseller_paper_urls: &[String],
    reseller_digital_urls: &[String],
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        r#"
        UPDATE books SET hash=$2, title=$3, authors=$4, editor=$5, tags=$6, edition_date=$7,
            summary=$8, introduction=$9, cover_text=$10, ean=$11, isbn=$12, content=$13,
            reseller_paper_urls=$14, reseller_digital_urls=$15, updated_at=NOW()
        WHERE ref_id=$1
        RETURNING id
        "#,
    )
    .bind(ref_id)
    .bind(hash)
    .bind(title)
    .bind(authors)
    .bind(editor)
    .bind(tags)
    .bind(edition_date)
    .bind(summary)
    .bind(introduction)
    .bind(cover_text)
    .bind(ean)
    .bind(isbn)
    .bind(content)
    .bind(reseller_paper_urls)
    .bind(reseller_digital_urls)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

pub async fn find_book_by_ref_id(
    pool: &PgPool,
    ref_id: &str,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    let row: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, hash FROM books WHERE ref_id = $1",
    )
    .bind(ref_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn upsert_chapter_summaries(
    pool: &PgPool,
    book_id: Uuid,
    summaries: &[(i32, Option<&str>, &str)],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chapter_summaries WHERE book_id = $1")
        .bind(book_id)
        .execute(pool)
        .await?;

    for (idx, title, summary) in summaries {
        sqlx::query(
            "INSERT INTO chapter_summaries (book_id, chapter_idx, title, summary) VALUES ($1, $2, $3, $4)",
        )
        .bind(book_id)
        .bind(idx)
        .bind(*title)
        .bind(*summary)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn search_books_fts(
    pool: &PgPool,
    query: &str,
    tags: &[String],
    author: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<(Vec<BookSearchResult>, i64), sqlx::Error> {
    let offset = page * page_size;

    let tags_param: Option<&[String]> = if tags.is_empty() { None } else { Some(tags) };

    let count_row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM books
        WHERE ($1 = '' OR to_tsvector('french', coalesce(title,'') || ' ' || coalesce(summary,'') || ' ' || coalesce(introduction,''))
              @@ plainto_tsquery('french', $1))
          AND ($2::text[] IS NULL OR tags && $2)
          AND ($3::text IS NULL OR $3 = ANY(authors))
        "#,
    )
    .bind(query)
    .bind(tags_param)
    .bind(author)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT id, ref_id, title, authors, tags, editor, edition_date, summary
        FROM books
        WHERE ($1 = '' OR to_tsvector('french', coalesce(title,'') || ' ' || coalesce(summary,'') || ' ' || coalesce(introduction,''))
              @@ plainto_tsquery('french', $1))
          AND ($2::text[] IS NULL OR tags && $2)
          AND ($3::text IS NULL OR $3 = ANY(authors))
        ORDER BY updated_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(query)
    .bind(tags_param)
    .bind(author)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let books = rows
        .iter()
        .map(|r| BookSearchResult {
            id: r.get("id"),
            ref_id: r.get("ref_id"),
            title: r.get("title"),
            authors: r.get("authors"),
            tags: r.get("tags"),
            editor: r.get("editor"),
            edition_date: r.get("edition_date"),
            summary: r.get("summary"),
        })
        .collect();

    Ok((books, count_row.0))
}

pub async fn get_book_by_ref_id(
    pool: &PgPool,
    ref_id: &str,
) -> Result<Option<BookDetail>, sqlx::Error> {
    let book_row = sqlx::query(
        r#"
        SELECT id, ref_id, title, authors, editor, tags, edition_date, summary,
               introduction, cover_text, ean, isbn, content,
               reseller_paper_urls, reseller_digital_urls
        FROM books WHERE ref_id = $1
        "#,
    )
    .bind(ref_id)
    .fetch_optional(pool)
    .await?;

    let book_row = match book_row {
        Some(r) => r,
        None => return Ok(None),
    };

    let book_id: Uuid = book_row.get("id");

    let chapter_rows = sqlx::query(
        "SELECT chapter_idx, title, summary FROM chapter_summaries WHERE book_id = $1 ORDER BY chapter_idx",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await?;

    let chapters = chapter_rows
        .iter()
        .map(|r| ChapterSummary {
            chapter_idx: r.get("chapter_idx"),
            title: r.get("title"),
            summary: r.get("summary"),
        })
        .collect();

    Ok(Some(BookDetail {
        id: book_id,
        ref_id: book_row.get("ref_id"),
        title: book_row.get("title"),
        authors: book_row.get("authors"),
        editor: book_row.get("editor"),
        tags: book_row.get("tags"),
        edition_date: book_row.get("edition_date"),
        summary: book_row.get("summary"),
        introduction: book_row.get("introduction"),
        cover_text: book_row.get("cover_text"),
        ean: book_row.get("ean"),
        isbn: book_row.get("isbn"),
        content: book_row.get("content"),
        reseller_paper_urls: book_row.get("reseller_paper_urls"),
        reseller_digital_urls: book_row.get("reseller_digital_urls"),
        chapter_summaries: chapters,
    }))
}

pub async fn list_all_tags(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT unnest(tags) AS tag FROM books ORDER BY 1",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn list_all_authors(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT unnest(authors) AS author FROM books ORDER BY 1",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn list_all_book_refs(pool: &PgPool) -> Result<Vec<(String, String)>, sqlx::Error> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT ref_id, title FROM books ORDER BY title",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
