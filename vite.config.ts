import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    host: host ?? "127.0.0.1",
    port: 1420,
    strictPort: true,
    ...(host === undefined
      ? {}
      : {
          hmr: {
            protocol: "ws" as const,
            host,
            port: 1421,
          },
        }),
    watch: {
      ignored: ["**/src-tauri/**", "**/crates/**"],
    },
  },
});
