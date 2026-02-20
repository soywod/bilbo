import type { APIRoute } from "astro";
import { embedTexts, ragChat } from "../../lib/mistral";
import { searchSimilar } from "../../lib/qdrant";
import { markdownToHtml } from "../../lib/markdown";
import type { ChatMessage, ChatSource } from "../../lib/types";

export const prerender = false;

export const POST: APIRoute = async ({ request }) => {
  try {
    const { messages }: { messages: ChatMessage[] } = await request.json();

    const mistralKey = process.env.MISTRAL_API_KEY ?? "";
    if (!mistralKey) {
      return new Response(
        JSON.stringify({ error: "Mistral API key not configured" }),
        { status: 500, headers: { "Content-Type": "application/json" } },
      );
    }

    const lastUserMsg = [...messages].reverse().find((m) => m.role === "user");
    if (!lastUserMsg) {
      return new Response(JSON.stringify({ error: "No user message" }), {
        status: 400,
        headers: { "Content-Type": "application/json" },
      });
    }

    const embeddings = await embedTexts([lastUserMsg.content]);
    if (embeddings.length === 0) {
      return new Response(JSON.stringify({ error: "No embedding returned" }), {
        status: 500,
        headers: { "Content-Type": "application/json" },
      });
    }

    const results = await searchSimilar(embeddings[0], [], null, 5);

    const context = results
      .map(
        (r, i) =>
          `[Source ${i + 1}: ${r.title} - ${r.reference}]\n${r.chunk_text}\n`,
      )
      .join("");

    const seen = new Set<string>();
    const sources: ChatSource[] = [];
    for (const r of results) {
      if (seen.has(r.reference)) continue;
      seen.add(r.reference);
      sources.push({
        reference: r.reference,
        title: r.title,
        chunk_text: r.chunk_text.slice(0, 200),
      });
    }

    const chatMessages = messages.map((m) => ({
      role: m.role,
      content: m.content,
    }));

    const response = await ragChat(context, chatMessages);
    const htmlResponse = markdownToHtml(response);

    const reply: ChatMessage = {
      role: "assistant",
      content: htmlResponse,
      sources,
    };

    return new Response(JSON.stringify(reply), {
      headers: { "Content-Type": "application/json" },
    });
  } catch (e) {
    console.error("chat error:", e);
    return new Response(JSON.stringify({ error: "Internal server error" }), {
      status: 500,
      headers: { "Content-Type": "application/json" },
    });
  }
};
