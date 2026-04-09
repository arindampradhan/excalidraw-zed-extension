# PRD — Excalidraw Preview for Zed (External WebView Window)

## 1) Overview

Build a Zed extension that enables previewing `.excalidraw` files by launching a lightweight native WebView window rendered by a Rust companion app. The preview auto-updates as the file changes.

* Editor: Zed
* Diagram format/UI: Excalidraw (`@excalidraw/excalidraw ^0.18.0`)
* WebView host: `wry` + `tao`
* HTTP server: `axum` (tokio)

This avoids in-editor UI (not supported by Zed extensions) while delivering a near-native preview workflow.

Reference implementation: `refs/excalidraw-vscode/` (git submodule) — the VS Code extension this is adapted from. Key differences: we replace VS Code's `postMessage` API with SSE + HTTP, and serve everything through axum rather than the VS Code webview host.

---

## 2) Goals

* Preview any `.excalidraw` file from Zed with one command.
* Live reload on file save (≤150 ms perceived update).
* No browser tab; open a small native window.
* Cross-platform: macOS, Linux, Windows.
* Minimal install friction and small runtime footprint.

---

## 3) Non-Goals

* Embedding preview inside Zed panes.
* Editing Excalidraw from the preview window (view-only v1).
* Multi-file session management (single file per window v1).
* Library item management (v1 loads drawing only, no sidebar library).

---

## 4) Users

* Developers/designers who keep architecture/flow diagrams as `.excalidraw` alongside code.
* Users of Zed who want fast diagram preview without leaving the editor.

---

## 5) User Experience

**Primary flow**

1. Open `diagram.excalidraw` in Zed.
2. Run `/preview-excalidraw`.
3. A native window opens showing the diagram, auto-fitted to content.
4. On every save in Zed, preview updates automatically.

**Secondary flows**

* Re-running the command focuses the existing window.
* Closing the window stops the preview server for that file.

---

## 6) Functional Requirements

| ID   | Requirement |
| ---- | ----------- |
| FR1  | Zed command `/preview-excalidraw` available for `.excalidraw` files |
| FR2  | Extension reads active file path and spawns companion binary |
| FR3  | Companion starts local HTTP server on ephemeral port |
| FR4  | WebView window opens pointing to `http://127.0.0.1:{port}` |
| FR5  | File watcher detects changes and pushes reload event to UI via SSE |
| FR6  | If window exists for file, focus instead of spawning new |
| FR7  | Clean shutdown when window closes |
| FR8  | Works offline (all assets bundled — no CDN calls) |
| FR9  | Supports all three Excalidraw file formats: `.excalidraw` (JSON), `.excalidraw.svg`, `.excalidraw.png` |
| FR10 | Auto-fits diagram to window on initial load (`scrollToContent: true`) |
| FR11 | Theme follows OS dark/light mode preference (`prefers-color-scheme`) |

---

## 7) Architecture

```
Zed Extension (WASM, Rust)
        │
        │ spawn process + pass file path
        ▼
excalidraw-preview (Rust binary)
        ├─ File watcher (notify)
        ├─ HTTP server (axum + tokio)
        │       ├─ GET /           → index.html
        │       ├─ GET /config     → JSON config for the webview
        │       ├─ GET /data       → raw file bytes
        │       ├─ GET /events     → SSE stream
        │       └─ GET /assets/*   → compiled React bundle + Excalidraw runtime assets
        └─ WebView window (wry)
                 └─ React app (@excalidraw/excalidraw)
```

---

## 8) Components

### A) Zed Extension (Rust → WASM)

Responsibilities:

* Register `/preview-excalidraw` slash command
* Get current file path from worktree
* Validate `.excalidraw`, `.excalidraw.svg`, or `.excalidraw.png` extension
* Spawn `excalidraw-preview <file>` via `zed_extension_api::process::Command`
* Track spawned PIDs + port in a `HashMap<PathBuf, (u32, u16)>`
* On re-invoke for same file: send `GET http://127.0.0.1:{port}/focus`

No UI, no direct file I/O, no networking beyond the focus ping.

---

### B) Companion Binary: `excalidraw-preview`

Responsibilities:

* Parse CLI: `excalidraw-preview <file-path> [--port <port>] [--debug]`
* Detect file format from extension (see §8E)
* Start HTTP server on ephemeral port
* Expose routes:
  * `GET /` → serve `index.html`
  * `GET /config` → serve JSON config object (see §8C)
  * `GET /data` → serve raw file bytes with correct `Content-Type`
  * `GET /events` → SSE stream; emit `data: reload\n\n` on file change
  * `GET /focus` → signal the window to come to the front
  * `GET /assets/*` → serve embedded React bundle + Excalidraw runtime assets (fonts, wasm, etc.)
* Write port to lock file: `$TMPDIR/excalidraw-{sha256(canonical_path)}.lock`
* Watch file with `notify` crate (80 ms debounce)
* On file change: broadcast on `tokio::sync::broadcast` channel → SSE clients
* Open WebView window via `wry` pointing to `http://127.0.0.1:{port}`
* On window close: remove lock file, shut down server

`assets.rs` embeds the entire `assets/` build output at compile time via `include_bytes!`. It serves:
- Our compiled React + Excalidraw bundle (`main.js`, CSS)
- Excalidraw's own runtime assets (`.woff2` fonts, `.wasm`) which `@excalidraw/excalidraw` fetches dynamically using `window.EXCALIDRAW_ASSET_PATH`

---

### C) Web UI — Runtime Behaviour

A single-page React app served by the companion binary. Adapted from `refs/excalidraw-vscode/webview/`.

#### `index.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Excalidraw Preview</title>
  <style>
    body, html { margin: 0; height: 100%; overflow: hidden; }
    #root { height: 100%; }
    #error {
      display: none; position: fixed; top: 0; left: 0; width: 100%;
      padding: 1em; background: #fee2e2; color: #991b1b;
      font-family: monospace; white-space: pre-wrap; z-index: 9999;
    }
  </style>
</head>
<body>
  <div id="root"></div>
  <div id="error"></div>
  <script>
    /* Excalidraw fetches its own fonts/wasm from this path at runtime */
    window.EXCALIDRAW_ASSET_PATH = "/assets/";
    window.EXCALIDRAW_EXPORT_SOURCE = "excalidraw-zed-preview";
  </script>
  <script type="module" src="/assets/main.js"></script>
</body>
</html>
```

#### `GET /config` — config object served by the binary

```json
{
  "contentType": "application/json",
  "name": "diagram",
  "theme": "auto"
}
```

| Field | Values | Source |
|---|---|---|
| `contentType` | `application/json` \| `image/svg+xml` \| `image/png` | detected from file extension |
| `name` | filename stem | file path |
| `theme` | `"auto"` (follows OS) | hardcoded v1; configurable v2 |

#### `main.tsx` — startup sequence

```
1. fetch('/config')              → parse { contentType, name, theme }
2. fetch('/data')                → ArrayBuffer (raw file bytes)
3. loadFromBlob(
     new Blob([bytes], { type: contentType }),
     null, null
   )                             → ExcalidrawInitialDataState
     ↓ on failure: try fallback content types (JSON → SVG → PNG)
4. ReactDOM.render(
     <App
       initialData={{ ...data, scrollToContent: true }}
       viewModeEnabled={true}
       theme={config.theme}
       name={config.name}
     />
   )
5. new EventSource('/events')
6. on message { data: 'reload' }:
     fetch('/data') → loadFromBlob(...)
     → excalidrawAPI.updateScene(newData)   [150 ms debounce]
7. on fetch/parse error:
     show #error overlay with message
```

#### `App.tsx` — Excalidraw component

```tsx
import { Excalidraw, loadFromBlob } from "@excalidraw/excalidraw";
import "@excalidraw/excalidraw/index.css";

export default function App({ initialData, viewModeEnabled, theme, name }) {
  const [api, setApi] = useState<ExcalidrawImperativeAPI>();

  // Theme: "auto" resolves via prefers-color-scheme media query
  const resolvedTheme = useOsTheme(theme);  // "light" | "dark"

  return (
    <div style={{ height: "100%" }}>
      <Excalidraw
        excalidrawAPI={setApi}
        initialData={initialData}
        viewModeEnabled={viewModeEnabled}
        theme={resolvedTheme}
        name={name}
        UIOptions={{
          canvasActions: {
            loadScene: false,
            saveToActiveFile: false,
            export: false,
          },
        }}
      />
    </div>
  );
}
```

The `useOsTheme` hook watches `window.matchMedia("(prefers-color-scheme: dark)")` and returns `"dark"` or `"light"`.

#### npm dependencies

| Package | Version | Purpose |
|---|---|---|
| `@excalidraw/excalidraw` | `^0.18.0` | Diagram renderer + load/export APIs |
| `react` / `react-dom` | `^18` | UI framework |
| `vite` + `@vitejs/plugin-react` | latest | Build toolchain |

---

### D) Web UI — Build Pipeline

Source: `preview-binary/webview-src/` (adapted from `refs/excalidraw-vscode/webview/`)

```
preview-binary/
  webview-src/
    package.json
    vite.config.ts
    index.html
    src/
      main.tsx          ← entry point; fetch config + data; SSE; render
      App.tsx           ← <Excalidraw> wrapper
      useOsTheme.ts     ← prefers-color-scheme hook
      styles.css
  assets/               ← Vite build output; embedded in Rust binary
    index.html
    assets/
      main-[hash].js
      main-[hash].css
      *.woff2           ← Excalidraw fonts
      *.wasm            ← Excalidraw wasm modules
```

**`vite.config.ts`:**
```ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
export default defineConfig({
  plugins: [react()],
  base: '/assets/',
  build: {
    outDir: '../assets',
    emptyOutDir: true,
  },
});
```

**Build command** (run before `cargo build`):
```bash
cd preview-binary/webview-src && npm install && npm run build
```

The `assets/` directory is committed to the repo so Rust can embed it without requiring Node at build time. CI runs the npm build step first.

---

### E) File Format Support

Mirrors `refs/excalidraw-vscode/src/document.ts` (`getContentType`) and `refs/excalidraw-vscode/webview/src/main.tsx` (format fallback chain).

| Extension | MIME type | `GET /data` Content-Type | Notes |
|---|---|---|---|
| `.excalidraw` | `application/json` | `application/json` | Primary format |
| `.excalidraw.json` | `application/json` | `application/json` | Alias |
| `.excalidraw.svg` | `image/svg+xml` | `image/svg+xml` | SVG with embedded scene JSON |
| `.excalidraw.png` | `image/png` | `image/png` | PNG with embedded scene data |

**Format fallback chain** (implemented in `main.tsx`):

If `loadFromBlob` fails with the declared content type, try the others in order:
- Declared JSON → try PNG → try SVG
- Declared SVG → try JSON → try PNG
- Declared PNG → try JSON → try SVG

This tolerates misnamed files.

**`loadFromBlob` signature** (from `@excalidraw/excalidraw`):
```ts
loadFromBlob(
  blob: Blob,
  localAppState: AppState | null,
  localElements: readonly ExcalidrawElement[] | null,
  fileHandle?: FileSystemFileHandle | null
): Promise<ExcalidrawInitialDataState>
```

Returns `{ elements, appState, files }` — passed directly as `initialData` to `<Excalidraw>`.

---

## 9) Data Flow

```
save file in Zed
      ↓
file watcher triggers (notify crate, 80 ms debounce)
      ↓
broadcast channel → SSE handler emits "data: reload\n\n"
      ↓
EventSource('/events') fires in webview JS
      ↓
fetch('/data') → ArrayBuffer of raw file bytes
      ↓
new Blob([bytes], { type: contentType })
      ↓
loadFromBlob(blob, null, null) → ExcalidrawInitialDataState
      ↓ (150 ms debounce)
excalidrawAPI.updateScene({ elements, appState, files })
      ↓
<Excalidraw> component re-renders canvas
```

---

## 10) IPC & Process Model

* Extension → binary: CLI args (`excalidraw-preview <file>`)
* Extension → binary (focus): `GET http://127.0.0.1:{port}/focus`
* Port discovery: lock file at `$TMPDIR/excalidraw-{sha256(path)}.lock` containing plain-text port number
* Single instance per file; startup probe checks lock file + `/ping` endpoint
* On clean exit: lock file removed

---

## 11) Performance Targets

| Metric | Target |
| --- | --- |
| Window open time | < 400 ms |
| Reload latency after save | < 150 ms |
| Memory footprint | < 120 MB |
| CPU at idle | ~0% |

---

## 12) Packaging & Distribution

* Ship:
  * Zed extension (WASM)
  * Prebuilt native binary per OS/arch (downloaded on first run or side-loaded)
* System WebView requirements:
  * macOS: WebKit (built-in)
  * Windows: WebView2 (built-in Win11; bootstrapper needed for older Win10)
  * Linux: `libwebkit2gtk-4.1-dev` or `libwebkit2gtk-4.0-dev`

---

## 13) Edge Cases

| Case | Handling |
| --- | --- |
| File deleted while open | SSE sends `reload`; `/data` returns 404; JS shows error overlay |
| Invalid / truncated JSON | `loadFromBlob` throws; JS shows error overlay with message |
| Malformed SVG/PNG | Format fallback chain tries other types; shows error if all fail |
| Multiple invocations same file | Lock file found + `/ping` succeeds → send `/focus`, exit |
| Port collision on bind | Auto-retry with next available port (up to 10 attempts) |
| WebView backend missing (Linux) | Binary prints clear error: "Install libwebkit2gtk-4.1-dev" |
| SSE client disconnects | Server drops the sender; no crash |
| Binary not found by extension | Extension shows error message in Zed assistant panel |

---

## 14) Security

* HTTP server binds to `127.0.0.1` only — no external network access
* Only the target file is read; no arbitrary filesystem traversal
* All assets bundled at compile time — no CDN, no external fetches
* `window.EXCALIDRAW_ASSET_PATH` points to local server only

---

## 15) Observability (dev)

* `--debug` flag enables `tracing` output to stderr
* `zed --foreground` surfaces extension stdout/stderr in the terminal
* WebView DevTools can be opened in debug builds via `wry` feature flag

---

## 16) Milestones

| Phase | Deliverable | Status |
| --- | --- | --- |
| M1 | Rust binary: wry window opens, serves static `index.html` | [x] |
| M2 | `webview-src/` scaffolded; Vite builds; `<Excalidraw>` renders from `/data` | [x] |
| M3 | File watcher + SSE + `updateScene` live reload working | [x] |
| M4 | Zed extension spawns binary, slash command works end-to-end | [ ] |
| M5 | Process reuse: lock file + `/focus` route | [x] |
| M6 | All three file formats (JSON/SVG/PNG) with fallback chain | [ ] |
| M7 | Cross-platform CI: build matrix + prebuilt binary download | [ ] |

## 16.1) TODO List

### Companion Binary (excalidraw-preview)
- [x] HTTP server with routes: `/`, `/config`, `/data`, `/events`, `/focus`, `/ping`, `/assets/*`
- [x] WebView window via wry/tao
- [x] File watcher with 80ms debounce
- [x] SSE live reload
- [x] Lock file for process reuse
- [x] Graceful error handling for headless environments
- [ ] `/focus` route: actually bring window to front
- [ ] Clean shutdown via signal handling (SIGTERM, SIGINT)
- [ ] Windows support
- [ ] macOS support

### WebView (React/Excalidraw)
- [x] Vite build setup with `base: '/assets/'`
- [x] Fetch config and data from server
- [x] Render Excalidraw diagram
- [x] `scrollToContent: true` on initial load
- [x] OS theme detection (prefers-color-scheme)
- [x] SSE live reload with debounce
- [x] Error overlay for failed loads
- [ ] Format fallback chain (JSON → SVG → PNG)
- [ ] SVG format support
- [ ] PNG format support

### Zed Extension
- [ ] WASM extension scaffold
- [ ] `/preview-excalidraw` slash command
- [ ] Spawn companion binary
- [ ] Track PIDs + ports per file
- [ ] Focus existing window on re-invoke

### Testing & CI
- [ ] Integration tests for companion binary
- [ ] Cross-platform CI (macOS, Linux, Windows)
- [ ] Prebuilt binary releases

---

## 17) Success Criteria

* Preview opens in < 400 ms on all three platforms.
* File save → canvas update in < 150 ms.
* No browser tabs used.
* All three Excalidraw file formats render correctly.
* Installation requires no manual steps beyond installing the Zed extension.

---

## 18) Future Enhancements (v2+)

* Bidirectional editing (write back to file — `serializeAsJSON` / `exportToSvg`)
* Multi-file tabs
* Remember window size and position
* `.excalidrawlib` workspace library panel
* Export to PNG/SVG via context menu
* Theme picker in window chrome
* Optional browser fallback mode

---

## 19) Risks

| Risk | Mitigation |
| --- | --- |
| Linux WebKitGTK version variance | Test against both 4.0 and 4.1; print clear error if missing |
| Zed extension WASM process limits | Keep extension minimal; all work in native binary |
| `@excalidraw/excalidraw` bundle size (~3 MB gzipped) | Accept size; assets embedded once in binary |
| `include_bytes!` compile time for large asset dir | Use `rust-embed` crate for lazy loading if needed |
| macOS codesigning for binary distribution | Must sign + notarize for non-dev distribution |

---

## 20) Why This Fits Zed

This follows Zed's intended extension model:

* Extension orchestrates native tools via `spawn`
* UI lives outside the editor in a dedicated window
* Rust-first, WASM-safe, system-level integration
* No in-editor pane hacks required
