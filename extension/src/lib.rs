use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;
use zed_extension_api::{
    http_client::{self, HttpMethod, HttpRequestBuilder},
    process::Command as ProcessCommand,
    Command, Extension, LanguageServerId, Range, Result, SlashCommand, SlashCommandOutput,
    SlashCommandOutputSection, Worktree,
};

struct ExcalidrawPreviewExtension {
    process_map: RwLock<HashMap<PathBuf, ProcessInfo>>,
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
        let root = worktree.root_path();
        let root_path = PathBuf::from(&root);

        if let Ok(entries) = std::fs::read_dir(&root_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy().to_string();
                        if name_str.ends_with(".excalidraw")
                            || name_str.ends_with(".excalidraw.svg")
                            || name_str.ends_with(".excalidraw.png")
                        {
                            return Some(path);
                        }
                    }
                }
            }
        }
        None
    }
}

impl Extension for ExcalidrawPreviewExtension {
    fn new() -> Self {
        ExcalidrawPreviewExtension {
            process_map: RwLock::new(HashMap::new()),
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        if language_server_id.as_ref() == "excalidraw-preview" {
            let binary = worktree
                .which("excalidraw-preview")
                .ok_or("excalidraw-preview not found in PATH. Run: cargo install --path preview-binary")?;
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
        eprintln!("[DEBUG] run_slash_command called with args: {:?}", args);
        match command.name.as_str() {
            "preview-excalidraw" => {
                let worktree = worktree.ok_or("No worktree available")?;

                let file_path = if let Some(file_arg) = args.first() {
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

                eprintln!("[DEBUG] Using file path: {:?}", file_path);

                if !Self::is_valid_extension(&file_path) {
                    return Err(
                        "File must end with .excalidraw, .excalidraw.svg, or .excalidraw.png"
                            .into(),
                    );
                }

                let file_path_str = file_path.to_string_lossy().to_string();

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

                let port = Self::port_for_path(&file_path);

                // zed_extension_api::process::Command only has `output()` (blocking).
                // We background the binary via the shell so `output()` returns immediately.
                let shell_cmd = format!(
                    "nohup excalidraw-preview {} --port {} >/dev/null 2>&1 &",
                    shlex_quote(&file_path_str),
                    port
                );
                let mut cmd = ProcessCommand::new("sh").arg("-c").arg(&shell_cmd);

                match cmd.output() {
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
                    Err(e) => Err(format!("Failed to start preview binary: {}. Make sure 'excalidraw-preview' is installed and in PATH.", e).into()),
                }
            }
            _ => Err(format!("Unknown command: {}", command.name).into()),
        }
    }
}

zed_extension_api::register_extension!(ExcalidrawPreviewExtension);

/// Quotes a path for safe shell interpolation (single-quote wrapping).
fn shlex_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────
// These test only the pure helper functions that don't touch the Zed extension
// API — the WASM runtime is not available in unit test builds.

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
        // Not strictly guaranteed for all inputs, but should hold for typical paths.
        let a = ExcalidrawPreviewExtension::port_for_path(&PathBuf::from("/tmp/a.excalidraw"));
        let b = ExcalidrawPreviewExtension::port_for_path(&PathBuf::from("/tmp/b.excalidraw"));
        assert_ne!(a, b);
    }
}
