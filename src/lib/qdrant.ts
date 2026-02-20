import { QdrantClient } from "@qdrant/js-client-rest";

const COLLECTION_NAME = "book_chunks";
const VECTOR_SIZE = 1024;

let client: QdrantClient;

export function getQdrant(): QdrantClient {
  if (!client) {
    client = new QdrantClient({
      url: process.env.QDRANT_URL!,
      apiKey: process.env.QDRANT_API_KEY || undefined,
    });
  }
  return client;
}

export async function ensureCollection(): Promise<void> {
  const q = getQdrant();
  const exists = await q.collectionExists(COLLECTION_NAME);
  if (exists.exists) return;

  await q.createCollection(COLLECTION_NAME, {
    vectors: { size: VECTOR_SIZE, distance: "Cosine" },
    quantization_config: {
      scalar: { type: "int8", always_ram: true },
    },
  });

  await q.createPayloadIndex(COLLECTION_NAME, {
    field_name: "book_id",
    field_schema: "keyword",
  });
  await q.createPayloadIndex(COLLECTION_NAME, {
    field_name: "tags",
    field_schema: "keyword",
  });
  await q.createPayloadIndex(COLLECTION_NAME, {
    field_name: "authors",
    field_schema: "keyword",
  });
}

export async function deleteBookPoints(bookId: string): Promise<void> {
  const q = getQdrant();
  await q.delete(COLLECTION_NAME, {
    filter: {
      must: [{ key: "book_id", match: { value: bookId } }],
    },
  });
}

export interface QdrantChunk {
  chapterIdx: number;
  chapterTitle: string | null;
  chunkIndex: number;
  text: string;
}

export async function upsertChunks(
  bookId: string,
  reference: string,
  title: string,
  authors: string[],
  tags: string[],
  chunks: QdrantChunk[],
  embeddings: number[][],
): Promise<void> {
  const q = getQdrant();
  const points: {
    id: string;
    vector: number[];
    payload: Record<string, unknown>;
  }[] = [];

  for (let i = 0; i < chunks.length; i++) {
    const chunk = chunks[i];
    const embedding = embeddings[i];
    points.push({
      id: crypto.randomUUID(),
      vector: embedding,
      payload: {
        book_id: bookId,
        reference,
        title,
        chunk_index: chunk.chunkIndex,
        chunk_text: chunk.text,
        chapter_idx: chunk.chapterIdx,
        chapter: chunk.chapterTitle ?? "",
        authors,
        tags,
      },
    });

    if (points.length >= 100) {
      const batch = points.splice(0, points.length);
      await q.upsert(COLLECTION_NAME, { points: batch });
    }
  }

  if (points.length > 0) {
    await q.upsert(COLLECTION_NAME, { points });
  }
}

export interface SearchResult {
  reference: string;
  title: string;
  chunk_text: string;
  score: number;
}

export async function searchSimilar(
  queryEmbedding: number[],
  tags: string[],
  author: string | null,
  limit: number,
): Promise<SearchResult[]> {
  const q = getQdrant();

  const must: Array<{
    key: string;
    match: { value: string };
  }> = [];

  for (const t of tags) {
    must.push({ key: "tags", match: { value: t } });
  }
  if (author) {
    must.push({ key: "authors", match: { value: author } });
  }

  const results = await q.search(COLLECTION_NAME, {
    vector: queryEmbedding,
    limit,
    with_payload: true,
    ...(must.length > 0 ? { filter: { must } } : {}),
  });

  return results.map((point) => ({
    reference: (point.payload?.reference as string) ?? "",
    title: (point.payload?.title as string) ?? "",
    chunk_text: (point.payload?.chunk_text as string) ?? "",
    score: point.score,
  }));
}
