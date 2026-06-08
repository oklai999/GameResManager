import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import packageJson from "./package.json";

const appVersion = packageJson.version.replace(/[^a-zA-Z0-9_-]/g, "_");

export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: {
      output: {
        entryFileNames: `assets/[name]-v${appVersion}-[hash].js`,
        chunkFileNames: `assets/[name]-v${appVersion}-[hash].js`,
        assetFileNames: `assets/[name]-v${appVersion}-[hash][extname]`,
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["src/test/setup.ts"],
  },
});
