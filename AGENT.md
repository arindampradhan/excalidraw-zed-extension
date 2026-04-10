# AGENT.md — Excalidraw Preview for Zed

> This file is the source of truth for AI agents working on this repo.
> `CLAUDE.md` is a symlink to this file.

## Project Purpose

A Zed editor extension that previews `.excalidraw` files in a native WebView window (powered by `wry`). The preview live-reloads on file save. No browser tabs. No in-editor UI panes. Pure offline, near-zero-latency diagram preview.

Supports all three Excalidraw file formats: `.excalidraw` (JSON), `.excalidraw.svg`, `.excalidraw.png`.

See [`docs/PRD.md`](docs/PRD.md) for the full product requirements.
See [`refs/excalidraw-vscode/`](refs/excalidraw-vscode/) (git submodule) for the reference implementation this is adapted from.

---

## Repository Layout

```
excalidraw-zed-extension/
│
├── AGENT.md                        ← you are here (source of truth)
├── CLAUDE.md                       ← symlink → AGENT.md
├── docs/
│   └── PRD.md                      ← full product requirements doc
│
├── extension/                      ← Zed extension (Rust → WASM)
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs                  ← slash command, spawn binary, focus ping
│   └── extension.toml              ← Zed extension manifest
│
├── preview-binary/                 ← companion native binary
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs                 ← CLI entry, all routes, file watcher, WebView (monolith)
│   ├── webview-src/                ← React + Vite source (npm project)
│   │   ├── package.json            ← @excalidraw/excalidraw ^0.18, react ^18, vite
│   │   ├── vite.config.ts          ← prod: assets only; dev: mock API plugin for any file
│   │   ├── index.html
│   │   └── src/
│   │       ├── main.tsx            ← fetch /config + /data → loadFromBlob → render, SSE
│   │       └── App.tsx             ← <Excalidraw> editor, Ctrl+S / auto-save, SSE reload
│   └── assets/                     ← Vite build output; committed; embedded at compile time
│       ├── index.html              ← served at GET /
│       └── assets/
│           ├── index-[hash].js     ← React + Excalidraw bundle
│           ├── index-[hash].css
│           └── *.woff2, *.wasm     ← Excalidraw runtime assets (GET /assets/*)
│
├── refs/
│   └── excalidraw-vscode/          ← git submodule: VS Code reference implementation
│
├── .claude/
│   └── skills/
│       └── zed-extension/
│           └── SKILL.md            ← Zed extension scaffolding skill
│
└── Cargo.toml                      ← workspace root
```

---

## Workspace Cargo.toml (root)

```toml
[workspace]
members = [
    "extension",
    "preview-binary",
]
resolver = "2"
```

---

## Component 1 — Zed Extension (`extension/`)

**Target:** `wasm32-wasip1`
**Crate type:** `cdylib`

### extension.toml

```toml
id = "excalidraw-preview"
name = "Excalidraw Preview"
version = "0.1.0"
schema_version = 1
authors = ["you"]
description = "Preview .excalidraw files in a native window"
repository = "https://github.com/you/excalidraw-zed-extension"

[slash_commands.preview-excalidraw]
description = "Open live preview for the active .excalidraw file"
requires_argument = false
```

### extension/src/lib.rs — responsibilities

1. Implement `zed_extension_api::Extension` trait.
2. Register `/preview-excalidraw` slash command.
3. On command run:
   - Get `worktree` + active file path via extension API.
   - Validate extension is `.excalidraw`, `.excalidraw.svg`, or `.excalidraw.png`.
   - Resolve path to companion binary (`excalidraw-preview`).
   - Spawn via `zed_extension_api::process::Command::new(binary).arg(file_path).spawn()`.
4. Track `HashMap<PathBuf, (u32, u16)>` — PID + port per file (in-memory, single process lifetime).
5. On re-invoke for same file: `GET http://127.0.0.1:{port}/focus` (HTTP ping via `zed::http_client_get`).

**Key constraint:** WASM extensions cannot open sockets or use `std::process`.
Use `zed_extension_api::process::Command` and `zed::http_client_get` only.

---

## Component 2 — Companion Binary (`preview-binary/`)

**Target:** native (x86_64/aarch64, macOS/Linux/Windows)

**Note:** all server, watcher, webview, and asset logic lives in a single `main.rs` (monolith); the AGENT.md originally described separate files that were never split out.

### CLI

```
excalidraw-preview <file-path> [--port <port>] [--auto-save] [--debug]
excalidraw-preview --lsp
excalidraw-preview --dev
excalidraw-preview --dev-server <url>
```

### Startup sequence

1. Parse args with `clap`.
2. If `--lsp`: run JSON-RPC LSP server loop (Zed language server integration).
3. If `--dev` / `--dev-server`: open WebView at the Vite dev server URL directly.
4. Otherwise: detect file format from extension → MIME type string.
5. Check lock file `$TMPDIR/excalidraw-{sha256(canonical_path)}.lock`.
   - If live (`GET /ping` succeeds) → send `GET /focus` and exit.
   - If stale → remove and continue.
6. Bind axum server on ephemeral port (or `--port`).
7. Write port to lock file.
8. Spawn file watcher thread (notify v6, 80 ms debounce → broadcast channel).
9. Open WebView window at `http://127.0.0.1:{port}`.
10. On window close: remove lock file, shut down server.

### HTTP Routes

| Route | Description |
|---|---|
| `GET /` | Serve embedded `index.html` |
| `GET /config` | JSON: `{ contentType, name, theme, autoSave }` |
| `GET /data` | Read file from disk, return bytes with correct `Content-Type` |
| `POST /data` | Write request body back to disk (save from WebView) |
| `GET /events` | SSE stream; emit `data: reload` on file change |
| `GET /focus` | Signal WebView window to call `window.set_focus()` |
| `GET /ping` | 200 OK liveness probe |
| `GET /shutdown` | Graceful shutdown (called on `textDocument/didClose`) |
| `GET /assets/*` | Serve embedded assets (rust-embed, MIME via mime_guess) |

### AppState fields

```rust
struct AppState {
    file_path: PathBuf,
    lock_path: PathBuf,
    content_type: String,   // MIME string
    file_name: String,
    auto_save: bool,        // forwarded to /config → frontend
    broadcast_tx: broadcast::Sender<()>,
    focus_tx: Arc<watch::Sender<bool>>,
}
```

### ConfigResponse (camelCase via serde)

```json
{ "contentType": "application/json", "name": "diagram", "theme": "auto", "autoSave": false }
```

### LSP server

Implements a minimal JSON-RPC LSP so Zed can invoke the binary as a language server for `.excalidraw` files:
- `textDocument/didOpen` → spawns `excalidraw-preview <path>` as a detached process
- `textDocument/didClose` → sends `GET /shutdown` to the running instance
- `initialize` / `shutdown` / `exit` handled normally

---

## Component 3 — Web UI (`preview-binary/webview-src/`)

### vite.config.ts

Uses Vite's function-form config to split prod vs dev cleanly:

- **Production build** (`vite build`): only the React plugin runs. No file watchers, no mock server, no `execSync`. Build exits immediately.
- **Dev server** (`vite dev`): a `mockApiPlugin()` is activated that implements the full API (`/config`, `/data`, `/events`) using the local filesystem, mirroring the Rust server.

### Dev server — any file via `?file=` param

In dev mode, open any excalidraw file without restarting the server:

```
http://localhost:5173?file=/absolute/path/to/diagram.excalidraw
http://localhost:5173?file=/absolute/path/to/other.excalidraw.svg
```

Multiple tabs work independently — each `?file=` gets its own SSE client set and `fs.watch` handle. Defaults to `DEV_FILE` env var or `preview-binary/test.excalidraw` if no param is given.

### main.tsx — startup sequence

```ts
// ?file= query param forwarded to all API calls in dev; ignored (absent) in prod.
const fileParam = new URLSearchParams(window.location.search).get("file");
function apiUrl(path) { return fileParam ? `${path}?file=${encodeURIComponent(fileParam)}` : path; }

const config = await fetch(apiUrl('/config')).then(r => r.json());
// config: { contentType, name, theme, autoSave }

// Resolve "auto" theme once before React mounts (avoids matchMedia issues in WebKitGTK).
if (config.theme === "auto") config.theme = window.matchMedia(...).matches ? "dark" : "light";

const bytes = await fetch(apiUrl('/data')).then(r => r.arrayBuffer());

// Format fallback chain: try declared type first, then the other two.
for (const type of reorderFallbacks(config.contentType)) { ... }

// SSE live reload — calls reloadScene() provided by App.
const es = new EventSource(apiUrl('/events'));
es.onmessage = debounce(async () => {
  const newData = await loadFromBlob(...);
  reloadScene?.(newData);   // skips if editingElement is active; never resets viewport/theme
}, 150);
```

### App.tsx — save modes

**Manual save (default):** Ctrl+S / Cmd+S or "Save to file" menu item → `POST /data`.

**Auto-save:** when `autoSave` prop is `true` (set from `config.autoSave`), `onChange` is wired to a debounced save (600 ms). Only fires when element hash changes (not on viewport/selection events).

**SSE reload (`reloadScene`):**
- Passed to `main.tsx` via `onReloadReady` callback.
- Skips update if `api.getAppState().editingElement` is non-null (user is typing).
- Calls `api.updateScene({ elements, files })` only — never passes `appState`, so viewport position and theme are never reset.

### window.EXCALIDRAW_ASSET_PATH

Set in `index.html` before the module script loads:
```html
<script>
  window.EXCALIDRAW_ASSET_PATH = "/assets/";
  window.EXCALIDRAW_EXPORT_SOURCE = "excalidraw-zed-preview";
</script>
```

---

## Key Dependencies

### extension/Cargo.toml

```toml
[dependencies]
zed_extension_api = "0.1"
```

### preview-binary/Cargo.toml

```toml
[dependencies]
wry        = "0.43"
tao        = "0.30"
axum       = { version = "0.8", features = ["tokio"] }
tokio      = { version = "1", features = ["full"] }
notify     = "6"
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
clap       = { version = "4", features = ["derive"] }
anyhow     = "1"
sha2       = "0.10"          # for lock file path hashing
rust-embed = "8"             # for embedding assets/ directory
tracing    = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### preview-binary/webview-src/package.json

```json
{
  "dependencies": {
    "@excalidraw/excalidraw": "^0.18.0",
    "react": "^18",
    "react-dom": "^18"
  },
  "devDependencies": {
    "@vitejs/plugin-react": "latest",
    "@types/react": "^18",
    "@types/react-dom": "^18",
    "typescript": "^5",
    "vite": "^6"
  },
  "scripts": {
    "build": "vite build",
    "dev": "vite"
  }
}
```

---

## Coding Conventions

- Use `anyhow::Result` for error propagation in the binary; no `unwrap` in prod paths.
- `tracing` for structured logs; gated behind `--debug` flag in release builds.
- No `unsafe` unless required by `wry`/`tao` platform calls.
- Format with `rustfmt` defaults; lint with `clippy -- -D warnings`.
- All public items must have doc comments.
- TypeScript strict mode in webview; no `any` types.

---

## Build & Run

```bash
# 0. One-time: install WASM target + symlink binary to PATH
rustup target add wasm32-wasip1
make symlink   # ~/.local/bin/excalidraw-preview → target/release (run once)

# 1. Normal build (UI + release binary)
make

# 2. Full release (UI + binary + extension WASM)
make release

# 3. Run binary directly for testing
./target/release/excalidraw-preview ./path/to/file.excalidraw --debug

# 4. Install extension into Zed (dev mode)
# In Zed: open the command palette → "zed: install dev extension" → select the ./extension directory
```

### Makefile targets

| Target | Description |
|---|---|
| `make` | Build UI + release binary |
| `make build` | Release binary only (no UI rebuild) |
| `make build-debug` | Debug binary |
| `make ui` | Vite build only (`webview-src/` → `assets/`) |
| `make release` | UI + binary + extension WASM |
| `make symlink` | One-time: symlink `~/.local/bin/excalidraw-preview` → `target/release` |
| `make dev` | Debug build + Vite dev server + WebView window in parallel |
| `make dev-ui` | Vite dev server only |
| `make dev-window` | WebView pointed at Vite dev server |
| `make clean` | `cargo clean` (keeps `assets/`) |

### Dev workflow

```bash
# Start dev server + WebView (default file or DEV_FILE env var)
make dev DEV_FILE=docs/examples/system-architecture.excalidraw

# Or open any file in the browser without restarting:
# http://localhost:5173?file=/absolute/path/to/diagram.excalidraw
```

Vite HMR updates the WebView on every `App.tsx` save — no Rust rebuild needed during UI development.
After UI changes are done: `make ui && make build` to bake them into the release binary.

---

## Testing Strategy

- **Unit**: debounce logic in `watcher.rs`; route handlers in `server.rs` via `axum::test`.
- **Integration**: spawn binary against a temp `.excalidraw` file; assert `GET /data` returns valid JSON; assert `GET /config` returns correct `contentType`; mutate file; assert SSE fires within 150 ms.
- **Format tests**: test all three content types + fallback chain in `main.tsx` with vitest.
- **Manual**: run against real Zed + `diagram.excalidraw`; measure reload latency with WebView DevTools.

---

## Milestones

| Phase | Deliverable | Done? |
| ----- | ---------------------------------------- | ----- |
| M1    | Rust binary opens wry window + serves static index.html | ✓ |
| M2    | `webview-src/` scaffolded; Vite builds; `<Excalidraw>` renders from `/data` | ✓ |
| M3    | File watcher + SSE + `updateScene` live reload | ✓ |
| M4    | Zed extension spawns binary; slash command works end-to-end | ✓ |
| M5    | Process reuse: lock file + `/focus` + `/ping` | ✓ |
| M6    | All three file formats + fallback chain | ✓ |
| M7    | Cross-platform CI + prebuilt binary download | [ ] |

---

## Constraints & Gotchas

- **WASM sandbox**: extension runs in `wasm32-wasip1`; use `zed_extension_api::process::Command` (not `std::process`) and `zed::http_client_get` (not raw sockets).
- **`window.EXCALIDRAW_ASSET_PATH`**: must be set in a `<script>` block *before* the module script loads, or Excalidraw will fail to fetch fonts/wasm.
- **assets/ directory**: Vite outputs `assets/assets/main-[hash].js` (nested). The outer `assets/` is the root served at `/`; the inner `assets/` is the JS/CSS/font dir served at `/assets/`. Ensure `assets.rs` handles both `index.html` (at root) and everything under `assets/`.
- **Linux WebKitGTK**: require `libwebkit2gtk-4.1-dev` or `libwebkit2gtk-4.0-dev`. Print a clear error if the library is missing at startup.
- **macOS notarization**: companion binary must be codesigned + notarized for non-dev distribution.
- **Windows WebView2**: bundled in Win11; older Win10 needs the runtime bootstrapper.
- **Lock file cleanup**: always remove `$TMPDIR/excalidraw-{sha256}.lock` on exit. Use a `Drop` impl to handle panics and signals.
- **Format detection**: detect by file *extension*, not by sniffing bytes. Extension is authoritative; fallback chain handles mismatches.
- **`scrollToContent: true`**: must be set on `initialData` passed to `<Excalidraw>` so the diagram auto-fits the window on first load.
