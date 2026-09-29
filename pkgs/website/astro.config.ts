import cloudflare from "@astrojs/cloudflare";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "astro/config";

export default defineConfig({
  site: "https://yatera.fyi",

  vite: {
    plugins: [tailwindcss()],
  },

  adapter: cloudflare(),
});
