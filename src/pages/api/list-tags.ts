import type { APIRoute } from "astro";
import { listAllTags } from "../../lib/db";

export const prerender = false;

export const GET: APIRoute = async () => {
  try {
    const tags = await listAllTags();
    return new Response(JSON.stringify(tags), {
      headers: { "Content-Type": "application/json" },
    });
  } catch (e) {
    console.error("list-tags error:", e);
    return new Response(JSON.stringify({ error: "Internal server error" }), {
      status: 500,
      headers: { "Content-Type": "application/json" },
    });
  }
};
