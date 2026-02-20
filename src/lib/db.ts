import postgres from "postgres";
import type {
  BookSearchResult,
  BookDetail,
  ChapterSummary,
  ResellerUrl,
} from "./types";

let sql: ReturnType<typeof postgres>;

export function getDb() {
  if (!sql) {
    sql = postgres(process.env.DATABASE_URL!);
  }
  return sql;
}

function buildSearchText(
  title: string,
  authors: string[],
  editor: string | null,
  content: string,
): string {
  return `${title} ${editor ?? ""} ${authors.join(" ")} ${content}`;
}

export async function insertBook(
  reference: string,
  hash: string,
  title: string,
  authors: string[],
  editor: string | null,
  tags: string[],
  editionDate: string | null,
  summary: string | null,
  introduction: string | null,
  coverText: string | null,
  ean: string | null,
  isbn: string | null,
  content: string,
  resellerPaperUrls: string[],
  resellerDigitalUrls: string[],
): Promise<string> {
  const db = getDb();
  const searchText = buildSearchText(title, authors, editor, content);

  const [row] = await db`
    INSERT INTO books (reference, hash, title, editor, edition_date, summary, introduction,
                       cover_text, ean, isbn, search_vector)
    VALUES (${reference}, ${hash}, ${title}, ${editor}, ${editionDate}, ${summary}, ${introduction},
            ${coverText}, ${ean}, ${isbn}, to_tsvector('french', ${searchText}))
    RETURNING id
  `;

  const bookId: string = row.id;

  await upsertAuthors(db, bookId, authors);
  await upsertTags(db, bookId, tags);
  await upsertResellerUrls(db, bookId, resellerPaperUrls, resellerDigitalUrls);

  return bookId;
}

export async function updateBook(
  reference: string,
  hash: string,
  title: string,
  authors: string[],
  editor: string | null,
  tags: string[],
  editionDate: string | null,
  summary: string | null,
  introduction: string | null,
  coverText: string | null,
  ean: string | null,
  isbn: string | null,
  content: string,
  resellerPaperUrls: string[],
  resellerDigitalUrls: string[],
): Promise<string> {
  const db = getDb();
  const searchText = buildSearchText(title, authors, editor, content);

  const [row] = await db`
    UPDATE books SET hash=${hash}, title=${title}, editor=${editor}, edition_date=${editionDate},
        summary=${summary}, introduction=${introduction}, cover_text=${coverText}, ean=${ean}, isbn=${isbn},
        search_vector=to_tsvector('french', ${searchText}), updated_at=NOW()
    WHERE reference=${reference}
    RETURNING id
  `;

  const bookId: string = row.id;

  await db`DELETE FROM book_authors WHERE book_id = ${bookId}`;
  await db`DELETE FROM book_tags WHERE book_id = ${bookId}`;
  await db`DELETE FROM reseller_urls WHERE book_id = ${bookId}`;

  await upsertAuthors(db, bookId, authors);
  await upsertTags(db, bookId, tags);
  await upsertResellerUrls(db, bookId, resellerPaperUrls, resellerDigitalUrls);

  return bookId;
}

async function upsertAuthors(
  db: ReturnType<typeof postgres>,
  bookId: string,
  authors: string[],
) {
  for (const name of authors) {
    const [row] = await db`
      INSERT INTO authors (name) VALUES (${name})
      ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
      RETURNING id
    `;
    await db`
      INSERT INTO book_authors (book_id, author_id) VALUES (${bookId}, ${row.id})
      ON CONFLICT DO NOTHING
    `;
  }
}

async function upsertTags(
  db: ReturnType<typeof postgres>,
  bookId: string,
  tags: string[],
) {
  for (const name of tags) {
    const [row] = await db`
      INSERT INTO tags (name) VALUES (${name})
      ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
      RETURNING id
    `;
    await db`
      INSERT INTO book_tags (book_id, tag_id) VALUES (${bookId}, ${row.id})
      ON CONFLICT DO NOTHING
    `;
  }
}

async function upsertResellerUrls(
  db: ReturnType<typeof postgres>,
  bookId: string,
  paperUrls: string[],
  digitalUrls: string[],
) {
  for (const url of paperUrls) {
    await db`INSERT INTO reseller_urls (book_id, url, type) VALUES (${bookId}, ${url}, 'paper')`;
  }
  for (const url of digitalUrls) {
    await db`INSERT INTO reseller_urls (book_id, url, type) VALUES (${bookId}, ${url}, 'digital')`;
  }
}

export async function findBookByReference(
  reference: string,
): Promise<{ id: string; hash: string } | null> {
  const db = getDb();
  const rows =
    await db`SELECT id, hash FROM books WHERE reference = ${reference}`;
  if (rows.length === 0) return null;
  return { id: rows[0].id, hash: rows[0].hash };
}

export async function upsertChapterSummaries(
  bookId: string,
  summaries: { chapterIdx: number; title: string | null; summary: string }[],
) {
  const db = getDb();
  await db`DELETE FROM chapter_summaries WHERE book_id = ${bookId}`;

  for (const s of summaries) {
    await db`
      INSERT INTO chapter_summaries (book_id, chapter_idx, title, summary)
      VALUES (${bookId}, ${s.chapterIdx}, ${s.title}, ${s.summary})
    `;
  }
}

export async function searchBooksFts(
  query: string,
  tags: string[],
  author: string | null,
  page: number,
  pageSize: number,
): Promise<{ books: BookSearchResult[]; total: number }> {
  const db = getDb();
  const offset = page * pageSize;
  const trimmed = query.trim();
  const tagsParam = tags.length > 0 ? tags : null;

  if (!trimmed) {
    const [countRow] = await db`
      SELECT COUNT(*)::int AS count FROM books b
      WHERE (${tagsParam}::text[] IS NULL OR EXISTS (
              SELECT 1 FROM book_tags bt
              JOIN tags t ON t.id = bt.tag_id
              WHERE bt.book_id = b.id AND t.name = ANY(${tagsParam}::text[])))
        AND (${author}::text IS NULL OR EXISTS (
              SELECT 1 FROM book_authors ba
              JOIN authors a ON a.id = ba.author_id
              WHERE ba.book_id = b.id AND a.name = ${author}))
    `;

    const rows = await db`
      SELECT b.id, b.reference, b.title, b.editor, b.edition_date, b.summary,
             (SELECT COALESCE(array_agg(DISTINCT a.name), ARRAY[]::text[])
              FROM book_authors ba JOIN authors a ON a.id = ba.author_id
              WHERE ba.book_id = b.id) AS authors,
             (SELECT COALESCE(array_agg(DISTINCT t.name), ARRAY[]::text[])
              FROM book_tags bt JOIN tags t ON t.id = bt.tag_id
              WHERE bt.book_id = b.id) AS tags
      FROM books b
      WHERE (${tagsParam}::text[] IS NULL OR EXISTS (
              SELECT 1 FROM book_tags bt
              JOIN tags t ON t.id = bt.tag_id
              WHERE bt.book_id = b.id AND t.name = ANY(${tagsParam}::text[])))
        AND (${author}::text IS NULL OR EXISTS (
              SELECT 1 FROM book_authors ba
              JOIN authors a ON a.id = ba.author_id
              WHERE ba.book_id = b.id AND a.name = ${author}))
      ORDER BY b.updated_at DESC
      LIMIT ${pageSize} OFFSET ${offset}
    `;

    return { books: rowsToSearchResults(rows), total: countRow.count };
  }

  const likePattern = `%${trimmed}%`;

  const [countRow] = await db`
    SELECT COUNT(*)::int AS count FROM books b
    WHERE (b.search_vector @@ plainto_tsquery('french', ${trimmed})
           OR b.title ILIKE ${likePattern}
           OR b.editor ILIKE ${likePattern})
      AND (${tagsParam}::text[] IS NULL OR EXISTS (
              SELECT 1 FROM book_tags bt
              JOIN tags t ON t.id = bt.tag_id
              WHERE bt.book_id = b.id AND t.name = ANY(${tagsParam}::text[])))
      AND (${author}::text IS NULL OR EXISTS (
              SELECT 1 FROM book_authors ba
              JOIN authors a ON a.id = ba.author_id
              WHERE ba.book_id = b.id AND a.name = ${author}))
  `;

  const rows = await db`
    SELECT b.id, b.reference, b.title, b.editor, b.edition_date, b.summary,
           (SELECT COALESCE(array_agg(DISTINCT a.name), ARRAY[]::text[])
            FROM book_authors ba JOIN authors a ON a.id = ba.author_id
            WHERE ba.book_id = b.id) AS authors,
           (SELECT COALESCE(array_agg(DISTINCT t.name), ARRAY[]::text[])
            FROM book_tags bt JOIN tags t ON t.id = bt.tag_id
            WHERE bt.book_id = b.id) AS tags
    FROM books b
    WHERE (b.search_vector @@ plainto_tsquery('french', ${trimmed})
           OR b.title ILIKE ${likePattern}
           OR b.editor ILIKE ${likePattern})
      AND (${tagsParam}::text[] IS NULL OR EXISTS (
              SELECT 1 FROM book_tags bt
              JOIN tags t ON t.id = bt.tag_id
              WHERE bt.book_id = b.id AND t.name = ANY(${tagsParam}::text[])))
      AND (${author}::text IS NULL OR EXISTS (
              SELECT 1 FROM book_authors ba
              JOIN authors a ON a.id = ba.author_id
              WHERE ba.book_id = b.id AND a.name = ${author}))
    ORDER BY b.updated_at DESC
    LIMIT ${pageSize} OFFSET ${offset}
  `;

  return { books: rowsToSearchResults(rows), total: countRow.count };
}

function rowsToSearchResults(
  rows: postgres.RowList<postgres.Row[]>,
): BookSearchResult[] {
  return rows.map((r) => ({
    id: r.id,
    reference: r.reference,
    title: r.title,
    authors: r.authors ?? [],
    tags: r.tags ?? [],
    editor: r.editor ?? null,
    edition_date: r.edition_date ?? null,
    summary: r.summary ?? null,
  }));
}

export async function getBookByReference(
  reference: string,
): Promise<BookDetail | null> {
  const db = getDb();

  const bookRows = await db`
    SELECT b.id, b.reference, b.title, b.editor, b.edition_date, b.summary,
           b.introduction, b.cover_text, b.ean, b.isbn,
           COALESCE(array_agg(DISTINCT a.name) FILTER (WHERE a.name IS NOT NULL), '{}') AS authors,
           COALESCE(array_agg(DISTINCT t.name) FILTER (WHERE t.name IS NOT NULL), '{}') AS tags
    FROM books b
    LEFT JOIN book_authors ba ON ba.book_id = b.id
    LEFT JOIN authors a ON a.id = ba.author_id
    LEFT JOIN book_tags bt ON bt.book_id = b.id
    LEFT JOIN tags t ON t.id = bt.tag_id
    WHERE b.reference = ${reference}
    GROUP BY b.id
  `;

  if (bookRows.length === 0) return null;
  const r = bookRows[0];
  const bookId: string = r.id;

  const chapterRows = await db`
    SELECT chapter_idx, title, summary FROM chapter_summaries
    WHERE book_id = ${bookId} ORDER BY chapter_idx
  `;

  const resellerRows = await db`
    SELECT url, type FROM reseller_urls
    WHERE book_id = ${bookId} ORDER BY type, url
  `;

  const chapters: ChapterSummary[] = chapterRows.map((c) => ({
    chapter_idx: c.chapter_idx,
    title: c.title ?? null,
    summary: c.summary,
  }));

  const resellerUrls: ResellerUrl[] = resellerRows.map((ru) => ({
    url: ru.url,
    kind: ru.type,
  }));

  return {
    id: bookId,
    reference: r.reference,
    title: r.title,
    authors: r.authors ?? [],
    editor: r.editor ?? null,
    tags: r.tags ?? [],
    edition_date: r.edition_date ?? null,
    summary: r.summary ?? null,
    introduction: r.introduction ?? null,
    cover_text: r.cover_text ?? null,
    ean: r.ean ?? null,
    isbn: r.isbn ?? null,
    reseller_urls: resellerUrls,
    chapter_summaries: chapters,
  };
}

export async function listAllTags(): Promise<string[]> {
  const db = getDb();
  const rows = await db`SELECT DISTINCT name FROM tags ORDER BY name`;
  return rows.map((r) => r.name);
}

export async function listAllAuthors(): Promise<string[]> {
  const db = getDb();
  const rows = await db`SELECT DISTINCT name FROM authors ORDER BY name`;
  return rows.map((r) => r.name);
}

export async function listAllBookReferences(): Promise<
  { reference: string; title: string }[]
> {
  const db = getDb();
  const rows = await db`SELECT reference, title FROM books ORDER BY title`;
  return rows.map((r) => ({ reference: r.reference, title: r.title }));
}
