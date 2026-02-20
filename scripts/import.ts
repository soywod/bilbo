import fs from "fs/promises";
import path from "path";
import {
  findBookByReference,
  insertBook,
  updateBook,
  upsertChapterSummaries,
} from "../src/lib/db";
import { parseMarkdown, extractChapters, chunkText } from "../src/lib/markdown";
import {
  embedTexts,
  generateSummary,
  generateChapterSummaries,
} from "../src/lib/mistral";
import {
  ensureCollection,
  deleteBookPoints,
  upsertChunks,
} from "../src/lib/qdrant";

const DATA_DIR = path.resolve("data");
const PROCESSED_DIR = path.join(DATA_DIR, "processed");
const FAILED_DIR = path.join(DATA_DIR, "failed");

async function main() {
  await fs.mkdir(PROCESSED_DIR, { recursive: true });
  await fs.mkdir(FAILED_DIR, { recursive: true });

  await ensureCollection();

  const entries = await fs.readdir(DATA_DIR);
  const mdFiles = entries.filter((f) => f.endsWith(".md"));

  for (const filename of mdFiles) {
    const filePath = path.join(DATA_DIR, filename);
    console.log(`processing ${filename}`);

    try {
      await importFile(filePath);
      const dest = path.join(PROCESSED_DIR, filename);
      await fs.rename(filePath, dest);
      console.log(`${filename} -> processed/`);
    } catch (e) {
      console.error(`failed to import ${filename}:`, e);
      const dest = path.join(FAILED_DIR, filename);
      await fs.rename(filePath, dest);
    }
  }
}

async function importFile(filePath: string) {
  const raw = await fs.readFile(filePath, "utf-8");
  const parsed = parseMarkdown(raw);
  const fm = parsed.frontmatter;

  // Check if book already exists
  const existing = await findBookByReference(fm.reference);

  if (existing && existing.hash === parsed.hash) {
    console.log(`book ${fm.reference} unchanged, skipping`);
    return;
  }

  // Generate summary via Mistral if not in frontmatter
  const mistralKey = process.env.MISTRAL_API_KEY ?? "";
  let summary = fm.summary;

  if (!summary && mistralKey) {
    try {
      summary = await generateSummary(parsed.content.slice(0, 6000));
    } catch (e) {
      console.error("summary generation failed:", e);
    }
  }

  // Upsert book in PostgreSQL
  let bookId: string;
  if (existing) {
    bookId = await updateBook(
      fm.reference,
      parsed.hash,
      fm.title,
      fm.authors,
      fm.editor,
      fm.tags,
      fm.edition_date,
      summary ?? null,
      fm.introduction,
      fm.cover_text,
      fm.ean,
      fm.isbn,
      parsed.content,
      fm.reseller_paper_urls,
      fm.reseller_digital_urls,
    );
  } else {
    bookId = await insertBook(
      fm.reference,
      parsed.hash,
      fm.title,
      fm.authors,
      fm.editor,
      fm.tags,
      fm.edition_date,
      summary ?? null,
      fm.introduction,
      fm.cover_text,
      fm.ean,
      fm.isbn,
      parsed.content,
      fm.reseller_paper_urls,
      fm.reseller_digital_urls,
    );
  }

  // Extract chapters and generate chapter summaries
  const chapters = extractChapters(parsed.content);

  if (mistralKey) {
    const chapterInputs = chapters.map((c) => ({
      title: c.title,
      text: c.text,
    }));

    const chapterSummaries = await generateChapterSummaries(
      chapterInputs,
    ).catch(() => [] as string[]);

    const summariesData = chapterSummaries
      .map((s, i) => ({
        chapterIdx: i,
        title: chapters[i]?.title ?? null,
        summary: s,
      }))
      .filter((s) => s.summary);

    await upsertChapterSummaries(bookId, summariesData);
  }

  // Chunk content and generate embeddings
  const chunks = chunkText(chapters);

  if (mistralKey && chunks.length > 0) {
    const texts = chunks.map((c) => c.text);
    const embeddings = await embedTexts(texts);

    // Delete old points if updating
    if (existing) {
      await deleteBookPoints(bookId);
    }

    await upsertChunks(
      bookId,
      fm.reference,
      fm.title,
      fm.authors,
      fm.tags,
      chunks,
      embeddings,
    );
  }

  console.log(`imported book ${fm.reference} (${fm.title})`);
}

main().catch((e) => {
  console.error("import failed:", e);
  process.exit(1);
});
