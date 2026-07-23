import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2021",
    outDir: "dist",
    sourcemap: true,
    // Zwei eigenständige HTML-Einstiegspunkte: die Haupt-App (index.html)
    // und das kleine Splashscreen-Fenster (splashscreen.html), das beim
    // Programmstart kurz angezeigt wird.
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        splashscreen: resolve(__dirname, "splashscreen.html"),
      },
    },
  },
});
