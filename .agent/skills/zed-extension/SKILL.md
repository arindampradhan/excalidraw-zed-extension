---
name: zed-extension
description: 'Scaffold and implement Zed editor extensions. Invoke with the type of extension and what it should do. Generates the full extension structure: extension.toml, Cargo.toml, src/lib.rs, and any supporting files for languages, themes, MCP servers, debuggers, agent servers, or slash commands.'
---

# Zed Extension Skill

Scaffold and implement Zed editor extensions (Rust compiled to WASM). Covers all extension types: slash commands, language support (with Tree-sitter queries), themes, icon themes, snippets, MCP servers, debugger extensions, and agent server extensions.

## When to Use This Skill

- `/zed-extension Create a slash command that formats JSON`
- `/zed-extension Add language support for .xyz files with syntax highlighting`
- `/zed-extension Create a theme extension`
- `/zed-extension Create an MCP server extension for GitHub`
- `/zed-extension Add a debugger extension for my language`
- `/zed-extension Scaffold a new extension called my-tool`

---

## Full Workflow

### STEP 0 — Parse the Request

Extract:
- **Extension ID**: kebab-case, no `zed` or `extension` in the name (Zed policy — hard reject)
- **Extension type(s)**: slash command, language, theme, MCP server, debugger, snippets, icon theme, agent server
- **Output directory**: default to `./` (current repo root) unless user specifies

**ID naming rules (strictly enforced by Zed registry):**

| Type | Convention | Example |
|---|---|---|
| Language / tool | bare name | `gleam`, `biome` |
| Theme | `-theme` suffix | `catppuccin-theme` |
| Snippets | `-snippets` suffix | `rust-snippets` |
| MCP server | `-server` suffix | `github-server` |
| Icon theme | `-icons` suffix | `material-icons` |
| Must NOT contain | `zed`, `Zed`, `extension` | (rejected by CI) |

---

### STEP 1 — Decide What Files Are Needed

| Capability | Files Required |
|---|---|
| Slash command / custom logic | `extension.toml`, `Cargo.toml`, `src/lib.rs` |
| Language support | `languages/<name>/config.toml` + query files |
| Tree-sitter grammar | `extension.toml` `[grammars]` section |
| Language server | `extension.toml` `[language_servers]` section + Rust |
| Debugger (DAP) | `extension.toml` `[debug_adapters]` + schema JSON + Rust |
| Debug locator | `extension.toml` `[debug_locators]` + Rust |
| Theme | `themes/<name>.json` |
| Icon theme | `icon_themes/<name>.json` |
| Snippets | `snippets/snippets.json` or `snippets/<lang>.json` |
| MCP server | `extension.toml` `[context_servers]` + Rust |
| Agent server | `extension.toml` `[agent_servers]` (no Rust needed) |

Only generate files actually needed for what was requested.

---

### STEP 2 — Generate `extension.toml`

Always required. Base template:

```toml
id = "<extension-id>"
name = "<Human Readable Name>"
version = "0.1.0"
schema_version = 1
authors = ["<Name> <email@example.com>"]
description = "<one-line description>"
repository = "https://github.com/<user>/<repo>"
```

**Add sections based on capabilities needed:**

#### Slash commands

```toml
[slash_commands.my-command]
description = "What the command does"
requires_argument = false  # set true if it takes args
```

#### Language + Grammar (Tree-sitter)

```toml
[grammars.my-language]
repository = "https://github.com/owner/tree-sitter-my-language"
rev = "abc123def456..."   # exact git SHA

[language_servers.my-lsp]
name = "My Language LSP"
languages = ["My Language"]

# Optional: map Zed language names to LSP languageId values
[language_servers.my-lsp.language_ids]
"My Language" = "mylang"
```

#### Debugger

```toml
[debug_adapters.my-debug-adapter]
schema_path = "debug_adapter_schemas/my-debug-adapter.json"

[debug_locators.my-debug-locator]
```

#### MCP Server

```toml
[context_servers.my-context-server]
```

#### Agent Server (deprecated since v0.221.x — prefer ACP Registry)

```toml
[agent_servers.my-agent]
name = "My Agent"
icon = "icon/agent.svg"  # optional SVG, 16x16

[agent_servers.my-agent.env]
MY_ENV_VAR = "value"

[agent_servers.my-agent.targets.darwin-aarch64]
archive = "https://github.com/owner/repo/releases/download/v1.0.0/agent-darwin-arm64.tar.gz"
cmd = "./agent"
args = ["--serve"]
sha256 = "<hash>"  # recommended

[agent_servers.my-agent.targets.linux-x86_64]
archive = "https://github.com/owner/repo/releases/download/v1.0.0/agent-linux-x64.tar.gz"
cmd = "./agent"
args = ["--serve"]

[agent_servers.my-agent.targets.windows-x86_64]
archive = "https://github.com/owner/repo/releases/download/v1.0.0/agent-windows-x64.zip"
cmd = "./agent.exe"
args = ["--serve"]
```

#### Snippets

```toml
snippets = ["./snippets/snippets.json", "./snippets/rust.json"]
```

---

### STEP 3 — Generate `Cargo.toml` (only if Rust code needed)

```toml
[package]
name = "<extension-id>"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
zed_extension_api = "0.1.0"   # check crates.io for latest
```

Add only if needed:
- `serde` / `serde_json` — JSON parsing
- Do NOT add `tokio`, `reqwest`, or threading crates — unavailable in WASM

**Hard WASM constraints (never violate):**
- No raw sockets or TCP
- No `std::thread`
- No `std::process::Command` — use `zed_extension_api::process::Command`
- No direct file I/O — use worktree/project APIs
- HTTP requests via `zed::http_client_get()` only

---

### STEP 4 — Generate `src/lib.rs`

#### Minimal slash command

```rust
use zed_extension_api::{self as zed, SlashCommand, SlashCommandOutput, SlashCommandOutputSection, Result};

struct MyExtension;

impl zed::Extension for MyExtension {
    fn new() -> Self { MyExtension }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<SlashCommandOutput> {
        match command.name.as_str() {
            "my-command" => {
                let text = "output here".to_string();
                Ok(SlashCommandOutput {
                    sections: vec![SlashCommandOutputSection {
                        range: 0..text.len(),
                        icon: zed::SlashCommandOutputSectionIcon::Doc,
                        label: "Result".into(),
                    }],
                    text,
                    run_commands_in_text: false,
                })
            }
            _ => Err(format!("unknown command: {}", command.name).into()),
        }
    }
}

zed::register_extension!(MyExtension);
```

#### Language server extension

```rust
use zed_extension_api::{self as zed, LanguageServerId, Result};

struct MyLangExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for MyLangExtension {
    fn new() -> Self {
        MyLangExtension { cached_binary_path: None }
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        if self.cached_binary_path.is_none() {
            let (os, arch) = zed::current_platform();
            let version = "1.2.3";
            let url = format!(
                "https://github.com/owner/repo/releases/download/v{version}/lsp-{os}-{arch}.tar.gz"
            );
            let path = format!("{}/lsp-{version}", zed::extension_dir());
            zed::download_file(&url, &path, zed::DownloadedFileType::GzipTar)?;
            zed::make_file_executable(&path)?;
            self.cached_binary_path = Some(path);
        }
        Ok(zed::Command {
            command: self.cached_binary_path.clone().unwrap(),
            args: vec!["--stdio".into()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(MyLangExtension);
```

#### MCP server extension

```rust
use zed_extension_api::{self as zed, ContextServerId, Result};

struct MyMcpExtension;

impl zed::Extension for MyMcpExtension {
    fn new() -> Self { MyMcpExtension }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &zed::Project,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: get_path_to_server()?,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(MyMcpExtension);
```

#### Debugger extension

```rust
use zed_extension_api::{self as zed, DebugAdapterBinary, DebugTaskDefinition, Result, Worktree};

struct MyDebugExtension;

impl zed::Extension for MyDebugExtension {
    fn new() -> Self { MyDebugExtension }

    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        _config: DebugTaskDefinition,
        user_provided_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        // Download or locate the debug adapter binary
        let binary_path = user_provided_path
            .or_else(|| worktree.which("my-debugger"))
            .ok_or("my-debugger not found")?;
        Ok(DebugAdapterBinary {
            command: binary_path,
            args: vec!["--interpreter=dap".into()],
            env: Default::default(),
        })
    }

    fn dap_request_kind(
        &mut self,
        _adapter_name: String,
        _config: serde_json::Value,
    ) -> Result<zed::StartDebuggingRequestArgumentsRequest, String> {
        Ok(zed::StartDebuggingRequestArgumentsRequest::Launch)
    }
}

zed::register_extension!(MyDebugExtension);
```

#### Key `zed_extension_api` patterns

| Task | API |
|---|---|
| Find binary in PATH | `worktree.which("binary-name")` |
| Shell environment | `worktree.shell_env()` |
| Read text file | `worktree.read_text_file(path)` |
| HTTP GET | `zed::http_client_get(url, headers)` |
| npm install | `zed::npm_install_package(package, version)` |
| Node binary path | `zed::node_binary_path()` |
| Current platform | `zed::current_platform()` → `(Os, Architecture)` |
| Download file | `zed::download_file(url, path, FileType::GzipTar)` |
| Make executable | `zed::make_file_executable(path)` |
| Extension work dir | `zed::extension_dir()` |

---

### STEP 5 — Language Support Files

#### `languages/<name>/config.toml`

```toml
name = "My Language"
grammar = "my-language"          # matches [grammars.my-language] in extension.toml
path_suffixes = ["xyz", "myext"]
line_comments = ["// "]
block_comment = ["/* ", " */"]
tab_size = 2
hard_tabs = false
# first_line_pattern = "^#!"   # optional: shebang detection

[brackets]
[[brackets.pair]]
start = "{"
end = "}"
close = true
newline = true

[[brackets.pair]]
start = "["
end = "]"
close = true
newline = false

[[brackets.pair]]
start = "("
end = ")"
close = true
newline = false

# Override settings per syntax scope:
# [overrides.string]
# completion_query_characters = ["-"]
```

#### Tree-sitter Query Files

All query files go in `languages/<name>/`. Generate only what's needed:

**`highlights.scm`** — Syntax highlighting. Full list of valid capture names:

```scheme
; Core captures (most common)
(identifier) @variable
(string) @string
(number) @number
(comment) @comment
(comment) @comment.doc   ; for doc comments

; Keywords / operators
["if" "else" "while" "for" "return" "fn"] @keyword
["+" "-" "*" "/" "="] @operator

; Functions
(function_declaration name: (identifier) @function)
(call_expression function: (identifier) @function.call)

; Types
(type_identifier) @type
(type_identifier) @type.builtin

; Other valid captures:
; @attribute @boolean @constant @constant.builtin @constructor
; @embedded @emphasis @emphasis.strong @enum
; @hint @label @link_text @link_uri
; @predictive @preproc @primary @property
; @punctuation @punctuation.bracket @punctuation.delimiter
; @punctuation.list_marker @punctuation.special
; @string.escape @string.regex @string.special @string.special.symbol
; @tag @tag.doctype @text.literal @title
; @variable.member @variable.parameter @variable.special @variant

; Fallback captures (right-to-left resolution):
; (type_identifier) @type @variable  ← tries @variable first, falls back to @type
```

**`brackets.scm`** — Bracket matching:

```scheme
("[" @open "]" @close)
("{" @open "}" @close)
("(" @open ")" @close)
("\"" @open "\"" @close)
; Opt out of rainbow brackets for a pair:
; (("\"" @open "\"" @close) (#set! rainbow.exclude))
```

**`outline.scm`** — Code outline / structure panel:

```scheme
(function_declaration
  name: (identifier) @name) @item

(class_declaration
  name: (identifier) @name) @item

; @context — adds context prefix in outline
; @context.extra — additional context
; @annotation — doc comments attached to items (used by AI assistant)
```

**`indents.scm`** — Auto-indentation:

```scheme
(block "}" @end) @indent
(array "]" @end) @indent
(object "}" @end) @indent
```

**`injections.scm`** — Embedded languages:

```scheme
(fenced_code_block
  (info_string (language) @injection.language)
  (code_fence_content) @injection.content)

((template_string) @injection.content
 (#set! injection.language "javascript"))
```

**`overrides.scm`** — Syntax scope overrides:

```scheme
(string) @string
(comment) @comment.inclusive   ; .inclusive = range includes delimiter chars
```

**`textobjects.scm`** — Vim mode text objects (added in v0.165):

```scheme
(function_declaration
  body: (_
    "{" (_)* @function.inside "}")) @function.around

(class_declaration
  body: (_
    "{" (_)* @class.inside "}")) @class.around

(comment)+ @comment.around
```

**`redactions.scm`** — Screen-share redaction:

```scheme
(pair value: (string) @redact)
(pair value: (number) @redact)
```

**`runnables.scm`** — Run button detection:

```scheme
(
  (document
    (object
      (pair
        key: (string (string_content) @_name (#eq? @_name "scripts"))
        value: (object
          (pair key: (string (string_content) @run @script))
        )
      )
    )
  )
  (#set! tag package-script)
)
; @run = where run button appears; other non-_ captures → ZED_CUSTOM_<name> env vars
```

**`semantic_token_rules.json`** — Custom LSP semantic token mapping:

```json
[
  { "token_type": "lifetime", "style": ["lifetime"] },
  { "token_type": "builtinType", "style": ["type"] },
  { "token_type": "selfKeyword", "style": ["variable.special"] }
]
```

---

### STEP 6 — Theme File

`themes/<name>.json` — must match `https://zed.dev/schema/themes/v0.2.0.json`:

```json
{
  "$schema": "https://zed.dev/schema/themes/v0.2.0.json",
  "name": "My Theme Family",
  "author": "Your Name",
  "themes": [
    {
      "name": "My Theme Dark",
      "appearance": "dark",
      "style": {
        "background": "#1e1e2e",
        "foreground": "#cdd6f4",
        "border": "#313244",
        "border.variant": "#45475a",
        "border.focused": "#89b4fa",
        "border.selected": "#89b4fa",
        "border.transparent": "#00000000",
        "border.disabled": "#313244",
        "elevated_surface.background": "#181825",
        "surface.background": "#313244",
        "element.background": "#313244",
        "element.hover": "#45475a",
        "element.active": "#585b70",
        "element.selected": "#45475a",
        "element.disabled": "#313244",
        "text": "#cdd6f4",
        "text.muted": "#a6adc8",
        "text.placeholder": "#6c7086",
        "text.disabled": "#6c7086",
        "text.accent": "#89b4fa",
        "editor.background": "#1e1e2e",
        "editor.gutter.background": "#1e1e2e",
        "editor.active_line.background": "#31324440",
        "editor.line_number": "#585b70",
        "editor.active_line_number": "#cba6f7",
        "editor.foreground": "#cdd6f4",
        "tab_bar.background": "#181825",
        "tab.inactive_background": "#181825",
        "tab.active_background": "#1e1e2e",
        "toolbar.background": "#1e1e2e",
        "status_bar.background": "#181825",
        "title_bar.background": "#181825",
        "terminal.background": "#1e1e2e",
        "terminal.foreground": "#cdd6f4",
        "terminal.ansi.black": "#45475a",
        "terminal.ansi.red": "#f38ba8",
        "terminal.ansi.green": "#a6e3a1",
        "terminal.ansi.yellow": "#f9e2af",
        "terminal.ansi.blue": "#89b4fa",
        "terminal.ansi.magenta": "#f5c2e7",
        "terminal.ansi.cyan": "#94e2d5",
        "terminal.ansi.white": "#bac2de",
        "syntax": {
          "keyword":   { "color": "#cba6f7", "font_style": null, "font_weight": null },
          "string":    { "color": "#a6e3a1", "font_style": null, "font_weight": null },
          "comment":   { "color": "#6c7086", "font_style": "italic", "font_weight": null },
          "function":  { "color": "#89b4fa", "font_style": null, "font_weight": null },
          "type":      { "color": "#f9e2af", "font_style": null, "font_weight": null },
          "number":    { "color": "#fab387", "font_style": null, "font_weight": null },
          "operator":  { "color": "#89dceb", "font_style": null, "font_weight": null },
          "variable":  { "color": "#cdd6f4", "font_style": null, "font_weight": null },
          "property":  { "color": "#89dceb", "font_style": null, "font_weight": null },
          "constant":  { "color": "#fab387", "font_style": null, "font_weight": null }
        }
      }
    }
  ]
}
```

Use Zed's Theme Builder at zed.dev to preview and fine-tune colors before writing the JSON.

---

### STEP 7 — Snippets File

`snippets/snippets.json` (all languages) or `snippets/<lowercase-language>.json`:

```json
[
  {
    "prefix": "fn",
    "body": "fn ${1:name}(${2:args}) -> ${3:ReturnType} {\n\t${4:todo!()}\n}",
    "description": "Function definition"
  },
  {
    "prefix": "struct",
    "body": "struct ${1:Name} {\n\t${2:field}: ${3:Type},\n}",
    "description": "Struct definition"
  }
]
```

Register in `extension.toml`:
```toml
snippets = ["./snippets/snippets.json", "./snippets/rust.json"]
```

---

## Extension Capabilities (User Settings)

Users can restrict what extensions can do via `granted_extension_capabilities` in settings:

```json
{
  "granted_extension_capabilities": [
    { "kind": "process:exec", "command": "*", "args": ["**"] },
    { "kind": "download_file", "host": "github.com", "path": ["**"] },
    { "kind": "npm:install", "package": "*" }
  ]
}
```

When writing extensions, declare only the capabilities you need and document them.

---

## Output Format

When this skill runs, output in this order:

1. `## Extension Plan` — what files will be created and why
2. Generate each file with the Write tool
3. `## Build & Install` — exact commands
4. `## Next Steps` — manual steps (e.g., find tree-sitter grammar SHA, sign binary for macOS)

---

## Build & Install

```bash
# Install wasm32-wasip1 target (one-time setup, requires rustup)
rustup target add wasm32-wasip1

# Build WASM extension
cargo build --release --target wasm32-wasip1

# Install as dev extension in Zed (Zed compiles it automatically)
zed --install-dev-extension .

# View extension logs
zed --foreground     # stdout/stderr of extension visible here
# Or: Cmd/Ctrl+Shift+P → "zed: open log"
```

**Requirements:** Rust must be installed via `rustup`, not Homebrew. If already installed via Homebrew, dev extensions will not compile.

**Extension install paths:**
- macOS: `~/Library/Application Support/Zed/extensions/`
- Linux: `$XDG_DATA_HOME/zed/extensions/` or `~/.local/share/zed/extensions/`
- Windows: `%LOCALAPPDATA%\Zed\extensions\`

---

## Publishing Checklist

- [ ] Extension ID has no `zed`, `Zed`, or `extension`
- [ ] `repository` field set in `extension.toml`
- [ ] License file at repo root (MIT, Apache-2.0, GPL-3.0, BSD-2-Clause, BSD-3-Clause, CC-BY-4.0, LGPL-3.0, Unlicense, or zlib) — **required since Oct 1, 2025**
- [ ] Version in `extension.toml` matches `Cargo.toml`
- [ ] Language servers / debug adapters downloaded at runtime, NOT bundled
- [ ] Themes don't mix with language/tool extensions (keep as separate repo)
- [ ] Fork `zed-industries/extensions` (to personal account, not org)
- [ ] `git submodule add <HTTPS-url> extensions/<id>`
- [ ] Add entry to `extensions.toml`
- [ ] Run `pnpm sort-extensions`
- [ ] Open PR to `zed-industries/extensions`

**Updating:** bump version in both `extension.toml` and `extensions.toml`, run `git submodule update --remote extensions/<id>`.

---

## Common Gotchas

- `zed_extension_api` version must be checked against crates.io for latest compatible version
- `println!` / `dbg!` only visible with `zed --foreground`
- Language server binaries must be downloaded to `zed::extension_dir()`, not bundled in the extension
- `schema_version = 1` is current and required
- Windows paths in `extension.toml` must use `/` not `\`
- Tree-sitter grammar `rev` must be a full commit SHA, not a tag
- The `grammar` field in `languages/<name>/config.toml` must exactly match the key in `[grammars.<key>]`
- Agent Server extensions are deprecated since v0.221.x — use the ACP Registry instead for new agent integrations
