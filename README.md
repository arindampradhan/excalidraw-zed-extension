# Excalidraw Preview for Zed

A Zed editor extension that previews `.excalidraw` files in a native WebView window. Live-reloads on file save. No browser tabs. Pure offline, near-zero-latency diagram preview.

Supports `.excalidraw` (JSON), `.excalidraw.svg`, and `.excalidraw.png`.

## Installation

### Prerequisites

- **Rust** (via `rustup`) + **Cargo**
- **Node.js** (for building the webview)
- **macOS**: WebKit (built-in)
- **Linux**: `sudo apt install libwebkit2gtk-4.1-dev`
- **Windows**: WebView2 (built-in on Win11)

### Build

```bash
# one-time: install WASM target
rustup target add wasm32-wasip1

# build UI + release binary
make

# one-time: symlink binary to PATH
make symlink
```

### Install Extension

In Zed: command palette → **"zed: install dev extension"** → select the `./extension` directory.

## Usage



https://github.com/user-attachments/assets/af3cd686-56b8-413c-9012-0f1c75e5f6c9



1. Open any `.excalidraw`, `.excalidraw.svg`, or `.excalidraw.png` file in Zed.
2. Run `/preview-excalidraw` from the command palette.
3. A native window opens with the rendered diagram.
4. Save the file in Zed — preview updates automatically.

Re-running the command focuses the existing window instead of opening a new one.

### Auto-save

Pass `--auto-save` to enable debounced auto-save (600 ms after every change):

```
/preview-excalidraw --auto-save
```

### Run Without Zed

```bash
./target/release/excalidraw-preview ./path/to/diagram.excalidraw --debug
./target/release/excalidraw-preview ./path/to/diagram.excalidraw.svg
```
