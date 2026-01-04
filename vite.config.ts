import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "path";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  resolve: {
    alias: {
      $lib: path.resolve("./src/lib"),
    },
  },
  server: {
    port: 8080,
    strictPort: true,
  },
});
