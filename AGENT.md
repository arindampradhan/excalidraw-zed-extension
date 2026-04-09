# AGENT.md — Excalidraw Preview for Zed

> This file is the source of truth for AI agents working on this repo.
> `CLAUDE.md` is a symlink to this file.

## Project Purpose

A Zed editor extension that previews `.excalidraw` files in a native WebView window
(powered by `wry`). The preview live-reloads on file save. No browser tabs. No
in-editor UI panes. Pure offline, near-zero-latency diagram preview.

See [`docs/PRD.md`](docs/PRD.md) for the full product requirements.

---

## Repository Layout

```
excalidraw-zed-extension/
│
├── AGENT.md                   ← you are here (source of truth)
├── CLAUDE.md                  ← symlink → AGENT.md
├── docs/
│   └── PRD.md                 ← full product requirements doc
│
├── extension/                 ← Zed extension (Rust → WASM)
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs             ← extension entry point
│   └── extension.toml         ← Zed extension manifest
│
├── preview-binary/            ← companion native binary
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs            ← CLI entry, arg parsing, orchestration
│   │   ├── server.rs          ← HTTP server (axum/tiny-http) + SSE
│   │   ├── watcher.rs         ← file watcher (notify crate)
│   │   ├── webview.rs         ← wry WebView window lifecycle
│   │   └── assets.rs          ← embed & serve static assets
│   └── assets/
│       ├── index.html         ← viewer shell
│       ├── excalidraw.min.js  ← bundled Excalidraw renderer
│       └── loader.js          ← SSE listener + re-render glue
│
└── Cargo.toml                 ← workspace root
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

[slash_commands.preview-excalidraw]
description = "Open live preview for the active .excalidraw file"
requires_argument = false
```

### extension/src/lib.rs — responsibilities

1. Implement `zed_extension_api::Extension` trait.
2. Register `/preview-excalidraw` slash command.
3. On command run:
   - Get `worktree` + active file path via extension API.
   - Validate extension is `.excalidraw`.
   - Resolve path to companion binary (`excalidraw-preview`).
   - `std::process::Command::new(binary).arg(file_path).spawn()`.
4. Track spawned PIDs in a `HashMap<PathBuf, u32>` (in-memory, single process lifetime).
5. On re-invoke for same file: send HTTP ping to `http://127.0.0.1:{port}/focus` (port stored alongside PID).

**Key constraint:** WASM extensions cannot do async I/O or open sockets directly.
All networking/UI work must live in the companion binary.

---

## Component 2 — Companion Binary (`preview-binary/`)

**Target:** native (x86_64/aarch64, macOS/Linux/Windows)

### main.rs

```
CLI: excalidraw-preview <file-path> [--port <port>] [--debug]
```

Startup sequence:
1. Parse args.
2. Check if another instance owns this file (lock file or `/ping` probe).
3. If yes → send `/focus` request and exit.
4. Bind HTTP server on ephemeral port (or `--port`).
5. Write port to lock file: `$TMPDIR/excalidraw-preview-{hash(file)}.port`.
6. Spawn file watcher thread.
7. Open WebView window pointing to `http://127.0.0.1:{port}`.
8. Run event loop (blocks until window closes).
9. On exit: remove lock file.

### server.rs

Routes:
- `GET /`        → serve `index.html`
- `GET /data`    → serve raw file JSON (`Content-Type: application/json`)
- `GET /events`  → SSE stream; sends `data: reload\n\n` on file change
- `GET /focus`   → bring window to front (platform call via `wry`/`tao`)
- `GET /assets/*`→ serve bundled JS/CSS

Use `axum` (tokio) or `tiny-http` (sync). Prefer `axum` for SSE ergonomics.

### watcher.rs

Use `notify` crate (v6). Watch the target file with `RecommendedWatcher`.
On `EventKind::Modify` or `EventKind::Create` → broadcast to SSE channel
(use `tokio::sync::broadcast` or `std::sync::mpsc` bridged to async).

Debounce: 80 ms to avoid double-fire on save.

### webview.rs

Use `wry` + `tao` for cross-platform WebView window.

```rust
let event_loop = EventLoop::new();
let window = WindowBuilder::new()
    .with_title("Excalidraw Preview")
    .with_inner_size(LogicalSize::new(1200, 800))
    .build(&event_loop)?;
let webview = WebViewBuilder::new(window)?
    .with_url(&format!("http://127.0.0.1:{}", port))?
    .build()?;
event_loop.run(move |event, _, control_flow| { ... });
```

Focus handler: expose a channel that `/focus` HTTP route can signal; on signal
call `window.set_focus()`.

### assets.rs

Use `include_str!` / `include_bytes!` macros to embed `assets/` at compile time.
Return appropriate `Content-Type` headers.

---

## Component 3 — Web UI (`preview-binary/assets/`)

### index.html skeleton

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Excalidraw Preview</title>
  <style>body,html{margin:0;height:100%} #app{height:100%}</style>
</head>
<body>
  <div id="app"></div>
  <script src="/assets/excalidraw.min.js"></script>
  <script src="/assets/loader.js"></script>
</body>
</html>
```

### loader.js responsibilities

1. `fetch('/data')` → parse JSON → call `ExcalidrawLib.restore()` + render to `#app`.
2. Open `EventSource('/events')`.
3. On `message` with `data === 'reload'`: re-fetch `/data` and re-render.
4. On parse error: overlay `<div id="error">` with message.

---

## Key Dependencies

### extension/Cargo.toml

```toml
[dependencies]
zed_extension_api = "0.1"    # exact version per Zed extension SDK
```

### preview-binary/Cargo.toml

```toml
[dependencies]
wry        = "0.43"
tao        = "0.30"
axum       = { version = "0.8", features = ["tokio"] }
tokio      = { version = "1", features = ["full"] }
notify     = "6"
serde_json = "1"
clap       = { version = "4", features = ["derive"] }
tracing    = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[build-dependencies]
# none needed; assets embedded via include_bytes!
```

---

## Coding Conventions

- Use `anyhow::Result` for error propagation in the binary; no `unwrap` in prod paths.
- `tracing` for structured logs; gated behind `--debug` flag in release.
- No `unsafe` unless required by `wry`/`tao` platform calls.
- Format with `rustfmt` defaults; lint with `clippy -- -D warnings`.
- All public items must have doc comments.

---

## Build & Run

```bash
# Build companion binary (native)
cargo build -p preview-binary --release

# Build Zed extension (WASM)
cargo build -p extension --release --target wasm32-wasip1

# Run binary directly for testing (no Zed)
./target/release/excalidraw-preview ./path/to/file.excalidraw --debug

# Install extension into Zed (dev mode)
zed --install-dev-extension ./extension
```

---

## Testing Strategy

- **Unit**: `watcher.rs` debounce logic, `server.rs` route handlers (with `axum::test`).
- **Integration**: spawn binary against a temp `.excalidraw` file; assert `/data` returns valid JSON; mutate file; assert SSE fires within 150 ms.
- **Manual**: run against real Zed + `diagram.excalidraw`; measure reload latency with DevTools.

---

## Milestones

| Phase | Deliverable                              | Done? |
| ----- | ---------------------------------------- | ----- |
| M1    | Rust binary opens WebView + static page  | [ ]   |
| M2    | HTTP server + `/data` + asset serving    | [ ]   |
| M3    | File watcher + SSE live reload           | [ ]   |
| M4    | Zed extension spawns binary              | [ ]   |
| M5    | Process reuse / `/focus` logic           | [ ]   |
| M6    | Cross-platform packaging + CI            | [ ]   |

---

## Constraints & Gotchas

- **WASM sandbox**: extension code runs in `wasm32-wasip1`; no raw sockets, no
  `std::process` access beyond what Zed's host exposes. Companion binary handles
  all OS interaction.
- **Linux WebKitGTK**: require `libwebkit2gtk-4.1-dev` or `libwebkit2gtk-4.0-dev`.
  Print a clear error if missing.
- **macOS notarization**: companion binary must be codesigned for distribution.
- **Windows WebView2**: bundled in Win11; older Win10 needs runtime bootstrapper.
- **Port lock file**: use `/tmp/excalidraw-{sha256(canonical_path)}.lock` containing
  the port number as plain text. Remove on clean exit; check on startup.
