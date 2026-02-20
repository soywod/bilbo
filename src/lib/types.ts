export interface BookSearchResult {
  id: string;
  reference: string;
  title: string;
  authors: string[];
  tags: string[];
  editor: string | null;
  edition_date: string | null;
  summary: string | null;
}

export interface BookDetail {
  id: string;
  reference: string;
  title: string;
  authors: string[];
  editor: string | null;
  tags: string[];
  edition_date: string | null;
  summary: string | null;
  introduction: string | null;
  cover_text: string | null;
  ean: string | null;
  isbn: string | null;
  reseller_urls: ResellerUrl[];
  chapter_summaries: ChapterSummary[];
}

export interface ResellerUrl {
  url: string;
  kind: string;
}

export interface ChapterSummary {
  chapter_idx: number;
  title: string | null;
  summary: string;
}

export interface DiscussionMessage {
  role: "user" | "assistant";
  content: string;
  sources: DiscussionSource[];
}

export interface DiscussionSource {
  reference: string;
  title: string;
  chunk_text: string;
}
