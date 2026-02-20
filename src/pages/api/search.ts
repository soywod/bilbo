import type { APIRoute } from "astro";
import { searchBooksFts } from "../../lib/db";

export const prerender = false;

export const POST: APIRoute = async ({ request }) => {
  try {
    const {
      query = "",
      tags = [],
      author = null,
      page = 0,
      page_size = 20,
    } = await request.json();

    const { books, total } = await searchBooksFts(
      query,
      tags,
      author,
      page,
      page_size,
    );

    return new Response(JSON.stringify({ books, total }), {
      headers: { "Content-Type": "application/json" },
    });
  } catch (e) {
    console.error("search error:", e);
    return new Response(JSON.stringify({ error: "Internal server error" }), {
      status: 500,
      headers: { "Content-Type": "application/json" },
    });
  }
};
