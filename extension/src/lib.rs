use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;
use zed_extension_api::{
    self as zed,
    http_client::{self, HttpMethod, HttpRequestBuilder},
    process::Command as ProcessCommand,
    Architecture, Command, Extension, LanguageServerId, Os, Range, Result, SlashCommand,
    SlashCommandOutput, SlashCommandOutputSection, Worktree,
};

/// Must match the GitHub Release tag (v{VERSION}).
const BINARY_VERSION: &str = "0.1.0";
const BINARY_NAME: &str = "excalidraw-preview";

struct ExcalidrawPreviewExtension {
    process_map: RwLock<HashMap<PathBuf, ProcessInfo>>,
    /// Cached path to the downloaded binary, so we don't re-download on every call.
    cached_binary_path: RwLock<Option<String>>,
}

struct ProcessInfo {
    #[allow(dead_code)]
    pid: u32,
    port: u16,
}

impl ExcalidrawPreviewExtension {
    fn is_valid_extension(path: &PathBuf) -> bool {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        name.ends_with(".excalidraw")
            || name.ends_with(".excalidraw.svg")
            || name.ends_with(".excalidraw.png")
    }

    fn port_for_path(path: &PathBuf) -> u16 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        path.hash(&mut h);
        (20000 + h.finish() % 10000) as u16
    }

    fn send_focus_request(port: u16) -> bool {
        let url = format!("http://127.0.0.1:{}/focus", port);
        match HttpRequestBuilder::new()
            .method(HttpMethod::Get)
            .url(&url)
            .build()
        {
            Ok(request) => http_client::fetch(&request).is_ok(),
            Err(_) => false,
        }
    }

    fn find_excalidraw_file(worktree: &Worktree) -> Option<PathBuf> {
        let root_path = PathBuf::from(worktree.root_path());
        if let Ok(entries) = std::fs::read_dir(&root_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && Self::is_valid_extension(&path) {
                    return Some(path);
                }
            }
        }
        None
    }

    /// Returns the path to the `excalidraw-preview` binary.
    ///
    /// Resolution order:
    /// 1. `PATH` — honours `make symlink` for local dev.
    /// 2. Previously cached download from this session.
    /// 3. Fresh download from GitHub Releases → cached for the rest of the session.
    fn get_binary_path(&self, worktree: &Worktree) -> Result<String> {
        // 1. Prefer whatever is on PATH (dev / make symlink workflow).
        if let Some(path) = worktree.which(BINARY_NAME) {
            return Ok(path);
        }

        // 2. Return cached download if the file is still there.
        {
            let cache = self.cached_binary_path.read().unwrap();
            if let Some(ref path) = *cache {
                if std::fs::metadata(path).is_ok() {
                    return Ok(path.clone());
                }
            }
        }

        // 3. Download from GitHub Releases.
        let binary_path = self.download_binary()?;
        *self.cached_binary_path.write().unwrap() = Some(binary_path.clone());
        Ok(binary_path)
    }

    /// Downloads the platform-specific binary from GitHub Releases and makes it executable.
    ///
    /// Release asset naming convention:
    ///   `excalidraw-preview-{arch}-{os}`
    /// e.g. `excalidraw-preview-x86_64-unknown-linux-gnu`
    ///      `excalidraw-preview-aarch64-apple-darwin`
    ///      `excalidraw-preview-x86_64-pc-windows-msvc.exe`
    fn download_binary(&self) -> Result<String> {
        let (platform, arch) = zed::current_platform();

        let arch_str = match arch {
            Architecture::Aarch64 => "aarch64",
            Architecture::X8664 => "x86_64",
            Architecture::X86 => "x86",
        };

        let os_str = match platform {
            Os::Mac => "apple-darwin",
            Os::Linux => "unknown-linux-gnu",
            Os::Windows => "pc-windows-msvc",
        };

        let ext = match platform {
            Os::Windows => ".exe",
            _ => "",
        };

        let asset_name = format!("{BINARY_NAME}-{arch_str}-{os_str}{ext}");

        let download_url = format!(
            "https://github.com/arindampradhan/excalidraw-zed-extension/releases/download/v{BINARY_VERSION}/{asset_name}"
        );

        // Zed stores downloaded files under the extension's own work directory.
        let output_path = format!("{BINARY_NAME}-{BINARY_VERSION}/{asset_name}");

        zed::download_file(
            &download_url,
            &output_path,
            zed::DownloadedFileType::Uncompressed,
        )
        .map_err(|e| format!("Failed to download {BINARY_NAME}: {e}"))?;

        zed::make_file_executable(&output_path)
            .map_err(|e| format!("Failed to make {BINARY_NAME} executable: {e}"))?;

        Ok(output_path)
    }
}

impl Extension for ExcalidrawPreviewExtension {
    fn new() -> Self {
        ExcalidrawPreviewExtension {
            process_map: RwLock::new(HashMap::new()),
            cached_binary_path: RwLock::new(None),
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        if language_server_id.as_ref() == "excalidraw-preview" {
            let binary = self.get_binary_path(worktree)?;
            Ok(Command {
                command: binary,
                args: vec!["--lsp".into()],
                env: Default::default(),
            })
        } else {
            Err(format!("unknown language server: {language_server_id}").into())
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput> {
        match command.name.as_str() {
            "preview-excalidraw" => {
                let worktree = worktree.ok_or("No worktree available")?;

                let binary = self.get_binary_path(worktree)?;

                // Separate flags (--auto-save) from the positional file argument.
                let auto_save = args.contains(&"--auto-save".to_string());
                let file_arg = args.iter().find(|a| !a.starts_with("--"));

                let file_path = if let Some(file_arg) = file_arg {
                    let p = PathBuf::from(file_arg);
                    if p.is_absolute() {
                        p
                    } else {
                        PathBuf::from(worktree.root_path()).join(&p)
                    }
                } else {
                    Self::find_excalidraw_file(worktree).ok_or(
                        "No .excalidraw file found in workspace. Provide a file path as argument.",
                    )?
                };

                if !Self::is_valid_extension(&file_path) {
                    return Err(
                        "File must end with .excalidraw, .excalidraw.svg, or .excalidraw.png"
                            .into(),
                    );
                }

                let file_path_str = file_path.to_string_lossy().to_string();
                let port = Self::port_for_path(&file_path);

                // If a window is already open for this file, focus it and return early.
                {
                    let map = self.process_map.read().unwrap();
                    if let Some(info) = map.get(&file_path) {
                        if Self::send_focus_request(info.port) {
                            return Ok(SlashCommandOutput {
                                sections: vec![SlashCommandOutputSection {
                                    range: Range { start: 0, end: 1 },
                                    label: "Preview focused".into(),
                                }],
                                text: "Existing preview window focused".into(),
                            });
                        }
                    }
                }

                self.process_map.write().unwrap().remove(&file_path);

                // Background the binary via the shell so output() returns immediately.
                let auto_save_flag = if auto_save { " --auto-save" } else { "" };
                let shell_cmd = format!(
                    "nohup {} {} --port {}{} >/dev/null 2>&1 &",
                    shlex_quote(&binary),
                    shlex_quote(&file_path_str),
                    port,
                    auto_save_flag,
                );

                match ProcessCommand::new("sh").arg("-c").arg(&shell_cmd).output() {
                    Ok(_) => {
                        self.process_map
                            .write()
                            .unwrap()
                            .insert(file_path.clone(), ProcessInfo { pid: 0, port });
                        Ok(SlashCommandOutput {
                            sections: vec![SlashCommandOutputSection {
                                range: Range { start: 0, end: 1 },
                                label: "Preview opened".into(),
                            }],
                            text: format!("Opened preview for {}", file_path.display()),
                        })
                    }
                    Err(e) => Err(format!("Failed to start preview: {e}").into()),
                }
            }
            _ => Err(format!("Unknown command: {}", command.name).into()),
        }
    }
}

zed_extension_api::register_extension!(ExcalidrawPreviewExtension);

/// Wraps a string in single quotes, escaping any existing single quotes.
fn shlex_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_extension_json() {
        assert!(ExcalidrawPreviewExtension::is_valid_extension(
            &PathBuf::from("diagram.excalidraw")
        ));
    }

    #[test]
    fn test_is_valid_extension_svg() {
        assert!(ExcalidrawPreviewExtension::is_valid_extension(
            &PathBuf::from("diagram.excalidraw.svg")
        ));
    }

    #[test]
    fn test_is_valid_extension_png() {
        assert!(ExcalidrawPreviewExtension::is_valid_extension(
            &PathBuf::from("diagram.excalidraw.png")
        ));
    }

    #[test]
    fn test_is_valid_extension_rejects_plain_svg() {
        assert!(!ExcalidrawPreviewExtension::is_valid_extension(
            &PathBuf::from("diagram.svg")
        ));
    }

    #[test]
    fn test_is_valid_extension_rejects_plain_json() {
        assert!(!ExcalidrawPreviewExtension::is_valid_extension(
            &PathBuf::from("diagram.json")
        ));
    }

    #[test]
    fn test_is_valid_extension_rejects_empty() {
        assert!(!ExcalidrawPreviewExtension::is_valid_extension(
            &PathBuf::from("")
        ));
    }

    #[test]
    fn test_is_valid_extension_with_absolute_path() {
        assert!(ExcalidrawPreviewExtension::is_valid_extension(
            &PathBuf::from("/home/user/diagrams/arch.excalidraw")
        ));
    }

    #[test]
    fn test_port_for_path_is_deterministic() {
        let p = PathBuf::from("/tmp/test.excalidraw");
        assert_eq!(
            ExcalidrawPreviewExtension::port_for_path(&p),
            ExcalidrawPreviewExtension::port_for_path(&p)
        );
    }

    #[test]
    fn test_port_for_path_is_in_valid_range() {
        let p = PathBuf::from("/tmp/test.excalidraw");
        let port = ExcalidrawPreviewExtension::port_for_path(&p);
        assert!(port >= 20000 && port < 30000, "port {port} out of range");
    }

    #[test]
    fn test_port_for_path_differs_for_different_files() {
        let a = ExcalidrawPreviewExtension::port_for_path(&PathBuf::from("/tmp/a.excalidraw"));
        let b = ExcalidrawPreviewExtension::port_for_path(&PathBuf::from("/tmp/b.excalidraw"));
        assert_ne!(a, b);
    }

    #[test]
    fn test_shlex_quote_plain() {
        assert_eq!(shlex_quote("/usr/bin/foo"), "'/usr/bin/foo'");
    }

    #[test]
    fn test_shlex_quote_with_single_quote() {
        assert_eq!(shlex_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn test_shlex_quote_with_spaces() {
        assert_eq!(shlex_quote("/path/to my/file"), "'/path/to my/file'");
    }
}
