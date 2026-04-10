# Contributing to Excalidraw Preview for Zed

## Development

```bash
# Start Vite dev server + WebView window (uses preview-binary/test.excalidraw by default)
make dev

# Point at a specific file
make dev DEV_FILE=docs/examples/system-architecture.excalidraw
```

Open any file in the browser without restarting:
```
http://localhost:5173?file=/absolute/path/to/diagram.excalidraw
```

Multiple browser tabs can preview different files simultaneously — each gets its own SSE stream and file watcher.

Edit `App.tsx` (or any file in `preview-binary/webview-src/src/`) — Vite HMR updates the WebView instantly. When done:

```bash
make ui && make build   # bake changes into the release binary
```

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

## Architecture

| Component | Target | Role |
|---|---|---|
| `extension/` | `wasm32-wasip1` | Slash command, spawns binary, focus ping |
| `preview-binary/` | native | HTTP server, file watcher, WebView window |
| `preview-binary/webview-src/` | — | React + Vite source (`@excalidraw/excalidraw`) |
| `preview-binary/assets/` | — | Vite build output, embedded in binary at compile time |

### HTTP Routes

| Route | Description |
|---|---|
| `GET /` | Viewer shell (`index.html`) |
| `GET /config` | JSON: `{ contentType, name, theme, autoSave }` |
| `GET /data` | Raw file bytes with correct `Content-Type` |
| `POST /data` | Write edited scene back to disk (triggered by Ctrl+S or auto-save) |
| `GET /events` | SSE stream — emits `data: reload` on file change |
| `GET /focus` | Brings window to front |
| `GET /ping` | Liveness probe (used by lock file check) |
| `GET /shutdown` | Graceful shutdown (called when file is closed in Zed) |
| `GET /assets/*` | Compiled React bundle + Excalidraw runtime assets |

### File Format Support

| Extension | MIME type |
|---|---|
| `.excalidraw` | `application/json` |
| `.excalidraw.svg` | `image/svg+xml` |
| `.excalidraw.png` | `image/png` |

If `loadFromBlob` fails for the detected type, the webview tries the other two formats as fallback.

### Save Behaviour

| Mode | How to trigger |
|---|---|
| Manual (default) | Ctrl+S / Cmd+S or "Save to file" in the menu |
| Auto-save | Pass `--auto-save` flag; saves 600 ms after every element change |

On save the webview POSTs to `/data`. The file watcher fires, but the SSE echo is suppressed for 2 s to avoid a reload loop. Viewport position and theme are never reset on reload.

## Makefile targets

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

## Performance Targets

| Metric | Target |
|---|---|
| Window open time | < 400 ms |
| Reload latency after save | < 150 ms |
| Memory footprint | < 120 MB |
| CPU at idle | ~0% |

## Security

- HTTP server binds to `127.0.0.1` only — no external network access.
- Only the target file is read/written; no arbitrary filesystem access.
- All assets bundled at compile time — no CDN calls.

## Milestones

| Phase | Deliverable | Status |
|---|---|---|
| M1 | Rust binary opens WebView + static page | ✓ |
| M2 | `webview-src/` scaffolded; Vite builds; `<Excalidraw>` renders from `/data` | ✓ |
| M3 | File watcher + SSE + `updateScene` live reload | ✓ |
| M4 | Zed extension spawns binary, slash command end-to-end | ✓ |
| M5 | Process reuse / `/focus` + lock file | ✓ |
| M6 | All three file formats + fallback chain | ✓ |
| M7 | Cross-platform CI + prebuilt binary download | [ ] |

## Future (v2+)

- Multi-file tabs
- Remember window size and position
- `.excalidrawlib` workspace library panel
- Export to PNG/SVG via context menu
- Theme picker in window chrome
- Optional browser fallback mode
