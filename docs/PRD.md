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
  * Static Excalidraw viewer bundle
  * Current file JSON via `/data`
* Watch file with notify/fs events
* Push reload event via WebSocket or SSE
* Launch WebView window using `wry`

---

### C) Web UI (served locally)

* Minimal HTML shell
* Loads Excalidraw renderer
* Fetches `/data`
* Listens for reload events
* Re-renders scene

---

## 9) Data Flow

```
save file in Zed
      ↓
file watcher triggers
      ↓
server notifies webview (ws/sse)
      ↓
webview refetches /data
      ↓
canvas re-renders
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
