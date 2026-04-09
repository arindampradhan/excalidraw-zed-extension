import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import fs from "fs";
import path from "path";

const TEST_FILE = path.resolve(__dirname, "../test.excalidraw");

export default defineConfig({
  plugins: [
    {
      name: "mock-api",
      configureServer(server) {
        server.middlewares.use("/config", (_, res) => {
          res.setHeader("Content-Type", "application/json");
          res.end(
            JSON.stringify({
              contentType: "application/json",
              name: "test.excalidraw",
              theme: "auto",
            })
          );
        });
        server.middlewares.use("/data", (_, res) => {
          const content = fs.readFileSync(TEST_FILE, "utf-8");
          res.setHeader("Content-Type", "application/json");
          res.end(content);
        });
        server.middlewares.use("/events", (_, res) => {
          res.setHeader("Content-Type", "text/event-stream");
          res.flushHeaders();
          res.write(":\n\n");
          res.on("close", () => res.destroy());
        });
      },
    },
    react(),
  ],
  base: "/assets/",
  build: {
    outDir: "../assets",
    emptyOutDir: true,
  },
});