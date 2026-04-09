# Challenges & Solutions

## WebView on Linux (Wayland)

### Problem
The initial implementation using `wry::WebViewBuilder::new(&window)` failed on Linux with the error:

```
WebView error: Failed to create WebView: the window handle kind is not supported
```

This happened even though WebKitGTK (`webkit2gtk-4.1`) was installed. The issue is that `tao` (via `winit`) creates a Wayland-native window, but `wry::WebViewBuilder::new()` only supports X11 window handles on Linux.

### Root Cause
- `wry` on Linux uses WebKitGTK, which requires GTK windows
- `tao` creates native Wayland windows when running under Wayland compositors (Hyprland, GNOME, etc.)
- `wry::WebViewBuilder::new()` uses `HasWindowHandle`, which only works with X11 on Linux

### Solution
Use `wry::WebViewBuilderExtUnix::new_gtk()` which creates a GTK-based WebView that works on both X11 and Wayland:

```rust
#[cfg(target_os = "linux")]
fn run_webview(port: u16, _focus_rx: watch::Receiver<bool>) -> Result<(), Box<dyn std::error::Error>> {
    use gtk::glib::Propagation;
    use gtk::prelude::*;
    use wry::WebViewBuilderExtUnix;

    gtk::init().map_err(|e| format!("Failed to init GTK: {}", e))?;

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Excalidraw Preview");
    window.set_default_size(1200, 800);

    let url = format!("http://127.0.0.1:{}", port);

    let _webview = wry::WebViewBuilder::new_gtk(&window)
        .with_url(&url)
        .build()
        .map_err(|e| format!("Failed to create WebView: {}", e))?;

    window.show_all();

    window.connect_delete_event(move |_, _| {
        gtk::main_quit();
        Propagation::Proceed
    });

    gtk::main();

    Ok(())
}
```

### Dependencies Added

```toml
[target.'cfg(target_os = "linux")'.dependencies]
gtk = { version = "0.18", features = ["v3_24"] }
```

### Environment Variables (if issues persist)

Some Linux environments may require additional WebKit workarounds:

```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1
# or
WEBKIT_DISABLE_DMABUF_RENDERER=1
```

### Platform Summary

| Platform | WebView Method | Status |
|----------|---------------|--------|
| macOS | `wry::WebViewBuilder::new(&window)` | Works |
| Windows | `wry::WebViewBuilder::new(&window)` | Works |
| Linux (X11) | `wry::WebViewBuilder::new(&window)` | Works |
| Linux (Wayland) | `wry::WebViewBuilderExtUnix::new_gtk()` | Works |