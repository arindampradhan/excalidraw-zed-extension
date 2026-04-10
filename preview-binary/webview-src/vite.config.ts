import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Production build: just React + the output paths.
// Dev server: adds a mock API (GET /config, GET+POST /data, GET /events)
// so you can run `vite dev` without the Rust binary.
//
// In dev, open any excalidraw file by passing a ?file= query param:
//   http://localhost:5173?file=/absolute/path/to/diagram.excalidraw
// Omit ?file= to fall back to DEV_FILE env var or preview-binary/test.excalidraw.
export default defineConfig(({ command }) => {
  const isDev = command === "serve";

  return {
    plugins: [
      isDev ? mockApiPlugin() : null,
      react(),
    ].filter(Boolean),
    base: "/assets/",
    build: {
      outDir: "../assets",
      emptyOutDir: true,
    },
  };
});

// ── Dev-only mock API ─────────────────────────────────────────────────────────

function mockApiPlugin() {
  // Lazy imports — only evaluated when the dev server starts, never during build.
  const fs = require("fs") as typeof import("fs");
  const path = require("path") as typeof import("path");
  const url = require("url") as typeof import("url");
  const { execSync } = require("child_process") as typeof import("child_process");

  // Default file when no ?file= param is given.
  const DEFAULT_FILE: string = process.env.DEV_FILE
    ? path.resolve(process.env.DEV_FILE as string)
    : path.resolve(__dirname, "../test.excalidraw");

  // Create a minimal blank diagram if the default file doesn't exist.
  if (!fs.existsSync(DEFAULT_FILE)) {
    fs.writeFileSync(
      DEFAULT_FILE,
      JSON.stringify({
        type: "excalidraw",
        version: 2,
        source: "excalidraw-zed-preview",
        elements: [],
        appState: { gridSize: null, viewBackgroundColor: "#ffffff" },
        files: {},
      }),
    );
    console.info(`[mock-api] Created empty test file at ${DEFAULT_FILE}`);
  }

  function resolveFile(reqUrl: string | undefined): string {
    const parsed = url.parse(reqUrl ?? "", true);
    const fileQ = parsed.query["file"];
    if (typeof fileQ === "string" && fileQ) {
      return path.resolve(fileQ);
    }
    return DEFAULT_FILE;
  }

  function contentTypeFor(filePath: string): string {
    const name = path.basename(filePath);
    if (name.endsWith(".excalidraw.svg")) return "image/svg+xml";
    if (name.endsWith(".excalidraw.png")) return "image/png";
    return "application/json";
  }

  function detectSystemTheme(): "dark" | "light" {
    const env = process.env.THEME;
    if (env === "dark" || env === "light") return env;
    try {
      const scheme = execSync(
        "gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null",
        { encoding: "utf-8", stdio: ["ignore", "pipe", "ignore"] },
      );
      if (scheme.includes("dark")) return "dark";
    } catch { /* not GNOME */ }
    try {
      const style = execSync("defaults read -g AppleInterfaceStyle 2>/dev/null", {
        encoding: "utf-8",
        stdio: ["ignore", "pipe", "ignore"],
      }).trim();
      if (style === "Dark") return "dark";
    } catch { /* not macOS */ }
    return "light";
  }

  const systemTheme = detectSystemTheme();

  // Per-file SSE client sets: Map<absoluteFilePath, Set<ServerResponse>>
  const sseClientMap = new Map<string, Set<import("http").ServerResponse>>();

  function getSseClients(filePath: string): Set<import("http").ServerResponse> {
    if (!sseClientMap.has(filePath)) sseClientMap.set(filePath, new Set());
    return sseClientMap.get(filePath)!;
  }

  function broadcastReload(filePath: string) {
    const clients = sseClientMap.get(filePath);
    if (!clients) return;
    for (const res of clients) {
      try { res.write("data: reload\n\n"); }
      catch { clients.delete(res); }
    }
  }

  // Watch multiple files — one fs.watch per unique file path that gets requested.
  const watched = new Set<string>();
  function ensureWatched(filePath: string) {
    if (watched.has(filePath)) return;
    watched.add(filePath);
    let debounce: ReturnType<typeof setTimeout> | null = null;
    try {
      fs.watch(filePath, () => {
        if (debounce) clearTimeout(debounce);
        debounce = setTimeout(() => broadcastReload(filePath), 80);
      });
    } catch (e) {
      console.warn(`[mock-api] Could not watch ${filePath}: ${e}`);
    }
  }

  // Start watching the default file immediately.
  ensureWatched(DEFAULT_FILE);

  return {
    name: "mock-api",
    configureServer(server: import("vite").ViteDevServer) {
      server.middlewares.use("/config", (req: import("http").IncomingMessage, res: import("http").ServerResponse) => {
        const filePath = resolveFile(req.url);
        res.setHeader("Content-Type", "application/json");
        res.end(JSON.stringify({
          contentType: contentTypeFor(filePath),
          name: path.basename(filePath),
          theme: systemTheme,
          autoSave: false,
        }));
      });

      server.middlewares.use("/data", (req: import("http").IncomingMessage, res: import("http").ServerResponse) => {
        const filePath = resolveFile(req.url);
        if (req.method === "POST") {
          const chunks: Buffer[] = [];
          req.on("data", (chunk: Buffer) => chunks.push(chunk));
          req.on("end", () => {
            fs.writeFile(filePath, Buffer.concat(chunks), (err) => {
              res.writeHead(err ? 500 : 200);
              res.end(err ? "write failed" : "ok");
            });
          });
          return;
        }
        // GET
        if (!fs.existsSync(filePath)) {
          res.writeHead(404);
          res.end(`File not found: ${filePath}`);
          return;
        }
        ensureWatched(filePath);
        try {
          res.setHeader("Content-Type", contentTypeFor(filePath));
          res.end(fs.readFileSync(filePath));
        } catch (e) {
          res.writeHead(500);
          res.end(`read failed: ${e}`);
        }
      });

      server.middlewares.use("/events", (req: import("http").IncomingMessage, res: import("http").ServerResponse) => {
        const filePath = resolveFile(req.url);
        ensureWatched(filePath);
        res.setHeader("Content-Type", "text/event-stream");
        res.setHeader("Cache-Control", "no-cache");
        res.setHeader("Connection", "keep-alive");
        res.flushHeaders();
        res.write(":\n\n");
        const clients = getSseClients(filePath);
        clients.add(res);
        res.on("close", () => clients.delete(res));
      });
    },
  };
}
