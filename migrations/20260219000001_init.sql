CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE books (
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    ref_id                TEXT NOT NULL UNIQUE,
    hash                  TEXT NOT NULL,
    title                 TEXT NOT NULL,
    authors               TEXT[] NOT NULL DEFAULT '{}',
    editor                TEXT,
    tags                  TEXT[] NOT NULL DEFAULT '{}',
    edition_date          TEXT,
    summary               TEXT,
    introduction          TEXT,
    cover_text            TEXT,
    ean                   TEXT,
    isbn                  TEXT,
    content               TEXT NOT NULL,
    reseller_paper_urls   TEXT[] NOT NULL DEFAULT '{}',
    reseller_digital_urls TEXT[] NOT NULL DEFAULT '{}',
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE chapter_summaries (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    book_id     UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    chapter_idx INT NOT NULL,
    title       TEXT,
    summary     TEXT NOT NULL,
    UNIQUE(book_id, chapter_idx)
);

CREATE INDEX idx_books_fts ON books USING gin (
    to_tsvector('french', coalesce(title,'') || ' ' || coalesce(summary,'') || ' ' || coalesce(introduction,''))
);
CREATE INDEX idx_books_authors ON books USING gin (authors);
CREATE INDEX idx_books_tags ON books USING gin (tags);
CREATE INDEX idx_books_ean ON books (ean) WHERE ean IS NOT NULL;
CREATE INDEX idx_books_isbn ON books (isbn) WHERE isbn IS NOT NULL;
