import { useState, useEffect } from "preact/hooks";
import type { BookSearchResult } from "../lib/types";

const PAGE_SIZE = 20;

export default function SearchIsland() {
  const [query, setQuery] = useState("");
  const [tags, setTags] = useState<string[]>([]);
  const [authors, setAuthors] = useState<string[]>([]);
  const [selectedTag, setSelectedTag] = useState("");
  const [selectedAuthor, setSelectedAuthor] = useState("");
  const [page, setPage] = useState(0);
  const [books, setBooks] = useState<BookSearchResult[]>([]);
  const [total, setTotal] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetch("/api/list-tags")
      .then((r) => r.json())
      .then(setTags)
      .catch(() => {});

    fetch("/api/list-authors")
      .then((r) => r.json())
      .then(setAuthors)
      .catch(() => {});
  }, []);

  const doSearch = async (p = 0) => {
    setLoading(true);
    setError(null);
    try {
      const resp = await fetch("/api/search", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          query,
          tags: selectedTag ? [selectedTag] : [],
          author: selectedAuthor || null,
          page: p,
          page_size: PAGE_SIZE,
        }),
      });
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const data = await resp.json();
      setBooks(data.books);
      setTotal(data.total);
      setPage(p);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    doSearch(0);
  }, []);

  const totalPages = total === 0 ? 1 : Math.ceil(total / PAGE_SIZE);

  const onSubmit = (e: Event) => {
    e.preventDefault();
    doSearch(0);
  };

  return (
    <div>
      <h1>Recherche par mots-clés</h1>

      <div class="search-section">
        <form class="search-bar" onSubmit={onSubmit}>
          <input
            type="text"
            placeholder="Rechercher un livre..."
            value={query}
            onInput={(e) => setQuery((e.target as HTMLInputElement).value)}
          />
          <button type="submit">Rechercher</button>
        </form>
      </div>

      <div class="search-filters">
        <select
          value={selectedTag}
          onChange={(e) =>
            setSelectedTag((e.target as HTMLSelectElement).value)
          }
        >
          <option value="">Tous les tags</option>
          {tags.map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </select>

        <select
          value={selectedAuthor}
          onChange={(e) =>
            setSelectedAuthor((e.target as HTMLSelectElement).value)
          }
        >
          <option value="">Tous les auteurs</option>
          {authors.map((a) => (
            <option key={a} value={a}>
              {a}
            </option>
          ))}
        </select>
      </div>

      {error && (
        <p class="search-error" style={{ color: "red" }}>
          Erreur de recherche : {error}
        </p>
      )}

      {loading ? (
        <p>Chargement...</p>
      ) : (
        <>
          <table class="book-table">
            <thead>
              <tr>
                <th>Titre</th>
                <th>Auteurs</th>
                <th>Éditeur</th>
                <th>Tags</th>
              </tr>
            </thead>
            <tbody>
              {books.map((book) => (
                <tr key={book.id}>
                  <td>
                    <a href={`/book/${book.reference}`}>{book.title}</a>
                  </td>
                  <td>{book.authors.join(", ")}</td>
                  <td>{book.editor ?? ""}</td>
                  <td>
                    {book.tags.map((t) => (
                      <span key={t} class="tag-badge">
                        {t}
                      </span>
                    ))}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          <div class="pagination">
            <button
              onClick={() => doSearch(Math.max(0, page - 1))}
              disabled={page === 0}
            >
              Précédent
            </button>
            <span>
              Page {page + 1} / {totalPages}
            </span>
            <button
              onClick={() => doSearch(page + 1)}
              disabled={page + 1 >= totalPages}
            >
              Suivant
            </button>
          </div>
        </>
      )}
    </div>
  );
}
