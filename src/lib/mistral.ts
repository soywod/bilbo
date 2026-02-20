const MISTRAL_EMBED_URL = "https://api.mistral.ai/v1/embeddings";
const MISTRAL_CHAT_URL = "https://api.mistral.ai/v1/chat/completions";
const EMBED_MODEL = "mistral-embed";
const CHAT_MODEL = "mistral-small-latest";

function getApiKey(): string {
  return process.env.MISTRAL_API_KEY ?? "";
}

export async function embedTexts(texts: string[]): Promise<number[][]> {
  if (texts.length === 0) return [];

  const apiKey = getApiKey();
  const allEmbeddings: number[][] = [];

  for (let i = 0; i < texts.length; i += 16) {
    const batch = texts.slice(i, i + 16);

    const resp = await fetch(MISTRAL_EMBED_URL, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${apiKey}`,
      },
      body: JSON.stringify({ model: EMBED_MODEL, input: batch }),
    });

    if (!resp.ok) {
      const body = await resp.text();
      throw new Error(`embed API error ${resp.status}: ${body}`);
    }

    const result: { data: { embedding: number[] }[] } = await resp.json();
    for (const d of result.data) {
      allEmbeddings.push(d.embedding);
    }
  }

  return allEmbeddings;
}

async function chatCompletion(
  messages: { role: string; content: string }[],
): Promise<string> {
  const apiKey = getApiKey();

  const resp = await fetch(MISTRAL_CHAT_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${apiKey}`,
    },
    body: JSON.stringify({ model: CHAT_MODEL, messages }),
  });

  if (!resp.ok) {
    const body = await resp.text();
    throw new Error(`chat API error ${resp.status}: ${body}`);
  }

  const result: { choices: { message: { content: string } }[] } =
    await resp.json();
  const choice = result.choices[0];
  if (!choice) throw new Error("no chat response");
  return choice.message.content;
}

export async function generateSummary(text: string): Promise<string> {
  const system =
    "Tu es un assistant qui rédige des résumés factuels de livres. " +
    "Tes résumés doivent être objectifs et concis. " +
    "Ne commence jamais par des phrases comme « Voici un résumé », « Ce texte parle de », etc. " +
    "Commence directement par le contenu du résumé. " +
    "Maximum 5 phrases.";

  const prompt = `Résume le texte suivant en français en 5 phrases maximum :\n\n${text}`;

  return chatCompletion([
    { role: "system", content: system },
    { role: "user", content: prompt },
  ]);
}

export async function generateChapterSummaries(
  chapters: { title: string | null; text: string }[],
): Promise<string[]> {
  const system =
    "Tu es un assistant qui rédige des résumés factuels de chapitres de livres. " +
    "Tes résumés doivent être objectifs et concis. " +
    "Ne commence jamais par des phrases comme « Voici un résumé », « Ce chapitre parle de », etc. " +
    "Commence directement par le contenu du résumé. " +
    "Maximum 3 phrases.";

  const summaries: string[] = [];

  for (const ch of chapters) {
    if (!ch.text.trim()) {
      summaries.push("");
      continue;
    }

    const chapterLabel = ch.title ? `le chapitre "${ch.title}"` : "ce chapitre";

    const prompt = `Résume ${chapterLabel} en 3 phrases maximum en français :\n\n${ch.text.slice(0, 4000)}`;

    const summary = await chatCompletion([
      { role: "system", content: system },
      { role: "user", content: prompt },
    ]);
    summaries.push(summary);
  }

  return summaries;
}

export async function ragDiscussion(
  context: string,
  messages: { role: string; content: string }[],
): Promise<string> {
  const systemPrompt =
    "Tu es un assistant bibliothécaire. Tu dois répondre UNIQUEMENT à partir des extraits de livres fournis ci-dessous. " +
    "N'utilise JAMAIS tes connaissances générales. Si la réponse ne se trouve pas dans les extraits, dis simplement que tu ne disposes pas de cette information dans la bibliothèque. " +
    "Cite les titres des livres quand c'est pertinent.\n\n" +
    `Extraits :\n${context}`;

  return chatCompletion([
    { role: "system", content: systemPrompt },
    ...messages,
  ]);
}
