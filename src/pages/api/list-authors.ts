import type { APIRoute } from "astro";
import { listAllAuthors } from "../../lib/db";

export const prerender = false;

export const GET: APIRoute = async () => {
  try {
    const authors = await listAllAuthors();
    return new Response(JSON.stringify(authors), {
      headers: { "Content-Type": "application/json" },
    });
  } catch (e) {
    console.error("list-authors error:", e);
    return new Response(JSON.stringify({ error: "Internal server error" }), {
      status: 500,
      headers: { "Content-Type": "application/json" },
    });
  }
};
