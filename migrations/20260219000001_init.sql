CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE books (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    reference       TEXT NOT NULL UNIQUE,
    hash            TEXT NOT NULL,
    title           TEXT NOT NULL,
    editor          TEXT,
    edition_date    TEXT,
    summary         TEXT,
    introduction    TEXT,
    cover_text      TEXT,
    ean             TEXT,
    isbn            TEXT,
    search_vector   TSVECTOR,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE authors (
    id   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE tags (
    id   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE book_authors (
    book_id   UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, author_id)
);

CREATE TABLE book_tags (
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    tag_id  UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (book_id, tag_id)
);

CREATE TABLE reseller_urls (
    id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    url     TEXT NOT NULL,
    type    TEXT NOT NULL CHECK (type IN ('paper', 'digital'))
);

CREATE TABLE chapter_summaries (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    book_id     UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    chapter_idx INT NOT NULL,
    title       TEXT,
    summary     TEXT NOT NULL,
    UNIQUE(book_id, chapter_idx)
);

CREATE INDEX idx_books_search_vector ON books USING gin (search_vector);
CREATE INDEX idx_books_ean ON books (ean) WHERE ean IS NOT NULL;
CREATE INDEX idx_books_isbn ON books (isbn) WHERE isbn IS NOT NULL;
CREATE INDEX idx_reseller_urls_book_id ON reseller_urls (book_id);
