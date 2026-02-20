import { defineMiddleware } from "astro:middleware";
import { createSupabaseServer } from "./lib/supabase";

const PROTECTED = ["/discussion", "/api/discussion"];

export const onRequest = defineMiddleware(async (context, next) => {
  const { pathname } = context.url;

  if (!PROTECTED.some((p) => pathname === p || pathname.startsWith(p + "/"))) {
    return next();
  }

  const supabase = createSupabaseServer(context.request, context.cookies);
  const {
    data: { user },
  } = await supabase.auth.getUser();

  if (!user) {
    if (pathname.startsWith("/api/")) {
      return new Response(JSON.stringify({ error: "Unauthorized" }), {
        status: 401,
        headers: { "Content-Type": "application/json" },
      });
    }
    return context.redirect("/login");
  }

  context.locals.user = user;
  return next();
});
