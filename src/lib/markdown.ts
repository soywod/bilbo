import matter from "gray-matter";
import { marked } from "marked";
import { createHash } from "crypto";

export interface BookFrontmatter {
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
  reseller_paper_urls: string[];
  reseller_digital_urls: string[];
}

export interface ParsedBook {
  frontmatter: BookFrontmatter;
  content: string;
  hash: string;
}

export function parseMarkdown(raw: string): ParsedBook {
  const { data, content } = matter(raw);

  const frontmatter: BookFrontmatter = {
    reference: data.reference,
    title: data.title,
    authors: data.authors ?? [],
    editor: data.editor ?? null,
    tags: data.tags ?? [],
    edition_date: data.edition_date ?? null,
    summary: data.summary ?? null,
    introduction: data.introduction ?? null,
    cover_text: data.cover_text ?? null,
    ean: data.ean ?? null,
    isbn: data.isbn ?? null,
    reseller_paper_urls: data.reseller_paper_urls ?? [],
    reseller_digital_urls: data.reseller_digital_urls ?? [],
  };

  const hash = createHash("sha256").update(raw).digest("hex");

  return { frontmatter, content: content.trim(), hash };
}

export interface Chapter {
  title: string | null;
  text: string;
}

export function extractChapters(content: string): Chapter[] {
  const lines = content.split("\n");
  const chapters: Chapter[] = [];
  let currentTitle: string | null = null;
  let currentText = "";

  for (const line of lines) {
    const headingMatch = line.match(/^#{1,2}\s+(.+)$/);
    if (headingMatch) {
      if (currentText.trim() || currentTitle !== null) {
        chapters.push({ title: currentTitle, text: currentText.trim() });
        currentText = "";
      }
      currentTitle = headingMatch[1].trim();
    } else {
      currentText += line + "\n";
    }
  }

  if (currentText.trim() || currentTitle !== null) {
    chapters.push({ title: currentTitle, text: currentText.trim() });
  }

  if (chapters.length === 0) {
    chapters.push({ title: null, text: content });
  }

  return chapters;
}

export interface Chunk {
  chapterIdx: number;
  chapterTitle: string | null;
  chunkIndex: number;
  text: string;
}

export function chunkText(chapters: Chapter[]): Chunk[] {
  const chunkSize = 2000;
  const overlap = 400;
  const chunks: Chunk[] = [];

  for (let chapterIdx = 0; chapterIdx < chapters.length; chapterIdx++) {
    const chapter = chapters[chapterIdx];
    const text = chapter.text;
    if (!text) continue;

    const chars = [...text];
    let start = 0;
    let chunkIndex = 0;

    while (start < chars.length) {
      const end = Math.min(start + chunkSize, chars.length);
      const chunkText = chars.slice(start, end).join("");

      chunks.push({
        chapterIdx,
        chapterTitle: chapter.title,
        chunkIndex,
        text: chunkText,
      });

      chunkIndex++;
      if (end >= chars.length) break;
      start += chunkSize - overlap;
    }
  }

  return chunks;
}

export function markdownToHtml(md: string): string {
  return marked.parse(md, { async: false }) as string;
}
