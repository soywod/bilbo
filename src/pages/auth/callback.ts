import type { APIRoute } from "astro";
import { createSupabaseServer } from "../../lib/supabase";

export const prerender = false;

export const GET: APIRoute = async ({ url, request, cookies, redirect }) => {
  const code = url.searchParams.get("code");

  if (code) {
    const supabase = createSupabaseServer(request, cookies);
    await supabase.auth.exchangeCodeForSession(code);
  }

  return redirect("/discussion");
};
