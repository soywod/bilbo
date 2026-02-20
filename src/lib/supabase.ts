import { createBrowserClient, createServerClient } from "@supabase/ssr";
import type { AstroCookies } from "astro";

export function createSupabaseBrowser() {
  return createBrowserClient(
    import.meta.env.PUBLIC_SUPABASE_URL,
    import.meta.env.PUBLIC_SUPABASE_ANON_KEY,
  );
}

export function createSupabaseServer(request: Request, cookies: AstroCookies) {
  return createServerClient(
    import.meta.env.PUBLIC_SUPABASE_URL,
    import.meta.env.PUBLIC_SUPABASE_ANON_KEY,
    {
      cookies: {
        getAll() {
          const header = request.headers.get("cookie") ?? "";
          return header
            .split(";")
            .map((pair) => {
              const [name, ...rest] = pair.trim().split("=");
              return { name: name ?? "", value: rest.join("=") };
            })
            .filter((c) => c.name);
        },
        setAll(cookiesToSet) {
          for (const { name, value, options } of cookiesToSet) {
            cookies.set(name, value, options);
          }
        },
      },
    },
  );
}
