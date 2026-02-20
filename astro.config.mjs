import { defineConfig } from "astro/config";
import preact from "@astrojs/preact";
import vercel from "@astrojs/vercel";

export default defineConfig({
  output: "static",
  adapter: vercel(),
  integrations: [preact()],
  vite: {
    css: {
      preprocessorOptions: {
        scss: {
          api: "modern-compiler",
        },
      },
    },
  },
});
