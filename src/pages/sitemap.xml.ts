import type { APIRoute } from "astro";
import { listAllBookReferences } from "../lib/db";

export const GET: APIRoute = async () => {
  const books = await listAllBookReferences();

  let xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://bilbo.example.com/</loc>
    <priority>1.0</priority>
  </url>
  <url>
    <loc>https://bilbo.example.com/discussion</loc>
    <priority>0.5</priority>
  </url>
`;

  for (const { reference } of books) {
    xml += `  <url>
    <loc>https://bilbo.example.com/book/${reference}</loc>
    <priority>0.8</priority>
  </url>
`;
  }

  xml += "</urlset>";

  return new Response(xml, {
    headers: { "Content-Type": "application/xml" },
  });
};
