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
│   │   ├── main.rs                 ← CLI entry, arg parsing, orchestration
│   │   ├── server.rs               ← axum HTTP server + SSE + all routes
│   │   ├── watcher.rs              ← notify file watcher, 80 ms debounce
│   │   ├── webview.rs              ← wry WebView window lifecycle
│   │   └── assets.rs               ← include_bytes! embed + serve assets/
│   ├── webview-src/                ← React + Vite source (npm project)
│   │   ├── package.json            ← @excalidraw/excalidraw ^0.18, react ^18, vite
│   │   ├── vite.config.ts          ← outDir: "../assets", base: "/assets/"
│   │   ├── index.html
│   │   └── src/
│   │       ├── main.tsx            ← fetch /config + /data → loadFromBlob → render, SSE
│   │       ├── App.tsx             ← <Excalidraw> view-only wrapper
│   │       ├── useOsTheme.ts       ← prefers-color-scheme → "light"|"dark"
│   │       └── styles.css
│   └── assets/                     ← Vite build output; committed; embedded at compile time
│       ├── index.html              ← served at GET /
│       └── assets/
│           ├── main-[hash].js      ← React + Excalidraw bundle
│           ├── main-[hash].css
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

### main.rs

```
CLI: excalidraw-preview <file-path> [--port <port>] [--debug]
```

Startup sequence:
1. Parse args with `clap`.
2. Detect file format from extension → `ContentType` enum (`Json` | `Svg` | `Png`).
3. Check lock file `$TMPDIR/excalidraw-{sha256(canonical_path)}.lock`.
   - If exists and port is live (`GET /ping` succeeds) → send `GET /focus` and exit.
   - If exists but stale → remove and continue.
4. Bind axum server on ephemeral port (or `--port`).
5. Write port to lock file as plain text.
6. Spawn file watcher thread (see `watcher.rs`).
7. Open WebView window pointing to `http://127.0.0.1:{port}` (see `webview.rs`).
8. Run event loop — blocks until window closes.
9. On exit: remove lock file, shut down server.

### server.rs

All routes served by axum (tokio):

| Route | Handler |
|---|---|
| `GET /` | serve embedded `index.html` |
| `GET /config` | return `{ contentType, name, theme: "auto" }` as JSON |
| `GET /data` | read file from disk, return bytes with correct `Content-Type` |
| `GET /events` | SSE stream; subscribe to broadcast channel; emit `data: reload\n\n` |
| `GET /focus` | signal webview window to call `window.set_focus()` |
| `GET /ping` | return 200 OK (liveness probe) |
| `GET /assets/*` | serve embedded assets from `assets.rs` |

SSE channel: `tokio::sync::broadcast::channel::<()>(16)`. Watcher sends `()` on file change; SSE handler subscribes and streams.

### watcher.rs

- Use `notify` v6 `RecommendedWatcher`.
- Watch target file for `EventKind::Modify(_)` and `EventKind::Create(_)`.
- Debounce: 80 ms (drop duplicate events within window).
- On trigger: `broadcast_tx.send(())`.

### webview.rs

```rust
let event_loop = EventLoop::new();
let window = WindowBuilder::new()
    .with_title("Excalidraw Preview")
    .with_inner_size(LogicalSize::new(1200, 800))
    .build(&event_loop)?;
let webview = WebViewBuilder::new(&window)
    .with_url(&format!("http://127.0.0.1:{}", port))
    .build()?;
event_loop.run(move |event, _, control_flow| {
    match event {
        Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
            *control_flow = ControlFlow::Exit;
        }
        _ => *control_flow = ControlFlow::Wait,
    }
});
```

Focus signal: a `tokio::sync::watch` channel; `/focus` route sends to it; the event loop polls it and calls `window.set_focus()`.

### assets.rs

Embeds the entire `assets/` directory at compile time. Use `rust-embed` crate (or manual `include_bytes!` per file) to serve with correct `Content-Type` headers keyed by file extension.

---

## Component 3 — Web UI (`preview-binary/webview-src/`)

Reference: `refs/excalidraw-vscode/webview/` — adapt, don't copy verbatim (no VS Code APIs).

### Key differences from excalidraw-vscode

| excalidraw-vscode | This project |
|---|---|
| Config via Base64 HTML attribute | Config via `GET /config` HTTP endpoint |
| File content via same Base64 attribute | File content via `GET /data` HTTP endpoint |
| Live updates via VS Code `postMessage` | Live updates via SSE `EventSource('/events')` |
| `vscode.postMessage` write-back | Read-only, no write-back (v1) |
| Asset path via `asWebviewUri` | Asset path hardcoded to `/assets/` |

### main.tsx — startup sequence

```ts
// 1. Fetch config
const config = await fetch('/config').then(r => r.json());
// config: { contentType: string, name: string, theme: "auto" }

// 2. Fetch raw file bytes
const bytes = await fetch('/data').then(r => r.arrayBuffer());

// 3. Load with format fallback chain (same logic as refs/excalidraw-vscode/webview/src/main.tsx)
let initialData: ExcalidrawInitialDataState;
const types = reorderFallbacks(config.contentType);  // try declared type first
for (const type of types) {
  try {
    initialData = await loadFromBlob(
      new Blob([bytes], { type }),
      null, null
    );
    break;
  } catch { /* try next */ }
}
if (!initialData) { showError("Failed to load file"); return; }

// 4. Render
ReactDOM.createRoot(document.getElementById('root')!).render(
  <App initialData={initialData} config={config} />
);

// 5. SSE live reload
const es = new EventSource('/events');
es.onmessage = debounce(async () => {
  const bytes = await fetch('/data').then(r => r.arrayBuffer());
  const newData = await loadFromBlob(new Blob([bytes], { type: config.contentType }), null, null);
  excalidrawApi?.updateScene(newData);
}, 150);
```

### App.tsx — Excalidraw component

```tsx
import { Excalidraw } from "@excalidraw/excalidraw";
import "@excalidraw/excalidraw/index.css";

export default function App({ initialData, config }) {
  const [api, setApi] = useState<ExcalidrawImperativeAPI>();
  const theme = useOsTheme(config.theme); // "light" | "dark"

  return (
    <div style={{ height: "100%" }}>
      <Excalidraw
        excalidrawAPI={setApi}
        initialData={{ ...initialData, scrollToContent: true }}
        viewModeEnabled={true}
        theme={theme}
        name={config.name}
        UIOptions={{
          canvasActions: { loadScene: false, saveToActiveFile: false, export: false },
        }}
      />
    </div>
  );
}
```

### useOsTheme.ts

```ts
export function useOsTheme(preference: "auto" | "light" | "dark"): "light" | "dark" {
  const [theme, setTheme] = useState<"light" | "dark">(
    preference !== "auto" ? preference :
    window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
  );
  useEffect(() => {
    if (preference !== "auto") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => setTheme(e.matches ? "dark" : "light");
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [preference]);
  return theme;
}
```

### File Format Fallback Chain

```ts
function reorderFallbacks(primary: string): string[] {
  const all = ["application/json", "image/svg+xml", "image/png"];
  return [primary, ...all.filter(t => t !== primary)];
}
```

Matches logic in `refs/excalidraw-vscode/webview/src/main.tsx`.

### window.EXCALIDRAW_ASSET_PATH

Must be set **before** the React bundle loads:
```html
<script>
  window.EXCALIDRAW_ASSET_PATH = "/assets/";
</script>
<script type="module" src="/assets/main.js"></script>
```

`@excalidraw/excalidraw` uses this to fetch its fonts and wasm modules at runtime. All these files are served by `assets.rs` from the embedded `assets/` directory.

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
make dev DEV_FILE=docs/examples/software-development-lifecycle.excalidraw
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
| M1    | Rust binary opens wry window + serves static index.html | [ ] |
| M2    | `webview-src/` scaffolded; Vite builds; `<Excalidraw>` renders from `/data` | [ ] |
| M3    | File watcher + SSE + `updateScene` live reload | [ ] |
| M4    | Zed extension spawns binary; slash command works end-to-end | [ ] |
| M5    | Process reuse: lock file + `/focus` + `/ping` | [ ] |
| M6    | All three file formats + fallback chain | [ ] |
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
