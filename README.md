# Excalidraw Preview for Zed

A Zed editor extension that previews `.excalidraw` files in a native WebView window. Live-reloads on file save. No browser tabs. Pure offline, near-zero-latency diagram preview.

## How It Works

```
Zed Extension (WASM)
      │
      │ spawn + file path
      ▼
excalidraw-preview (native binary)
      ├─ HTTP server (axum)
      ├─ File watcher (notify)
      └─ WebView window (wry)
               └─ Excalidraw renderer (bundled)
```

The Zed extension registers a `/preview-excalidraw` slash command. When invoked, it spawns a native companion binary that opens a WebView window and serves the diagram locally. File changes are pushed to the UI via SSE — no polling.

## Usage

1. Open any `.excalidraw` file in Zed.
2. Run `/preview-excalidraw` from the command palette.
3. A native window opens with the rendered diagram.
4. Save the file in Zed — preview updates automatically.

Re-running the command focuses the existing window instead of opening a new one.

## Requirements

- **macOS**: WebKit (built-in)
- **Linux**: `libwebkit2gtk-4.1-dev` or `libwebkit2gtk-4.0-dev`
- **Windows**: WebView2 (built-in on Win11; runtime bootstrapper needed on older Win10)

## Build

```bash
# Build companion binary (native)
cargo build -p preview-binary --release

# Build Zed extension (WASM)
cargo build -p extension --release --target wasm32-wasip1
```

## Run Without Zed

```bash
./target/release/excalidraw-preview ./path/to/diagram.excalidraw --debug
```

## Install Dev Extension in Zed

```bash
zed --install-dev-extension ./extension
```

## Architecture

| Component | Target | Role |
|---|---|---|
| `extension/` | `wasm32-wasip1` | Slash command, spawns binary |
| `preview-binary/` | native | HTTP server, file watcher, WebView |
| `preview-binary/assets/` | — | Bundled HTML + Excalidraw JS |

### HTTP Routes (companion binary)

| Route | Description |
|---|---|
| `GET /` | Viewer shell (`index.html`) |
| `GET /data` | Raw `.excalidraw` JSON |
| `GET /events` | SSE stream — sends `reload` on file change |
| `GET /focus` | Brings window to front |
| `GET /assets/*` | Bundled static assets |

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
- All assets are bundled at compile time.

## Milestones

| Phase | Deliverable | Status |
|---|---|---|
| M1 | Rust binary opens WebView + static page | [ ] |
| M2 | HTTP server + `/data` + asset serving | [ ] |
| M3 | File watcher + SSE live reload | [ ] |
| M4 | Zed extension spawns binary | [ ] |
| M5 | Process reuse / `/focus` logic | [ ] |
| M6 | Cross-platform packaging + CI | [ ] |

## Future (v2+)

- Bidirectional editing (write back to file)
- Multi-file tabs
- Remember window positions
- `.excalidrawlib` support
- Optional browser fallback mode
