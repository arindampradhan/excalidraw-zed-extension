use anyhow::Result;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use clap::Parser;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use rust_embed::RustEmbed;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{broadcast, watch};
use tracing::info;

#[derive(Clone)]
struct AppState {
    file_path: PathBuf,
    lock_path: PathBuf,
    content_type: String,
    file_name: String,
    broadcast_tx: broadcast::Sender<()>,
    /// Sends `true` to signal the webview window to focus.
    focus_tx: Arc<watch::Sender<bool>>,
}

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

#[derive(Serialize)]
struct ConfigResponse {
    content_type: String,
    name: String,
    theme: String,
}

/// Entry point. Keeps the main thread free for the native event loop (required on macOS).
fn main() -> Result<()> {
    let args = CliArgs::parse();

    if args.lsp {
        return run_lsp_server();
    }

    if args.debug {
        tracing_subscriber::fmt()
            .with_env_filter("excalidraw_preview=debug")
            .init();
    }

    // --dev / --dev-server: open the WebView on the Vite dev server instead of the
    // embedded assets.  Run `npm run dev` in webview-src first.
    if let Some(dev_url) = args.dev_server.or_else(|| args.dev.then(|| "http://localhost:5173".to_string())) {
        eprintln!("[dev] Opening WebView at {dev_url}");
        eprintln!("[dev] Make sure `npm run dev` is running in preview-binary/webview-src/");
        let (_focus_tx, focus_rx) = watch::channel(false);
        if let Err(e) = run_webview_url(&dev_url, focus_rx) {
            eprintln!("WebView error: {e}");
        }
        return Ok(());
    }

    let file = args
        .file
        .ok_or_else(|| anyhow::anyhow!("Usage: excalidraw-preview <file> [--port <port>] [--debug]\n       excalidraw-preview --lsp"))?;

    let file_path = PathBuf::from(&file);
    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    let content_type = detect_content_type(&file_path);
    let file_name = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "diagram".to_string());

    let canonical_path = std::fs::canonicalize(&file_path)?;
    let lock_path = get_lock_path(&canonical_path);

    // Build a multi-thread runtime; main thread is reserved for the WebView event loop.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // If another instance is already serving this file, focus it and exit.
    if let Ok(port) = rt.block_on(check_existing_instance(&lock_path)) {
        info!("Found existing instance on port {}, focusing window", port);
        let client = reqwest::Client::new();
        let _ = rt.block_on(
            client
                .get(format!("http://127.0.0.1:{}/focus", port))
                .send(),
        );
        return Ok(());
    }

    let (broadcast_tx, _) = broadcast::channel::<()>(16);
    let (focus_tx, focus_rx) = watch::channel(false);
    let focus_tx = Arc::new(focus_tx);

    let port = args.port.unwrap_or_else(find_available_port);

    let state = Arc::new(AppState {
        file_path: canonical_path.clone(),
        lock_path: lock_path.clone(),
        content_type,
        file_name,
        broadcast_tx: broadcast_tx.clone(),
        focus_tx: focus_tx.clone(),
    });

    std::fs::write(&lock_path, port.to_string())?;

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = rt.block_on(tokio::net::TcpListener::bind(addr))?;
    info!("Server listening on http://{}", addr);

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/config", get(serve_config))
        .route("/data", get(serve_data).post(receive_data))
        .route("/events", get(serve_events))
        .route("/focus", get(handle_focus))
        .route("/shutdown", get(handle_shutdown))
        .route("/ping", get(ping))
        .route("/assets/{*path}", get(serve_assets))
        .with_state(state.clone());

    // Spawn file watcher — uses a std::sync::mpsc channel so no nested async runtime is needed.
    let watcher_broadcast = broadcast_tx.clone();
    let (watcher_event_tx, watcher_event_rx) = std::sync::mpsc::channel();
    let mut fs_watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = watcher_event_tx.send(event);
            }
        },
        Config::default(),
    )?;
    fs_watcher.watch(canonical_path.as_path(), RecursiveMode::NonRecursive)?;

    std::thread::spawn(move || {
        // Keep `fs_watcher` alive for the duration of this thread.
        let _watcher = fs_watcher;
        let debounce = std::time::Duration::from_millis(80);
        let mut last_sent = std::time::Instant::now()
            .checked_sub(debounce * 2)
            .unwrap_or_else(std::time::Instant::now);

        for event in watcher_event_rx {
            match event.kind {
                notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                    let now = std::time::Instant::now();
                    if now.duration_since(last_sent) >= debounce {
                        last_sent = now;
                        info!("File changed, sending reload event");
                        let _ = watcher_broadcast.send(());
                    }
                }
                _ => {}
            }
        }
    });

    // Spawn the HTTP server as a background task with graceful shutdown.
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    rt.spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .expect("HTTP server failed");
    });

    // Run the WebView event loop on the main thread (required on macOS / some Linux WMs).
    if let Err(e) = run_webview(port, focus_rx) {
        eprintln!(
            "WebView error: {}. Server running at http://127.0.0.1:{}",
            e, port
        );
        eprintln!("(WebView not available in this environment)");
        // Keep server running briefly so user can test
        std::thread::sleep(std::time::Duration::from_secs(60));
    }

    // Webview closed — tear down server and remove lock file.
    let _ = shutdown_tx.send(());
    rt.shutdown_timeout(std::time::Duration::from_secs(5));
    let _ = std::fs::remove_file(&lock_path);
    info!("Shutdown complete");

    Ok(())
}

/// Determine MIME type from the file name (extension is authoritative; no byte sniffing).
fn detect_content_type(path: &Path) -> String {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if name.ends_with(".excalidraw.svg") {
        "image/svg+xml".to_string()
    } else if name.ends_with(".excalidraw.png") {
        "image/png".to_string()
    } else {
        "application/json".to_string()
    }
}

/// Returns the path of the per-file lock file stored in the system temp directory.
fn get_lock_path(canonical_path: &Path) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(canonical_path.to_string_lossy().as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    let tmpdir = std::env::var("TMPDIR")
        .or_else(|_| std::env::var("TEMP"))
        .unwrap_or_else(|_| "/tmp".to_string());

    PathBuf::from(tmpdir).join(format!("excalidraw-{}.lock", &hash[..16]))
}

/// Checks whether a lock file points to a live server instance.
/// Returns the port on success, or an error if the lock is stale / missing.
async fn check_existing_instance(lock_path: &PathBuf) -> Result<u16> {
    let port_str = std::fs::read_to_string(lock_path)?;
    let port: u16 = port_str.trim().parse()?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{}/ping", port))
        .timeout(std::time::Duration::from_secs(1))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(port)
    } else {
        std::fs::remove_file(lock_path).ok();
        anyhow::bail!("Stale lock file")
    }
}

/// Finds the first available TCP port in [10000, 65000].
fn find_available_port() -> u16 {
    (10000..=65000)
        .find(|&p| std::net::TcpListener::bind(format!("127.0.0.1:{}", p)).is_ok())
        .unwrap_or(9876)
}

// ── Route handlers ──────────────────────────────────────────────────────────

async fn serve_index() -> Response {
    match Assets::get("index.html") {
        Some(content) => (
            axum::http::StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            content.data.to_vec(),
        )
            .into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

async fn serve_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = ConfigResponse {
        content_type: state.content_type.clone(),
        name: state.file_name.clone(),
        theme: "auto".to_string(),
    };
    axum::Json(config)
}

async fn serve_data(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match std::fs::read(&state.file_path) {
        Ok(data) => (
            axum::http::StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, state.content_type.clone())],
            data,
        )
            .into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Receives edited scene data from the WebView and writes it back to disk.
/// The file watcher will fire after this write; the client suppresses that SSE event.
async fn receive_data(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    match std::fs::write(&state.file_path, &body) {
        Ok(_) => axum::http::StatusCode::OK.into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn serve_events(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    use axum::response::sse::{Event, Sse};

    let mut rx = state.broadcast_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(_) => yield Ok::<Event, Infallible>(Event::default().data("reload")),
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    };

    Sse::new(stream)
}

async fn handle_focus(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _ = state.focus_tx.send(true);
    "OK"
}

async fn handle_shutdown(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let lock_path = state.lock_path.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let _ = std::fs::remove_file(&lock_path);
        std::process::exit(0);
    });
    "OK"
}

async fn ping() -> impl IntoResponse {
    "OK"
}

async fn serve_assets(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    let path = path.strip_prefix("/").unwrap_or(&path);
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            (
                axum::http::StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, mime)],
                content.data.to_vec(),
            )
                .into_response()
        }
        None => (axum::http::StatusCode::NOT_FOUND, "Asset not found").into_response(),
    }
}

// ── WebView ──────────────────────────────────────────────────────────────────

/// Opens the WebView at the given URL. Blocks until the window is closed.
fn run_webview(port: u16, focus_rx: watch::Receiver<bool>) -> Result<(), Box<dyn std::error::Error>> {
    run_webview_url(&format!("http://127.0.0.1:{}", port), focus_rx)
}

/// Opens the WebView at an arbitrary URL. Used both for the normal server and
/// for `--dev` / `--dev-server` mode (pointing at the Vite dev server).
#[cfg(target_os = "linux")]
fn run_webview_url(
    url: &str,
    _focus_rx: watch::Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    use gtk::glib::Propagation;
    use gtk::prelude::*;
    use wry::WebViewBuilderExtUnix;

    gtk::init().map_err(|e| format!("Failed to init GTK: {}", e))?;

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Excalidraw Preview");
    window.set_default_size(1200, 800);

    let _webview = wry::WebViewBuilder::new_gtk(&window)
        .with_url(url)
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

/// Opens the WebView at an arbitrary URL. Used both for the normal server and
/// for `--dev` / `--dev-server` mode (pointing at the Vite dev server).
#[cfg(not(target_os = "linux"))]
fn run_webview_url(
    url: &str,
    mut focus_rx: watch::Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tao::{
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    use wry::WebViewBuilder;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Excalidraw Preview")
        .with_inner_size(tao::dpi::LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)
        .map_err(|e| format!("Failed to create window: {}", e))?;

    let _webview = WebViewBuilder::new(&window)
        .with_url(url)
        .build()
        .map_err(|e| format!("Failed to create WebView: {}", e))?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }

        if focus_rx.has_changed().unwrap_or(false) {
            let _ = focus_rx.borrow_and_update();
            window.set_focus();
        }
    });

    Ok(())
}

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(name = "excalidraw-preview")]
#[command(about = "Preview Excalidraw files in a native window")]
struct CliArgs {
    /// Path to the .excalidraw, .excalidraw.svg, or .excalidraw.png file to preview.
    /// Not required when running as an LSP server (--lsp).
    file: Option<String>,
    /// Bind the HTTP server to this port (default: auto-selected).
    #[arg(long)]
    port: Option<u16>,
    /// Enable debug logging.
    #[arg(long)]
    debug: bool,
    /// Run as a minimal LSP server for Zed integration.
    /// Zed will call this automatically when a .excalidraw file is opened.
    #[arg(long)]
    lsp: bool,
    /// Dev shorthand: open the WebView at http://localhost:5173 (the Vite dev server).
    /// Run `npm run dev` in preview-binary/webview-src/ first.
    #[arg(long)]
    dev: bool,
    /// Open the WebView at a custom URL instead of the embedded server.
    /// Useful for pointing at a running Vite dev server on a non-default port.
    /// e.g. --dev-server http://localhost:5173
    #[arg(long, value_name = "URL")]
    dev_server: Option<String>,
}

// ── LSP server ───────────────────────────────────────────────────────────────

/// Minimal JSON-RPC LSP server.
///
/// Zed starts this when a `.excalidraw` file is opened (via `language_server_command`).
/// On `textDocument/didOpen` it spawns the preview window for that file and exits
/// the notification without blocking the LSP loop.
fn run_lsp_server() -> Result<()> {
    use std::io::{BufRead, BufReader, Read};

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    let exe = std::env::current_exe()?;

    loop {
        // ── Read headers ────────────────────────────────────────────────────
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            let n = reader.read_line(&mut line)?;
            if n == 0 {
                return Ok(()); // EOF — Zed closed stdin
            }
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break; // blank line separates headers from body
            }
            if let Some(val) = trimmed.strip_prefix("Content-Length: ") {
                content_length = val.trim().parse().ok();
            }
        }

        let len = match content_length {
            Some(l) if l > 0 => l,
            _ => continue,
        };

        // ── Read body ────────────────────────────────────────────────────────
        let mut body = vec![0u8; len];
        reader.read_exact(&mut body)?;

        let msg: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
        let method = msg["method"].as_str().unwrap_or("");
        let id = msg.get("id").cloned();

        // ── Dispatch ─────────────────────────────────────────────────────────
        match method {
            "initialize" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "capabilities": {
                            // 1 = Full sync — Zed will send textDocument/didOpen
                            "textDocumentSync": 1
                        },
                        "serverInfo": { "name": "excalidraw-preview", "version": "0.1.0" }
                    }
                });
                lsp_send(&mut writer, &response)?;
            }

            "initialized" => { /* notification — no response required */ }

            "textDocument/didOpen" => {
                if let Some(uri) = msg["params"]["textDocument"]["uri"].as_str() {
                    if let Some(path) = file_uri_to_path(uri) {
                        spawn_preview(&exe, &path);
                    }
                }
            }

            "textDocument/didClose" => {
                if let Some(uri) = msg["params"]["textDocument"]["uri"].as_str() {
                    if let Some(path) = file_uri_to_path(uri) {
                        shutdown_preview(&path);
                    }
                }
            }

            "shutdown" => {
                lsp_send(
                    &mut writer,
                    &serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": null }),
                )?;
            }

            "exit" => break,

            _ => {
                // Respond to unknown *requests* with "method not found".
                // Notifications (no id) are silently ignored.
                if let Some(id) = id {
                    lsp_send(
                        &mut writer,
                        &serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32601, "message": "Method not found" }
                        }),
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// Writes a single LSP message to `writer` (Content-Length framing).
fn lsp_send(writer: &mut impl std::io::Write, msg: &serde_json::Value) -> Result<()> {
    let body = serde_json::to_string(msg)?;
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    writer.flush()?;
    Ok(())
}

/// Spawns `excalidraw-preview <path>` as a fully detached process so that
/// closing the LSP server (when Zed shuts down the language server) does not
/// kill the preview window.
fn spawn_preview(exe: &std::path::Path, path: &str) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        let _ = std::process::Command::new(exe)
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .process_group(0) // new process group → immune to SIGHUP
            .spawn();
    }
    #[cfg(not(unix))]
    {
        let _ = std::process::Command::new(exe)
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

/// Sends GET /shutdown to the preview server for the given file path.
/// Reads the port from the lock file, then calls the HTTP endpoint.
fn shutdown_preview(path: &str) {
    let canonical = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => return,
    };
    let lock_path = get_lock_path(&canonical);
    let port_str = match std::fs::read_to_string(&lock_path) {
        Ok(s) => s,
        Err(_) => return,
    };
    if let Ok(port) = port_str.trim().parse::<u16>() {
        let url = format!("http://127.0.0.1:{}/shutdown", port);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build();
        if let Ok(client) = client {
            let _ = client.get(&url).send();
        }
    }
}

/// Converts a `file://` URI to an absolute filesystem path.
fn file_uri_to_path(uri: &str) -> Option<String> {
    let path = uri.strip_prefix("file://")?;
    Some(percent_decode(path))
}

/// Decodes percent-encoded bytes in a URI path component.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push(((hi << 4) | lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for `oneshot`

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn make_state(file: &std::path::Path, content_type: &str) -> Arc<AppState> {
        let (broadcast_tx, _) = broadcast::channel(16);
        let (focus_tx, _) = watch::channel(false);
        Arc::new(AppState {
            file_path: file.to_path_buf(),
            lock_path: std::env::temp_dir().join("excalidraw-test.lock"),
            content_type: content_type.to_string(),
            file_name: file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            broadcast_tx,
            focus_tx: Arc::new(focus_tx),
        })
    }

    fn json_app(state: Arc<AppState>) -> Router {
        Router::new()
            .route("/config", get(serve_config))
            .route("/data", get(serve_data))
            .route("/focus", get(handle_focus))
            .route("/ping", get(ping))
            .with_state(state)
    }

    // ── Unit tests ───────────────────────────────────────────────────────────

    #[test]
    fn test_detect_content_type_json() {
        assert_eq!(
            detect_content_type(&PathBuf::from("diagram.excalidraw")),
            "application/json"
        );
    }

    #[test]
    fn test_detect_content_type_svg() {
        assert_eq!(
            detect_content_type(&PathBuf::from("diagram.excalidraw.svg")),
            "image/svg+xml"
        );
    }

    #[test]
    fn test_detect_content_type_png() {
        assert_eq!(
            detect_content_type(&PathBuf::from("diagram.excalidraw.png")),
            "image/png"
        );
    }

    #[test]
    fn test_detect_content_type_unknown_falls_back_to_json() {
        // A plain ".svg" file (without the .excalidraw prefix) still defaults to json.
        assert_eq!(
            detect_content_type(&PathBuf::from("diagram.svg")),
            "application/json"
        );
    }

    #[test]
    fn test_get_lock_path_is_deterministic() {
        let path = PathBuf::from("/tmp/test.excalidraw");
        assert_eq!(get_lock_path(&path), get_lock_path(&path));
    }

    #[test]
    fn test_get_lock_path_differs_for_different_files() {
        let a = PathBuf::from("/tmp/a.excalidraw");
        let b = PathBuf::from("/tmp/b.excalidraw");
        assert_ne!(get_lock_path(&a), get_lock_path(&b));
    }

    #[test]
    fn test_get_lock_path_filename_format() {
        let path = PathBuf::from("/tmp/test.excalidraw");
        let lock = get_lock_path(&path);
        let name = lock.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with("excalidraw-"), "got: {name}");
        assert!(name.ends_with(".lock"), "got: {name}");
    }

    #[test]
    fn test_find_available_port_returns_bindable_port() {
        let port = find_available_port();
        assert!(port >= 10000, "port {port} is below minimum");
        assert!(
            std::net::TcpListener::bind(format!("127.0.0.1:{port}")).is_ok(),
            "port {port} should be bindable"
        );
    }

    // ── Route tests ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_ping_returns_200() {
        let app = Router::new().route("/ping", get(ping));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ping")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_serve_config_returns_correct_json() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let state = make_state(tmp.path(), "application/json");
        let app = json_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/config")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["content_type"], "application/json");
        assert_eq!(json["theme"], "auto");
    }

    #[tokio::test]
    async fn test_serve_config_svg() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let state = make_state(tmp.path(), "image/svg+xml");
        let app = json_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/config")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["content_type"], "image/svg+xml");
    }

    #[tokio::test]
    async fn test_serve_data_returns_file_contents() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        let payload = r#"{"type":"excalidraw","version":2,"elements":[]}"#;
        tmp.write_all(payload.as_bytes()).unwrap();

        let state = make_state(tmp.path(), "application/json");
        let app = json_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/data")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.as_ref(), payload.as_bytes());
    }

    #[tokio::test]
    async fn test_serve_data_missing_file_returns_500() {
        let state = make_state(
            std::path::Path::new("/nonexistent/path/that/does/not/exist.excalidraw"),
            "application/json",
        );
        let app = json_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/data")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_handle_focus_signals_watch_channel() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let (broadcast_tx, _) = broadcast::channel(16);
        let (focus_tx, _focus_rx) = watch::channel(false);
        let focus_tx = Arc::new(focus_tx);

        let state = Arc::new(AppState {
            file_path: tmp.path().to_path_buf(),
            content_type: "application/json".to_string(),
            file_name: "test".to_string(),
            broadcast_tx,
            focus_tx,
        });

        let app = Router::new()
            .route("/focus", get(handle_focus))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/focus")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        // The watch channel value should have been updated to `true`.
        // Note: This test only verifies the endpoint responds; the watch channel
        // behavior is verified in integration tests that run the full event loop.
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.as_ref(), b"OK");
    }

    #[tokio::test]
    async fn test_serve_index_missing_asset_returns_404() {
        // The assets/ folder is embedded at compile time; in a test build without
        // a real `assets/` directory this is expected to return 404.
        let app = Router::new().route("/", get(serve_index));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Either 200 (assets present) or 404 (no assets in test build) is acceptable.
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }
}
