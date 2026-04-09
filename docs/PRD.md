# PRD — Excalidraw Preview for Zed (External WebView Window)

## 1) Overview

Build a Zed extension that enables previewing `.excalidraw` files by launching a lightweight native WebView window rendered by a Rust companion app. The preview auto-updates as the file changes.

* Editor: Zed
* Diagram format/UI: Excalidraw
* WebView host: wry

This avoids in-editor UI (not supported by Zed extensions) while delivering a near-native preview workflow.

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

---

## 4) Users

* Developers/designers who keep architecture/flow diagrams as `.excalidraw` alongside code.
* Users of Zed who want fast diagram preview without leaving the editor.

---

## 5) User Experience

**Primary flow**

1. Open `diagram.excalidraw` in Zed.
2. Run `/preview-excalidraw`.
3. A native window opens showing the diagram.
4. On every save in Zed, preview updates automatically.

**Secondary flows**

* Re-running the command focuses the existing window.
* Closing the window stops the preview server for that file.

---

## 6) Functional Requirements

| ID  | Requirement                                                         |
| --- | ------------------------------------------------------------------- |
| FR1 | Zed command `/preview-excalidraw` available for `.excalidraw` files |
| FR2 | Extension reads active file path and spawns companion binary        |
| FR3 | Companion starts local HTTP server on ephemeral port                |
| FR4 | WebView window opens pointing to `http://localhost:{port}`          |
| FR5 | File watcher detects changes and pushes updates to UI               |
| FR6 | If window exists for file, focus instead of spawning new            |
| FR7 | Clean shutdown when window closes                                   |
| FR8 | Works offline (all assets bundled)                                  |

---

## 7) Architecture

```
Zed Extension (WASM, Rust)
        │
        │ spawn process + pass file path
        ▼
excalidraw-preview (Rust binary)
        ├─ File watcher
        ├─ HTTP server (serves UI + file JSON)
        └─ WebView window (wry)
                 └─ Excalidraw renderer (static assets)
```

---

## 8) Components

### A) Zed Extension (Rust → WASM)

Responsibilities:

* Register slash command
* Get current file path
* Spawn `excalidraw-preview <file>`
* Track PID per file
* Re-invoke = focus existing window (IPC ping)

No UI, no networking.

---

### B) Companion Binary: `excalidraw-preview`

Responsibilities:

* Start HTTP server
* Serve:
  * `GET /` → `index.html` (React app shell)
  * `GET /data` → current `.excalidraw` file JSON
  * `GET /events` → SSE stream (`data: reload\n\n` on file change)
  * `GET /assets/*` → compiled React/Excalidraw JS bundle **and** Excalidraw's own runtime assets (fonts, wasm) referenced via `window.EXCALIDRAW_ASSET_PATH = "/assets/"`. Both are embedded at compile time via `include_bytes!` in `assets.rs`.
* Watch file with `notify` crate (80 ms debounce)
* Push `reload` event via SSE broadcast channel
* Launch WebView window using `wry`

---

### C) Web UI — Runtime Behaviour

A single-page React app (built with Vite, served by the companion binary).

**`index.html`**

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Excalidraw Preview</title>
  <style>body,html{margin:0;height:100%} #root{height:100%}</style>
</head>
<body>
  <div id="root"></div>
  <div id="error" style="display:none;position:absolute;top:0;left:0;width:100%;padding:1em;background:#fee;color:#c00;font-family:monospace;white-space:pre;z-index:9999"></div>
  <script>
    window.EXCALIDRAW_ASSET_PATH = "/assets/";
    window.EXCALIDRAW_EXPORT_SOURCE = "excalidraw-zed-preview";
  </script>
  <script type="module" src="/assets/main.js"></script>
</body>
</html>
```

**`main.tsx` — startup sequence**

1. `fetch('/data')` → raw `.excalidraw` JSON bytes
2. `loadFromBlob(new Blob([bytes], { type: "application/json" }), null, null)` → `initialData`
   (uses `loadFromBlob` from `@excalidraw/excalidraw` — same approach as `excalidraw-vscode`, see `refs/excalidraw-vscode/webview/src/main.tsx`)
3. Render:
   ```tsx
   <Excalidraw
     initialData={{ ...initialData, scrollToContent: true }}
     viewModeEnabled={true}
     theme="auto"
     excalidrawAPI={(api) => setApi(api)}
   />
   ```
4. Open `new EventSource('/events')`
5. On `message` event with `data === 'reload'`:
   - Re-fetch `/data`
   - `loadFromBlob(...)` → `newData`
   - `api.updateScene(newData)` — debounced 150 ms to avoid double-fire
6. On any parse/fetch error → populate and show `#error` overlay

**Key npm dependencies**

| Package | Version | Purpose |
|---|---|---|
| `@excalidraw/excalidraw` | `^0.18.0` | Diagram renderer component |
| `react` / `react-dom` | `^18` | UI framework |

---

### D) Web UI — Build Pipeline

Source lives in `preview-binary/webview-src/` (adapted from `refs/excalidraw-vscode/webview/`).

```
preview-binary/
  webview-src/
    package.json        ← @excalidraw/excalidraw, react, vite
    vite.config.ts      ← outDir: "../assets", base: "/assets/"
    index.html
    src/
      main.tsx
      App.tsx
      styles.css
  assets/               ← Vite build output (committed or built in CI)
    index.html          ← served at GET /
    main.js             ← compiled React + Excalidraw bundle
    *.woff2, *.wasm     ← Excalidraw's own font/wasm assets
```

**Build command:**
```bash
cd preview-binary/webview-src && npm install && npm run build
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
    rollupOptions: {
      input: 'index.html',
    },
  },
});
```

**Embedding in Rust (`assets.rs`):**

`include_bytes!` embeds the entire `assets/` directory at compile time. `assets.rs` serves:
- Our compiled React bundle (`main.js`, `index.html`, CSS)
- Excalidraw's own runtime assets (fonts `.woff2`, Wasm) which the `@excalidraw/excalidraw` package fetches dynamically at `window.EXCALIDRAW_ASSET_PATH` (`/assets/`)

---

## 9) Data Flow

```
save file in Zed
      ↓
file watcher triggers (notify crate, 80 ms debounce)
      ↓
broadcast channel → SSE handler sends "data: reload\n\n"
      ↓
EventSource('/events') fires in webview JS
      ↓
fetch('/data') → raw .excalidraw JSON bytes
      ↓
loadFromBlob(new Blob([bytes], { type: "application/json" }))
      ↓
excalidrawAPI.updateScene(newData)   ← 150 ms debounce
      ↓
<Excalidraw> component re-renders canvas
```

---

## 10) IPC & Process Model

* Extension → binary: CLI args (file path)
* Binary → extension (optional): localhost ping to focus window
* Single instance per file
* Port chosen dynamically

---

## 11) Performance Targets

| Metric                    | Target   |
| ------------------------- | -------- |
| Window open time          | < 400 ms |
| Reload latency after save | < 150 ms |
| Memory footprint          | < 120 MB |
| CPU idle                  | ~0%      |

---

## 12) Packaging & Distribution

* Ship:
  * Zed extension (WASM)
  * Prebuilt binaries per OS (download on first run or bundled)
* No external dependencies beyond system WebView:
  * macOS: WebKit
  * Windows: WebView2
  * Linux: WebKitGTK

---

## 13) Edge Cases

| Case                            | Handling                 |
| ------------------------------- | ------------------------ |
| File deleted                    | Show error in preview    |
| Invalid JSON                    | Show parse error overlay |
| Multiple previews same file     | Focus existing           |
| Port collision                  | Auto retry               |
| WebView backend missing (Linux) | Clear install message    |

---

## 14) Security

* Bind server to `127.0.0.1` only
* No external network access
* Serve static assets only
* No arbitrary file reads beyond target file

---

## 15) Observability (dev)

* `--debug` flag for logs
* Console logs in WebView
* Structured logs for file events

---

## 16) Milestones

| Phase | Deliverable                        |
| ----- | ---------------------------------- |
| M1    | Rust binary: WebView + static page |
| M2    | HTTP server + file serving         |
| M3    | File watcher + live reload         |
| M4    | Zed extension spawning binary      |
| M5    | Process reuse/focus logic          |
| M6    | Cross-platform packaging           |

---

## 17) Success Criteria

* Preview works reliably across OSes.
* Update feels instantaneous.
* No browser tabs used.
* Installation friction minimal.

---

## 18) Future Enhancements (v2+)

* Bidirectional editing (write back to file)
* Multi-file tabs
* Remember window positions
* Support `.excalidrawlib`
* Optional browser fallback mode

---

## 19) Risks

| Risk                         | Mitigation               |
| ---------------------------- | ------------------------ |
| Linux WebKit issues          | Provide browser fallback |
| Zed extension process limits | Keep extension minimal   |
| Excalidraw bundle size       | Minify and cache         |

---

## 20) Why this fits Zed

This follows Zed's intended model:

* Extension orchestrates tools
* UI lives outside editor
* Rust-first, WASM-safe, system-level integration
