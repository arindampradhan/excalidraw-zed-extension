import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import fs from "fs";
import path from "path";
import { execSync } from "child_process";
import type { IncomingMessage, ServerResponse } from "http";

// Point DEV_FILE env var at any .excalidraw file you want to use while developing.
// Falls back to preview-binary/test.excalidraw, which is auto-created if missing.
const TEST_FILE = process.env.DEV_FILE
  ? path.resolve(process.env.DEV_FILE)
  : path.resolve(__dirname, "../test.excalidraw");

// Detect the system color scheme on the host so the mock returns the same
// theme the real Rust server would return, avoiding a light/dark flash.
function detectSystemTheme(): "dark" | "light" {
  // Explicit override via env var takes priority.
  const env = process.env.THEME;
  if (env === "dark" || env === "light") return env;
  try {
    // GNOME / GTK (covers most Linux desktops)
    const scheme = execSync(
      "gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null",
      { encoding: "utf-8", stdio: ["ignore", "pipe", "ignore"] },
    );
    if (scheme.includes("dark")) return "dark";
  } catch { /* not GNOME or gsettings unavailable */ }
  try {
    // macOS
    const style = execSync("defaults read -g AppleInterfaceStyle 2>/dev/null", {
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
    if (style === "Dark") return "dark";
  } catch { /* not macOS or not in dark mode */ }
  return "light";
}

const systemTheme = detectSystemTheme();

// Ensure the test file exists so the dev server doesn't crash on first run.
if (!fs.existsSync(TEST_FILE)) {
  fs.writeFileSync(
    TEST_FILE,
    JSON.stringify({
      type: "excalidraw",
      version: 2,
      source: "excalidraw-zed-preview",
      elements: [],
      appState: { gridSize: null, viewBackgroundColor: "#ffffff" },
      files: {},
    }),
  );
  console.info(`[mock-api] Created empty test file at ${TEST_FILE}`);
}

// SSE: keep a list of active response streams and broadcast to them.
const sseClients = new Set<ServerResponse>();

function broadcastReload() {
  for (const res of sseClients) {
    try {
      res.write("data: reload\n\n");
    } catch {
      sseClients.delete(res);
    }
  }
}

// Watch the test file for external changes (e.g. you save it in Zed) and
// broadcast an SSE reload event so the webview's EventSource picks it up.
let fsWatchDebounce: ReturnType<typeof setTimeout> | null = null;
fs.watch(TEST_FILE, () => {
  if (fsWatchDebounce) clearTimeout(fsWatchDebounce);
  fsWatchDebounce = setTimeout(broadcastReload, 80);
});

export default defineConfig({
  plugins: [
    {
      name: "mock-api",
      configureServer(server) {
        // GET /config
        server.middlewares.use(
          "/config",
          (_req: IncomingMessage, res: ServerResponse) => {
            res.setHeader("Content-Type", "application/json");
            res.end(
              JSON.stringify({
                contentType: "application/json",
                name: path.basename(TEST_FILE),
                theme: systemTheme,
              }),
            );
          },
        );

        // GET + POST /data
        server.middlewares.use(
          "/data",
          (req: IncomingMessage, res: ServerResponse) => {
            if (req.method === "POST") {
              // Write-back from App.tsx's onChange → save to TEST_FILE.
              const chunks: Buffer[] = [];
              req.on("data", (chunk: Buffer) => chunks.push(chunk));
              req.on("end", () => {
                const body = Buffer.concat(chunks);
                fs.writeFile(TEST_FILE, body, (err) => {
                  if (err) {
                    res.writeHead(500);
                    res.end("write failed");
                  } else {
                    // Don't broadcast SSE here — App.tsx suppresses the echo itself.
                    res.writeHead(200);
                    res.end("ok");
                  }
                });
              });
              return;
            }
            // GET
            try {
              const content = fs.readFileSync(TEST_FILE);
              res.setHeader("Content-Type", "application/json");
              res.end(content);
            } catch (e) {
              res.writeHead(500);
              res.end(`read failed: ${e}`);
            }
          },
        );

        // GET /events — real SSE stream that fires when the file changes.
        server.middlewares.use(
          "/events",
          (_req: IncomingMessage, res: ServerResponse) => {
            res.setHeader("Content-Type", "text/event-stream");
            res.setHeader("Cache-Control", "no-cache");
            res.setHeader("Connection", "keep-alive");
            res.flushHeaders();
            // Keep-alive comment so the browser doesn't time out.
            res.write(":\n\n");
            sseClients.add(res);
            res.on("close", () => sseClients.delete(res));
          },
        );
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
