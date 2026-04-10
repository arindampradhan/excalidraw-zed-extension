# Excalidraw Preview for Zed

A Zed editor extension that previews `.excalidraw` files in a native WebView window. Live-reloads on file save. No browser tabs. Pure offline, near-zero-latency diagram preview.

Supports all three Excalidraw file formats: `.excalidraw` (JSON), `.excalidraw.svg`, and `.excalidraw.png`.

## How It Works

```
Zed Extension (WASM)
      │
      │ spawn + file path
      ▼
excalidraw-preview (native binary)
      ├─ HTTP server (axum)           GET /config, /data, /events, /assets/*, /focus
      ├─ File watcher (notify)        80 ms debounce → SSE broadcast
      └─ WebView window (wry)
               └─ React app
                    └─ @excalidraw/excalidraw   loadFromBlob → updateScene
```

The Zed extension registers a `/preview-excalidraw` slash command. When invoked, it spawns a native companion binary that:
1. Starts a local HTTP server
2. Opens a WebView window pointing to it
3. Serves the diagram via `GET /data`
4. Pushes `reload` events via SSE whenever the file changes on disk

The React app in the webview uses `@excalidraw/excalidraw` to render the diagram. On each `reload` event it re-fetches `/data`, calls `loadFromBlob()`, and calls `excalidrawAPI.updateScene()` — no page refresh needed.

## Usage

1. Open any `.excalidraw`, `.excalidraw.svg`, or `.excalidraw.png` file in Zed.
2. Run `/preview-excalidraw` from the command palette.
3. A native window opens with the rendered diagram, auto-fitted to content.
4. Save the file in Zed — preview updates automatically (< 150 ms).

Re-running the command focuses the existing window instead of opening a new one.

## Requirements

- **macOS**: WebKit (built-in)
- **Linux**: `libwebkit2gtk-4.1-dev` or `libwebkit2gtk-4.0-dev`
- **Windows**: WebView2 (built-in on Win11; runtime bootstrapper needed on older Win10)
- **Rust** (via `rustup`) + **Node.js** for building from source

## Build

### Prerequisites

- **Rust** (via `rustup`) + **Cargo**
- **Node.js** (for building the webview)
- **macOS**: WebKit (built-in)
- **Linux**: `libwebkit2gtk-4.1-dev` or `libwebkit2gtk-4.0-dev` (e.g., `sudo apt install libwebkit2gtk-4.1-dev`)
- **Windows**: WebView2 (built-in on Win11; runtime bootstrapper needed on older Win10)

```bash
# one-time: install WASM target
rustup target add wasm32-wasip1
```

### Using Make (recommended)

```bash
make              # build UI + release binary (most common)
make release      # UI + binary + extension WASM
make symlink      # one-time: ~/.local/bin/excalidraw-preview → target/release
```

### Manual steps

```bash
# 1. Build the webview React app
cd preview-binary/webview-src && npm install && npm run build && cd ../..

# 2. Build companion binary
cargo build -p excalidraw-preview-binary --release

# 3. Build Zed extension WASM
cargo build -p excalidraw-preview --release --target wasm32-wasip1
```

### Install binary to PATH

```bash
make symlink   # symlinks ~/.local/bin/excalidraw-preview → target/release
               # so cargo build automatically updates what Zed sees
```

## Run Without Zed

```bash
./target/release/excalidraw-preview ./path/to/diagram.excalidraw --debug
./target/release/excalidraw-preview ./path/to/diagram.excalidraw.svg
./target/release/excalidraw-preview ./path/to/diagram.excalidraw.png
```

## Install Dev Extension in Zed

In Zed: open the command palette → **"zed: install dev extension"** → select the `./extension` directory.

## Development

```bash
make dev                              # Vite dev server + WebView window (uses preview-binary/test.excalidraw)
make dev DEV_FILE=path/to/file.excalidraw   # point at a specific file
```

Edit `App.tsx` (or any file in `preview-binary/webview-src/src/`) — Vite HMR updates the WebView instantly, no Rust rebuild needed. When done:

```bash
make ui && make build   # bake changes into the release binary
```

## Architecture

| Component | Target | Role |
|---|---|---|
| `extension/` | `wasm32-wasip1` | Slash command, spawns binary, focus ping |
| `preview-binary/` | native | HTTP server, file watcher, WebView window |
| `preview-binary/webview-src/` | — | React + Vite source (`@excalidraw/excalidraw`) |
| `preview-binary/assets/` | — | Vite build output, embedded in binary at compile time |
| `refs/excalidraw-vscode/` | — | Git submodule: reference implementation |

### HTTP Routes

| Route | Description |
|---|---|
| `GET /` | Viewer shell (`index.html`) |
| `GET /config` | JSON config: `{ contentType, name, theme }` |
| `GET /data` | Raw file bytes with correct `Content-Type` |
| `GET /events` | SSE stream — emits `data: reload` on file change |
| `GET /focus` | Brings window to front |
| `GET /assets/*` | Compiled React bundle + Excalidraw runtime assets (fonts, wasm) |

### File Format Support

| Extension | How it loads |
|---|---|
| `.excalidraw` | JSON — primary format |
| `.excalidraw.json` | JSON — alias |
| `.excalidraw.svg` | SVG with embedded scene data |
| `.excalidraw.png` | PNG with embedded scene data |

If `loadFromBlob` fails for the detected type, the webview tries the other two formats as fallback (mirrors the approach in `refs/excalidraw-vscode`).

### Webview JS Pipeline

```
fetch('/config') → contentType, theme
fetch('/data')   → ArrayBuffer
loadFromBlob(new Blob([bytes], { type: contentType }), null, null)
  → ExcalidrawInitialDataState { elements, appState, files }
<Excalidraw initialData={...} scrollToContent={true} />

onChange (debounced 600 ms) / Ctrl+S
  → POST /data  → written to disk

EventSource('/events')
  on 'reload' → re-fetch /data → loadFromBlob → excalidrawAPI.updateScene()
```

## Performance Targets

| Metric | Target |
|---|---|
| Window open time | < 400 ms |
| Reload latency after save | < 150 ms |
| Memory footprint | < 120 MB |
| CPU at idle | ~0% |

## Security

- HTTP server binds to `127.0.0.1` only — no external network access.
- Only the target file is read; no arbitrary filesystem access.
- All assets bundled at compile time — no CDN calls.

## Milestones

| Phase | Deliverable | Status |
|---|---|---|
| M1 | Rust binary opens WebView + static page | [ ] |
| M2 | `webview-src/` scaffolded; Vite builds; `<Excalidraw>` renders from `/data` | [ ] |
| M3 | File watcher + SSE + `updateScene` live reload | [ ] |
| M4 | Zed extension spawns binary, slash command end-to-end | [ ] |
| M5 | Process reuse / `/focus` + lock file | [ ] |
| M6 | All three file formats + fallback chain | [ ] |
| M7 | Cross-platform CI + prebuilt binary download | [ ] |

## Future (v2+)

- Multi-file tabs
- Remember window size and position
- `.excalidrawlib` workspace library panel
- Export to PNG/SVG via context menu
- Theme picker in window chrome
- Optional browser fallback mode
