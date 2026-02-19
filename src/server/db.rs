use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::model::book::{BookDetail, BookSearchResult, ChapterSummary, ResellerUrl};

pub async fn insert_book(
    pool: &PgPool,
    reference: &str,
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
    let search_text = build_search_text(title, authors, editor, content);

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO books (reference, hash, title, editor, edition_date, summary, introduction,
                           cover_text, ean, isbn, search_vector)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, to_tsvector('french', $11))
        RETURNING id
        "#,
    )
    .bind(reference)
    .bind(hash)
    .bind(title)
    .bind(editor)
    .bind(edition_date)
    .bind(summary)
    .bind(introduction)
    .bind(cover_text)
    .bind(ean)
    .bind(isbn)
    .bind(&search_text)
    .fetch_one(pool)
    .await?;

    let book_id = row.0;

    upsert_authors(pool, book_id, authors).await?;
    upsert_tags(pool, book_id, tags).await?;
    upsert_reseller_urls(pool, book_id, reseller_paper_urls, reseller_digital_urls).await?;

    Ok(book_id)
}

pub async fn update_book(
    pool: &PgPool,
    reference: &str,
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
    let search_text = build_search_text(title, authors, editor, content);

    let row: (Uuid,) = sqlx::query_as(
        r#"
        UPDATE books SET hash=$2, title=$3, editor=$4, edition_date=$5,
            summary=$6, introduction=$7, cover_text=$8, ean=$9, isbn=$10,
            search_vector=to_tsvector('french', $11), updated_at=NOW()
        WHERE reference=$1
        RETURNING id
        "#,
    )
    .bind(reference)
    .bind(hash)
    .bind(title)
    .bind(editor)
    .bind(edition_date)
    .bind(summary)
    .bind(introduction)
    .bind(cover_text)
    .bind(ean)
    .bind(isbn)
    .bind(&search_text)
    .fetch_one(pool)
    .await?;

    let book_id = row.0;

    // Clear old junction data and re-insert
    sqlx::query("DELETE FROM book_authors WHERE book_id = $1")
        .bind(book_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM book_tags WHERE book_id = $1")
        .bind(book_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM reseller_urls WHERE book_id = $1")
        .bind(book_id)
        .execute(pool)
        .await?;

    upsert_authors(pool, book_id, authors).await?;
    upsert_tags(pool, book_id, tags).await?;
    upsert_reseller_urls(pool, book_id, reseller_paper_urls, reseller_digital_urls).await?;

    Ok(book_id)
}

async fn upsert_authors(
    pool: &PgPool,
    book_id: Uuid,
    authors: &[String],
) -> Result<(), sqlx::Error> {
    for name in authors {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO authors (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        sqlx::query("INSERT INTO book_authors (book_id, author_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(book_id)
            .bind(row.0)
            .execute(pool)
            .await?;
    }
    Ok(())
}

async fn upsert_tags(
    pool: &PgPool,
    book_id: Uuid,
    tags: &[String],
) -> Result<(), sqlx::Error> {
    for name in tags {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO tags (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        sqlx::query("INSERT INTO book_tags (book_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(book_id)
            .bind(row.0)
            .execute(pool)
            .await?;
    }
    Ok(())
}

async fn upsert_reseller_urls(
    pool: &PgPool,
    book_id: Uuid,
    paper_urls: &[String],
    digital_urls: &[String],
) -> Result<(), sqlx::Error> {
    for url in paper_urls {
        sqlx::query("INSERT INTO reseller_urls (book_id, url, type) VALUES ($1, $2, 'paper')")
            .bind(book_id)
            .bind(url)
            .execute(pool)
            .await?;
    }
    for url in digital_urls {
        sqlx::query("INSERT INTO reseller_urls (book_id, url, type) VALUES ($1, $2, 'digital')")
            .bind(book_id)
            .bind(url)
            .execute(pool)
            .await?;
    }
    Ok(())
}

fn build_search_text(title: &str, authors: &[String], editor: Option<&str>, content: &str) -> String {
    format!(
        "{} {} {} {}",
        title,
        editor.unwrap_or(""),
        authors.join(" "),
        content,
    )
}

pub async fn find_book_by_reference(
    pool: &PgPool,
    reference: &str,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    let row: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, hash FROM books WHERE reference = $1",
    )
    .bind(reference)
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
    let query = query.trim();

    let tags_param: Option<&[String]> = if tags.is_empty() { None } else { Some(tags) };

    if query.is_empty() {
        // No text query: return all books, optionally filtered by tag/author
        let count_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM books b
            WHERE ($1::text[] IS NULL OR EXISTS (
                    SELECT 1 FROM book_tags bt
                    JOIN tags t ON t.id = bt.tag_id
                    WHERE bt.book_id = b.id AND t.name = ANY($1)))
              AND ($2::text IS NULL OR EXISTS (
                    SELECT 1 FROM book_authors ba
                    JOIN authors a ON a.id = ba.author_id
                    WHERE ba.book_id = b.id AND a.name = $2))
            "#,
        )
        .bind(tags_param)
        .bind(author)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query(
            r#"
            SELECT b.id, b.reference, b.title, b.editor, b.edition_date, b.summary,
                   (SELECT COALESCE(array_agg(DISTINCT a.name), ARRAY[]::text[])
                    FROM book_authors ba JOIN authors a ON a.id = ba.author_id
                    WHERE ba.book_id = b.id) AS authors,
                   (SELECT COALESCE(array_agg(DISTINCT t.name), ARRAY[]::text[])
                    FROM book_tags bt JOIN tags t ON t.id = bt.tag_id
                    WHERE bt.book_id = b.id) AS tags
            FROM books b
            WHERE ($1::text[] IS NULL OR EXISTS (
                    SELECT 1 FROM book_tags bt
                    JOIN tags t ON t.id = bt.tag_id
                    WHERE bt.book_id = b.id AND t.name = ANY($1)))
              AND ($2::text IS NULL OR EXISTS (
                    SELECT 1 FROM book_authors ba
                    JOIN authors a ON a.id = ba.author_id
                    WHERE ba.book_id = b.id AND a.name = $2))
            ORDER BY b.updated_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(tags_param)
        .bind(author)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let books = rows_to_search_results(&rows);
        Ok((books, count_row.0))
    } else {
        // Text query: FTS + ILIKE on title/editor, optionally filtered by tag/author
        let like_pattern = format!("%{query}%");

        let count_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM books b
            WHERE (b.search_vector @@ plainto_tsquery('french', $1)
                   OR b.title ILIKE $2
                   OR b.editor ILIKE $2)
              AND ($3::text[] IS NULL OR EXISTS (
                    SELECT 1 FROM book_tags bt
                    JOIN tags t ON t.id = bt.tag_id
                    WHERE bt.book_id = b.id AND t.name = ANY($3)))
              AND ($4::text IS NULL OR EXISTS (
                    SELECT 1 FROM book_authors ba
                    JOIN authors a ON a.id = ba.author_id
                    WHERE ba.book_id = b.id AND a.name = $4))
            "#,
        )
        .bind(query)
        .bind(&like_pattern)
        .bind(tags_param)
        .bind(author)
        .fetch_one(pool)
        .await?;

        let rows = sqlx::query(
            r#"
            SELECT b.id, b.reference, b.title, b.editor, b.edition_date, b.summary,
                   (SELECT COALESCE(array_agg(DISTINCT a.name), ARRAY[]::text[])
                    FROM book_authors ba JOIN authors a ON a.id = ba.author_id
                    WHERE ba.book_id = b.id) AS authors,
                   (SELECT COALESCE(array_agg(DISTINCT t.name), ARRAY[]::text[])
                    FROM book_tags bt JOIN tags t ON t.id = bt.tag_id
                    WHERE bt.book_id = b.id) AS tags
            FROM books b
            WHERE (b.search_vector @@ plainto_tsquery('french', $1)
                   OR b.title ILIKE $2
                   OR b.editor ILIKE $2)
              AND ($3::text[] IS NULL OR EXISTS (
                    SELECT 1 FROM book_tags bt
                    JOIN tags t ON t.id = bt.tag_id
                    WHERE bt.book_id = b.id AND t.name = ANY($3)))
              AND ($4::text IS NULL OR EXISTS (
                    SELECT 1 FROM book_authors ba
                    JOIN authors a ON a.id = ba.author_id
                    WHERE ba.book_id = b.id AND a.name = $4))
            ORDER BY b.updated_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(query)
        .bind(&like_pattern)
        .bind(tags_param)
        .bind(author)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let books = rows_to_search_results(&rows);
        Ok((books, count_row.0))
    }
}

fn rows_to_search_results(rows: &[sqlx::postgres::PgRow]) -> Vec<BookSearchResult> {
    rows.iter()
        .map(|r| BookSearchResult {
            id: r.get("id"),
            reference: r.get("reference"),
            title: r.get("title"),
            authors: r.get("authors"),
            tags: r.get("tags"),
            editor: r.get("editor"),
            edition_date: r.get("edition_date"),
            summary: r.get("summary"),
        })
        .collect()
}

pub async fn get_book_by_reference(
    pool: &PgPool,
    reference: &str,
) -> Result<Option<BookDetail>, sqlx::Error> {
    let book_row = sqlx::query(
        r#"
        SELECT b.id, b.reference, b.title, b.editor, b.edition_date, b.summary,
               b.introduction, b.cover_text, b.ean, b.isbn,
               COALESCE(array_agg(DISTINCT a.name) FILTER (WHERE a.name IS NOT NULL), '{}') AS authors,
               COALESCE(array_agg(DISTINCT t.name) FILTER (WHERE t.name IS NOT NULL), '{}') AS tags
        FROM books b
        LEFT JOIN book_authors ba ON ba.book_id = b.id
        LEFT JOIN authors a ON a.id = ba.author_id
        LEFT JOIN book_tags bt ON bt.book_id = b.id
        LEFT JOIN tags t ON t.id = bt.tag_id
        WHERE b.reference = $1
        GROUP BY b.id
        "#,
    )
    .bind(reference)
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

    let reseller_rows = sqlx::query(
        "SELECT url, type FROM reseller_urls WHERE book_id = $1 ORDER BY type, url",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await?;

    let reseller_urls = reseller_rows
        .iter()
        .map(|r| ResellerUrl {
            url: r.get("url"),
            kind: r.get("type"),
        })
        .collect();

    Ok(Some(BookDetail {
        id: book_id,
        reference: book_row.get("reference"),
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
        reseller_urls,
        chapter_summaries: chapters,
    }))
}

pub async fn list_all_tags(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT name FROM tags ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn list_all_authors(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT name FROM authors ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn list_all_book_references(pool: &PgPool) -> Result<Vec<(String, String)>, sqlx::Error> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT reference, title FROM books ORDER BY title",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
