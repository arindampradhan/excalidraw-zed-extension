Zed Industries
Search docs…


Download
Getting Started
Installation
Update
Uninstall
Troubleshooting
Overview
Agent Panel
Tools
Tool Permissions
External Agents
Inline Assistant
Edit Prediction
Rules
Model Context Protocol
Configuration
LLM Providers
Agent Settings
Subscription
Models
Plans and Usage
Billing
Editing Code
Code Completions
Snippets
Diagnostics & Quick Fixes
Multibuffers
Finding & Navigating
Command Palette
Outline Panel
Tab Switcher
Running & Testing
Terminal
Tasks
Debugger
REPL
Git
Modelines
Overview
Channels
Contacts and Private Calls
Overview
Environment Variables
Dev Containers
macOS
Windows
Linux
Appearance
Themes
Icon Themes
Fonts & Visual Tweaks
Keybindings
Vim Mode
Helix Mode
All Languages
Configuring Languages
Toolchains
Semantic Tokens
Ansible
AsciiDoc
Astro
Bash
Biome
C
C++
C#
Clojure
CSS
Dart
Deno
Diff
Docker
Elixir
Elm
Emmet
Erlang
Fish
GDScript
Gleam
GLSL
Go
Groovy
Haskell
Helm
HTML
Java
JavaScript
Julia
JSON
Jsonnet
Kotlin
Lua
Luau
Makefile
Markdown
Nim
OCaml
OpenTofu
PHP
PowerShell
Prisma
Proto
PureScript
Python
R
Rego
ReStructuredText
Racket
Roc
Ruby
Rust
Scala
Scheme
Shell Script
SQL
Svelte
Swift
Tailwind CSS
Terraform
TOML
TypeScript
Uiua
Vue
XML
YAML
Yara
Yarn
Zig
Overview
Installing Extensions
Developing Extensions
Extension Capabilities
Language Extensions
Debugger Extensions
Theme Extensions
Icon Theme Extensions
Snippets Extensions
Agent Server Extensions
MCP Server Extensions
VS Code
IntelliJ IDEA
PyCharm
WebStorm
RustRover
All Settings
All Actions
CLI Reference
Authenticate
Roles
Privacy and Security
Worktree Trust
AI Improvement
Telemetry
Developing Zed
macOS
Linux
Windows
FreeBSD
Using Debuggers
Performance
Glossary
Release Notes
Debugging Crashes
Installing Extensions
Extensions add functionality to Zed, including languages, themes, and AI tools. Browse and install them from the Extension Gallery.

Open the Extension Gallery with ctrl-shift-x, or select "Zed > Extensions" from the menu bar.

Installation Location
On macOS, extensions are installed in ~/Library/Application Support/Zed/extensions.
On Linux, they are installed in either $XDG_DATA_HOME/zed/extensions or ~/.local/share/zed/extensions.
On Windows, the directory is %LOCALAPPDATA%\Zed\extensions.
This directory contains two subdirectories:

installed, which contains the source code for each extension.
work which contains files created by the extension itself, such as downloaded language servers.
Auto-installing
To automate extension installation/uninstallation see the docs for auto_install_extensions.

Overview
Developing Extensions
Zed Industries
•
Back to Site
•
Releases
•
Roadmap
•
GitHub
•
Blog
•
Manage Site Cookies
On This Page

Installation Location
Auto-installing


Zed Industries
Search docs…


Download
Getting Started
Installation
Update
Uninstall
Troubleshooting
Overview
Agent Panel
Tools
Tool Permissions
External Agents
Inline Assistant
Edit Prediction
Rules
Model Context Protocol
Configuration
LLM Providers
Agent Settings
Subscription
Models
Plans and Usage
Billing
Editing Code
Code Completions
Snippets
Diagnostics & Quick Fixes
Multibuffers
Finding & Navigating
Command Palette
Outline Panel
Tab Switcher
Running & Testing
Terminal
Tasks
Debugger
REPL
Git
Modelines
Overview
Channels
Contacts and Private Calls
Overview
Environment Variables
Dev Containers
macOS
Windows
Linux
Appearance
Themes
Icon Themes
Fonts & Visual Tweaks
Keybindings
Vim Mode
Helix Mode
All Languages
Configuring Languages
Toolchains
Semantic Tokens
Ansible
AsciiDoc
Astro
Bash
Biome
C
C++
C#
Clojure
CSS
Dart
Deno
Diff
Docker
Elixir
Elm
Emmet
Erlang
Fish
GDScript
Gleam
GLSL
Go
Groovy
Haskell
Helm
HTML
Java
JavaScript
Julia
JSON
Jsonnet
Kotlin
Lua
Luau
Makefile
Markdown
Nim
OCaml
OpenTofu
PHP
PowerShell
Prisma
Proto
PureScript
Python
R
Rego
ReStructuredText
Racket
Roc
Ruby
Rust
Scala
Scheme
Shell Script
SQL
Svelte
Swift
Tailwind CSS
Terraform
TOML
TypeScript
Uiua
Vue
XML
YAML
Yara
Yarn
Zig
Overview
Installing Extensions
Developing Extensions
Extension Capabilities
Language Extensions
Debugger Extensions
Theme Extensions
Icon Theme Extensions
Snippets Extensions
Agent Server Extensions
MCP Server Extensions
VS Code
IntelliJ IDEA
PyCharm
WebStorm
RustRover
All Settings
All Actions
CLI Reference
Authenticate
Roles
Privacy and Security
Worktree Trust
AI Improvement
Telemetry
Developing Zed
macOS
Linux
Windows
FreeBSD
Using Debuggers
Performance
Glossary
Release Notes
Debugging Crashes
Developing Extensions
Zed extensions are Git repositories containing an extension.toml manifest. They can provide languages, themes, debuggers, snippets, and MCP servers.

Extension Features
Extensions can provide:

Languages
Debuggers
Themes
Icon Themes
Snippets
MCP Servers
Developing an Extension Locally
Before starting to develop an extension for Zed, be sure to install Rust via rustup.

Rust must be installed via rustup. If you have Rust installed via homebrew or otherwise, installing dev extensions will not work.

When developing an extension, you can use it in Zed without needing to publish it by installing it as a dev extension.

From the extensions page, click the Install Dev Extension button (or the zed: install dev extension action) and select the directory containing your extension.

If you need to troubleshoot, check Zed.log (zed: open log) for additional output. For debug output, close and relaunch Zed from the command line with zed --foreground, which shows more verbose INFO-level logs.

If you already have the published version of the extension installed, the published version will be uninstalled prior to the installation of the dev extension. After successful installation, the Extensions page will indicate that the upstream extension is "Overridden by dev extension".

Directory Structure of a Zed Extension
A Zed extension is a Git repository that contains an extension.toml. This file must contain some basic information about the extension:

id = "my-extension"
name = "My extension"
version = "0.0.1"
schema_version = 1
authors = ["Your Name <you@example.com>"]
description = "Example extension"
repository = "https://github.com/your-name/my-zed-extension"
In addition to this, there are several other optional files and directories that can be used to add functionality to a Zed extension. An example directory structure of an extension that provides all capabilities is as follows:

my-extension/
  extension.toml
  Cargo.toml
  src/
    lib.rs
  languages/
    my-language/
      config.toml
      highlights.scm
  themes/
    my-theme.json
  snippets/
    snippets.json
    rust.json
WebAssembly
Procedural parts of extensions are written in Rust and compiled to WebAssembly. To develop an extension that includes custom code, include a Cargo.toml like this:

[package]
name = "my-extension"
version = "0.0.1"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
zed_extension_api = "0.1.0"
Use the latest version of the zed_extension_api available on crates.io. Make sure it's still compatible with Zed versions you want to support.

In the src/lib.rs file in your Rust crate you will need to define a struct for your extension and implement the Extension trait, as well as use the register_extension! macro to register your extension:

use zed_extension_api as zed;

struct MyExtension {
    // ... state
}

impl zed::Extension for MyExtension {
    // ...
}

zed::register_extension!(MyExtension);
stdout/stderr is forwarded directly to the Zed process. In order to see println!/dbg! output from your extension, you can start Zed in your terminal with a --foreground flag.

Forking and cloning the repo
Fork the repo
Note: It is very helpful if you fork the zed-industries/extensions repo to a personal GitHub account instead of a GitHub organization, as this allows Zed staff to push any needed changes to your PR to expedite the publishing process.

Clone the repo to your local machine
# Substitute the url of your fork here:
# git clone https://github.com/zed-industries/extensions
cd extensions
git submodule init
git submodule update
Extension License Requirements
As of October 1st, 2025, extension repositories must include a license. The following licenses are accepted:

Apache 2.0
BSD 2-Clause
BSD 3-Clause
CC BY 4.0
GNU GPLv3
GNU LGPLv3
MIT
Unlicense
zlib
This allows us to distribute the resulting binary produced from your extension code to our users. Without a valid license, the pull request to add or update your extension in the following steps will fail CI.

Your license file should be at the root of your extension repository. Any filename that has LICENCE or LICENSE as a prefix (case insensitive) will be inspected to ensure it matches one of the accepted licenses. See the license validation source code.

This license requirement applies only to your extension code itself (the code that gets compiled into the extension binary). It does not apply to any tools your extension may download or interact with, such as language servers or other external dependencies. If your repository contains both extension code and other projects (like a language server), you are not required to relicense those other projects — only the extension code needs to be one of the aforementioned accepted licenses.

Extension Publishing Prerequisites
Before publishing your extension, make sure that you have chosen a unique extension ID for your extension in the extension manifest. This will be the primary identifier for your extension and cannot be changed after your extension has been published. Also, ensure that you have filled out all the required fields in the manifest.

Furthermore, please make sure that your extension fulfills the following preconditions before you move on to publishing your extension:

Extension IDs and names must not contain the words zed, Zed or extension, since they are all Zed extensions.
Your extension ID should provide some information on what your extension tries to accomplish. E.g. for themes, it should be suffixed with -theme, snippet extensions should be suffixed with -snippets and so on. An exception to that rule are extension that provide support for languages or popular tooling that people would expect to find under that ID. You can take a look at the list of existing extensions to get a grasp on how this usually is enforced.
Extensions should provide something that is not yet available in the marketplace as opposed to fixing something that could be resolved within an existing extension. For example, if you find that an existing extension's support for a language server is not functioning properly, first try contributing a fix to the existing extension as opposed to submitting a new extension immediately.
If you receive no response or reaction within the upstream repository within a reasonable amount of time, feel free to submit a pull request that aims to fix said issue. Please ensure that you provide your previous efforts within the pull request to the extensions repository for adding your extension. Zed maintainers will then decide on how to proceed on a case by case basis.
Extensions that intend to provide a language, debugger or MCP server must not ship the language server as part of the extension. Instead, the extension should either download the language server or check for the availability of the language server in the users environment using the APIs as provided by the Zed Rust Extension API.
Themes and icon themes should not be published as part of extensions that provide other features, e.g. language support. Instead, they should be published as a distinct extension. This also applies to theme and icon themes living in the same repository.
Note that non-compliance will be raised during the publishing process by reviewers and delay the release of your extension.

Publishing your extension
To publish an extension, open a PR to the zed-industries/extensions repo.

In your PR, do the following:

Add your extension as a Git submodule within the extensions/ directory under the extensions/{extension-id} path
git submodule add https://github.com/your-username/foobar-zed.git extensions/my-extension
git add extensions/my-extension
All extension submodules must use HTTPS URLs and not SSH URLS (git@github.com).

Add a new entry to the top-level extensions.toml file containing your extension:
[my-extension]
submodule = "extensions/my-extension"
version = "0.0.1"
If your extension is in a subdirectory within the submodule, you can use the path field to point to where the extension resides:

[my-extension]
submodule = "extensions-my-extension"
path = "packages/zed"
version = "0.0.1"
Note that the required extension license must reside at the specified path, a license at the root of the repository will not work. However, you are free to symlink an existing license within the repository or choose an alternative license from the list of accepted licenses for the extension code.

Run pnpm sort-extensions to ensure extensions.toml and .gitmodules are sorted
Once your PR is merged, the extension will be packaged and published to the Zed extension registry.

Updating an extension
To update an extension, open a PR to the zed-industries/extensions repo.

In your PR do the following:

Update the extension's submodule to the commit of the new version. For this, you can run
# From the root of the repository:
git submodule update --remote extensions/your-extension-name
to update your extension to the latest commit available in your remote repository.

Update the version field for the extension in extensions.toml
Make sure the version matches the one set in extension.toml at the particular commit.
If you'd like to automate this process, there is a community GitHub Action you can use.

Note: If your extension repository has a different license, you'll need to update it to be one of the accepted extension licenses before publishing your update.

Installing Extensions
Extension Capabilities
Zed Industries
•
Back to Site
•
Releases
•
Roadmap
•
GitHub
•
Blog
•
Manage Site Cookies
On This Page

Extension Features
Developing an Extension Locally
Directory Structure of a Zed Extension
WebAssembly
Forking and cloning the repo
Extension License Requirements
Extension Publishing Prerequisites
Publishing your extension
Updating an extension

Zed Industries
Search docs…


Download
Getting Started
Installation
Update
Uninstall
Troubleshooting
Overview
Agent Panel
Tools
Tool Permissions
External Agents
Inline Assistant
Edit Prediction
Rules
Model Context Protocol
Configuration
LLM Providers
Agent Settings
Subscription
Models
Plans and Usage
Billing
Editing Code
Code Completions
Snippets
Diagnostics & Quick Fixes
Multibuffers
Finding & Navigating
Command Palette
Outline Panel
Tab Switcher
Running & Testing
Terminal
Tasks
Debugger
REPL
Git
Modelines
Overview
Channels
Contacts and Private Calls
Overview
Environment Variables
Dev Containers
macOS
Windows
Linux
Appearance
Themes
Icon Themes
Fonts & Visual Tweaks
Keybindings
Vim Mode
Helix Mode
All Languages
Configuring Languages
Toolchains
Semantic Tokens
Ansible
AsciiDoc
Astro
Bash
Biome
C
C++
C#
Clojure
CSS
Dart
Deno
Diff
Docker
Elixir
Elm
Emmet
Erlang
Fish
GDScript
Gleam
GLSL
Go
Groovy
Haskell
Helm
HTML
Java
JavaScript
Julia
JSON
Jsonnet
Kotlin
Lua
Luau
Makefile
Markdown
Nim
OCaml
OpenTofu
PHP
PowerShell
Prisma
Proto
PureScript
Python
R
Rego
ReStructuredText
Racket
Roc
Ruby
Rust
Scala
Scheme
Shell Script
SQL
Svelte
Swift
Tailwind CSS
Terraform
TOML
TypeScript
Uiua
Vue
XML
YAML
Yara
Yarn
Zig
Overview
Installing Extensions
Developing Extensions
Extension Capabilities
Language Extensions
Debugger Extensions
Theme Extensions
Icon Theme Extensions
Snippets Extensions
Agent Server Extensions
MCP Server Extensions
VS Code
IntelliJ IDEA
PyCharm
WebStorm
RustRover
All Settings
All Actions
CLI Reference
Authenticate
Roles
Privacy and Security
Worktree Trust
AI Improvement
Telemetry
Developing Zed
macOS
Linux
Windows
FreeBSD
Using Debuggers
Performance
Glossary
Release Notes
Debugging Crashes
Extension Capabilities
The operations that Zed extensions are able to perform are governed by a capability system.

Restricting capabilities
As a user, you have the option of restricting the capabilities that are granted to extensions.

This is controlled via the granted_extension_capabilities setting.

Restricting or removing a capability will cause an error to be returned when an extension attempts to call the corresponding extension API without sufficient capabilities.

For example, to restrict downloads to files from GitHub, set host for the download_file capability:

{
  "granted_extension_capabilities": [
    { "kind": "process:exec", "command": "*", "args": ["**"] },
-   { "kind": "download_file", "host": "*", "path": ["**"] },
+   { "kind": "download_file", "host": "github.com", "path": ["**"] },
    { "kind": "npm:install", "package": "*" }
  ]
}
If you don't want extensions to be able to perform any capabilities, you can remove all granted capabilities:

{
  "granted_extension_capabilities": []
}
Note that this will likely make many extensions non-functional, at least in their default configuration.

Capabilities
process:exec
The process:exec capability grants extensions the ability to invoke commands using zed_extension_api::process::Command.

Examples
To allow any command to be executed with any arguments:

{ kind = "process:exec", command = "*", args = ["**"] }
To allow a specific command (e.g., gem) to be executed with any arguments:

{ kind = "process:exec", command = "gem", args = ["**"] }
download_file
The download_file capability grants extensions the ability to download files using zed_extension_api::download_file.

Examples
To allow any file to be downloaded:

{ kind = "download_file", host = "*", path = ["**"] }
To allow any file to be downloaded from github.com:

{ kind = "download_file", host = "github.com", path = ["**"] }
To allow any file to be downloaded from a specific GitHub repository:

{ kind = "download_file", host = "github.com", path = ["zed-industries", "zed", "**"] }
npm:install
The npm:install capability grants extensions the ability to install npm packages using zed_extension_api::npm_install_package.

Examples
To allow any npm package to be installed:

{ kind = "npm:install", package = "*" }
To allow a specific npm package (e.g., typescript) to be installed:

{ kind = "npm:install", package = "typescript" }
Developing Extensions
Language Extensions
Zed Industries
•
Back to Site
•
Releases
•
Roadmap
•
GitHub
•
Blog
•
Manage Site Cookies
On This Page

Restricting capabilities
Capabilities
process:exec
Examples
download_file
Examples
npm:install
Examples

Zed Industries
Search docs…


Download
Getting Started
Installation
Update
Uninstall
Troubleshooting
Overview
Agent Panel
Tools
Tool Permissions
External Agents
Inline Assistant
Edit Prediction
Rules
Model Context Protocol
Configuration
LLM Providers
Agent Settings
Subscription
Models
Plans and Usage
Billing
Editing Code
Code Completions
Snippets
Diagnostics & Quick Fixes
Multibuffers
Finding & Navigating
Command Palette
Outline Panel
Tab Switcher
Running & Testing
Terminal
Tasks
Debugger
REPL
Git
Modelines
Overview
Channels
Contacts and Private Calls
Overview
Environment Variables
Dev Containers
macOS
Windows
Linux
Appearance
Themes
Icon Themes
Fonts & Visual Tweaks
Keybindings
Vim Mode
Helix Mode
All Languages
Configuring Languages
Toolchains
Semantic Tokens
Ansible
AsciiDoc
Astro
Bash
Biome
C
C++
C#
Clojure
CSS
Dart
Deno
Diff
Docker
Elixir
Elm
Emmet
Erlang
Fish
GDScript
Gleam
GLSL
Go
Groovy
Haskell
Helm
HTML
Java
JavaScript
Julia
JSON
Jsonnet
Kotlin
Lua
Luau
Makefile
Markdown
Nim
OCaml
OpenTofu
PHP
PowerShell
Prisma
Proto
PureScript
Python
R
Rego
ReStructuredText
Racket
Roc
Ruby
Rust
Scala
Scheme
Shell Script
SQL
Svelte
Swift
Tailwind CSS
Terraform
TOML
TypeScript
Uiua
Vue
XML
YAML
Yara
Yarn
Zig
Overview
Installing Extensions
Developing Extensions
Extension Capabilities
Language Extensions
Debugger Extensions
Theme Extensions
Icon Theme Extensions
Snippets Extensions
Agent Server Extensions
MCP Server Extensions
VS Code
IntelliJ IDEA
PyCharm
WebStorm
RustRover
All Settings
All Actions
CLI Reference
Authenticate
Roles
Privacy and Security
Worktree Trust
AI Improvement
Telemetry
Developing Zed
macOS
Linux
Windows
FreeBSD
Using Debuggers
Performance
Glossary
Release Notes
Debugging Crashes
Language Extensions
Language support in Zed has several components:

Language metadata and configuration
Grammar
Queries
Language servers
Language Metadata
Each language supported by Zed must be defined in a subdirectory inside the languages directory of your extension.

This subdirectory must contain a file called config.toml file with the following structure:

name = "My Language"
grammar = "my-language"
path_suffixes = ["myl"]
line_comments = ["# "]
name (required) is the human readable name that will show up in the Select Language dropdown.
grammar (required) is the name of a grammar. Grammars are registered separately, described below.
path_suffixes is an array of file suffixes that should be associated with this language. Unlike file_types in settings, this does not support glob patterns.
line_comments is an array of strings that are used to identify line comments in the language. This is used for the editor::ToggleComments keybind: ctrl-k ctrl-c for toggling lines of code.
tab_size defines the indentation/tab size used for this language (default is 4).
hard_tabs whether to indent with tabs (true) or spaces (false, the default).
first_line_pattern is a regular expression that can be used alongside path_suffixes (above) or file_types in settings to match files that should use this language. For example, Zed uses this to identify Shell Scripts by matching shebang lines in the first line of a script.
debuggers is an array of strings that are used to identify debuggers in the language. When launching a debugger's New Process Modal, Zed will order available debuggers by the order of entries in this array.
Grammar
Zed uses the Tree-sitter parsing library to provide built-in language-specific features. There are grammars available for many languages, and you can also develop your own grammar. A growing list of Zed features are built using pattern matching over syntax trees with Tree-sitter queries. As mentioned above, every language that is defined in an extension must specify the name of a Tree-sitter grammar that is used for parsing. These grammars are then registered separately in extensions' extension.toml file, like this:

[grammars.gleam]
repository = "https://github.com/gleam-lang/tree-sitter-gleam"
rev = "58b7cac8fc14c92b0677c542610d8738c373fa81"
The repository field must specify a repository where the Tree-sitter grammar should be loaded from, and the rev field must contain a Git revision to use, such as the SHA of a Git commit. If you're developing an extension locally and want to load a grammar from the local filesystem, you can use a file:// URL for repository. An extension can provide multiple grammars by referencing multiple tree-sitter repositories.

Tree-sitter Queries
Zed uses the syntax tree produced by the Tree-sitter query language to implement several features:

Syntax highlighting
Bracket matching
Code outline/structure
Auto-indentation
Code injections
Syntax overrides
Text redactions
Runnable code detection
Selecting classes, functions, etc.
The following sections elaborate on how Tree-sitter queries enable these features in Zed, using JSON syntax as a guiding example.

Syntax highlighting
In Tree-sitter, the highlights.scm file defines syntax highlighting rules for a particular syntax.

Here's an example from a highlights.scm for JSON:

(string) @string

(pair
  key: (string) @property.json_key)

(number) @number
This query marks strings, object keys, and numbers for highlighting. The following is the full list of captures supported by themes:

Capture	Description
@attribute	Captures attributes
@boolean	Captures boolean values
@comment	Captures comments
@comment.doc	Captures documentation comments
@constant	Captures constants
@constant.builtin	Captures built-in constants
@constructor	Captures constructors
@embedded	Captures embedded content
@emphasis	Captures emphasized text
@emphasis.strong	Captures strongly emphasized text
@enum	Captures enumerations
@function	Captures functions
@hint	Captures hints
@keyword	Captures keywords
@label	Captures labels
@link_text	Captures link text
@link_uri	Captures link URIs
@number	Captures numeric values
@operator	Captures operators
@predictive	Captures predictive text
@preproc	Captures preprocessor directives
@primary	Captures primary elements
@property	Captures properties
@punctuation	Captures punctuation
@punctuation.bracket	Captures brackets
@punctuation.delimiter	Captures delimiters
@punctuation.list_marker	Captures list markers
@punctuation.special	Captures special punctuation
@string	Captures string literals
@string.escape	Captures escaped characters in strings
@string.regex	Captures regular expressions
@string.special	Captures special strings
@string.special.symbol	Captures special symbols
@tag	Captures tags
@tag.doctype	Captures doctypes (e.g., in HTML)
@text.literal	Captures literal text
@title	Captures titles
@type	Captures types
@type.builtin	Captures built-in types
@variable	Captures variables
@variable.special	Captures special variables
@variable.parameter	Captures function/method parameters
@variant	Captures variants
Fallback captures
A single Tree-sitter pattern can specify multiple captures on the same node to define fallback highlights. Zed resolves them right-to-left: It first tries the rightmost capture, and if the current theme has no style for it, falls back to the next capture to the left, and so on.

For example:

(type_identifier) @type @variable
Here Zed will first try to resolve @variable from the theme. If the theme defines a style for @variable, that style is used. Otherwise, Zed falls back to @type.

This is useful when a language wants to provide a preferred highlight that not all themes may support, while still falling back to a more common capture that most themes define.

Bracket matching
The brackets.scm file defines matching brackets.

Here's an example from a brackets.scm file for JSON:

("[" @open "]" @close)
("{" @open "}" @close)
("\"" @open "\"" @close)
This query identifies opening and closing brackets, braces, and quotation marks.

Capture	Description
@open	Captures opening brackets, braces, and quotes
@close	Captures closing brackets, braces, and quotes
Zed uses these to highlight matching brackets: painting each bracket pair with a different color ("rainbow brackets") and highlighting the brackets if the cursor is inside the bracket pair.

To opt out of rainbow brackets colorization, add the following to the corresponding brackets.scm entry:

(("\"" @open "\"" @close) (#set! rainbow.exclude))
Code outline/structure
The outline.scm file defines the structure for the code outline.

Here's an example from an outline.scm file for JSON:

(pair
  key: (string (string_content) @name)) @item
This query captures object keys for the outline structure.

Capture	Description
@name	Captures the content of object keys
@item	Captures the entire key-value pair
@context	Captures elements that provide context for the outline item
@context.extra	Captures additional contextual information for the outline item
@annotation	Captures nodes that annotate outline item (doc comments, attributes, decorators)1
1 These annotations are used by Assistant when generating code modification steps.
Auto-indentation
The indents.scm file defines indentation rules.

Here's an example from an indents.scm file for JSON:

(array "]" @end) @indent
(object "}" @end) @indent
This query marks the end of arrays and objects for indentation purposes.

Capture	Description
@end	Captures closing brackets and braces
@indent	Captures entire arrays and objects for indentation
Code injections
The injections.scm file defines rules for embedding one language within another, such as code blocks in Markdown or SQL queries in Python strings.

Here's an example from an injections.scm file for Markdown:

(fenced_code_block
  (info_string
    (language) @injection.language)
  (code_fence_content) @injection.content)

((inline) @content
 (#set! injection.language "markdown-inline"))
This query identifies fenced code blocks, capturing the language specified in the info string and the content within the block. It also captures inline content and sets its language to "markdown-inline".

Capture	Description
@injection.language	Captures the language identifier for a code block
@injection.content	Captures the content to be treated as a different language
Note that we couldn't use JSON as an example here because it doesn't support language injections.

Syntax overrides
The overrides.scm file defines syntactic scopes that can be used to override certain editor settings within specific language constructs.

For example, there is a language-specific setting called word_characters that controls which non-alphabetic characters are considered part of a word, for example when you double click to select a variable. In JavaScript, "$" and "#" are considered word characters.

There is also a language-specific setting called completion_query_characters that controls which characters trigger autocomplete suggestions. In JavaScript, when your cursor is within a string, - should be considered a completion query character. To achieve this, the JavaScript overrides.scm file contains the following pattern:

[
  (string)
  (template_string)
] @string
And the JavaScript config.toml contains this setting:

word_characters = ["#", "$"]

[overrides.string]
completion_query_characters = ["-"]
You can also disable certain auto-closing brackets in a specific scope. For example, to prevent auto-closing ' within strings, you could put the following in the JavaScript config.toml:

brackets = [
  { start = "'", end = "'", close = true, newline = false, not_in = ["string"] },
  # other pairs...
]
Range inclusivity
By default, the ranges defined in overrides.scm are exclusive. So in the case above, if your cursor was outside the quotation marks delimiting the string, the string scope would not take effect. Sometimes, you may want to make the range inclusive. You can do this by adding the .inclusive suffix to the capture name in the query.

For example, in JavaScript, we also disable auto-closing of single quotes within comments. And the comment scope must extend all the way to the newline after a line comment. To achieve this, the JavaScript overrides.scm contains the following pattern:

(comment) @comment.inclusive
Text objects
The textobjects.scm file defines rules for navigating by text objects. This was added in Zed v0.165 and is currently used only in Vim mode.

Vim provides two levels of granularity for navigating around files. Section-by-section with [] etc., and method-by-method with ]m etc. Even languages that don't support functions and classes can work well by defining similar concepts. For example CSS defines a rule-set as a method, and a media-query as a class.

For languages with closures, these typically should not count as functions in Zed. This is best-effort, however, because languages like JavaScript do not syntactically differentiate between closures and top-level function declarations.

For languages with declarations like C, provide queries that match @class.around or @function.around. The if and ic text objects will default to these if there is no inside.

If you are not sure what to put in textobjects.scm, both nvim-treesitter-textobjects, and the Helix editor have queries for many languages. You can refer to the Zed built-in languages to see how to adapt these.

Capture	Description	Vim mode
@function.around	An entire function definition or equivalent small section of a file.	[m, ]m, [M,]M motions. af text object
@function.inside	The function body (the stuff within the braces).	if text object
@class.around	An entire class definition or equivalent large section of a file.	[[, ]], [], ][ motions. ac text object
@class.inside	The contents of a class definition.	ic text object
@comment.around	An entire comment (e.g. all adjacent line comments, or a block comment)	gc text object
@comment.inside	The contents of a comment	igc text object (rarely supported)
For example:

; include only the content of the method in the function
(method_definition
    body: (_
        "{"
        (_)* @function.inside
        "}")) @function.around

; match function.around for declarations with no body
(function_signature_item) @function.around

; join all adjacent comments into one
(comment)+ @comment.around
Text redactions
The redactions.scm file defines text redaction rules. When collaborating and sharing your screen, it makes sure that certain syntax nodes are rendered in a redacted mode to avoid them from leaking.

Here's an example from a redactions.scm file for JSON:

(pair value: (number) @redact)
(pair value: (string) @redact)
(array (number) @redact)
(array (string) @redact)
This query marks number and string values in key-value pairs and arrays for redaction.

Capture	Description
@redact	Captures values to be redacted
Runnable code detection
The runnables.scm file defines rules for detecting runnable code.

Here's an example from a runnables.scm file for JSON:

(
    (document
        (object
            (pair
                key: (string
                    (string_content) @_name
                    (#eq? @_name "scripts")
                )
                value: (object
                    (pair
                        key: (string (string_content) @run @script)
                    )
                )
            )
        )
    )
    (#set! tag package-script)
    (#set! tag composer-script)
)
This query detects runnable scripts in package.json and composer.json files.

The @run capture specifies where the run button should appear in the editor. Other captures, except those prefixed with an underscore, are exposed as environment variables with a prefix of ZED_CUSTOM_$(capture_name) when running the code.

Capture	Description
@_name	Captures the "scripts" key
@run	Captures the script name
@script	Also captures the script name (for different purposes)
Language Servers
Zed uses the Language Server Protocol to provide advanced language support.

An extension may provide any number of language servers. To provide a language server from your extension, add an entry to your extension.toml with the name of your language server and the language(s) it applies to. The entry in the list of languages has to match the name field from the config.toml file for that language:

[language_servers.my-language-server]
name = "My Language LSP"
languages = ["My Language"]
Then, in the Rust code for your extension, implement the language_server_command method on your extension:

impl zed::Extension for MyExtension {
    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: get_path_to_language_server_executable()?,
            args: get_args_for_language_server()?,
            env: get_env_for_language_server()?,
        })
    }
}
You can customize the handling of the language server using several optional methods in the Extension trait. For example, you can control how completions are styled using the label_for_completion method. For a complete list of methods, see the API docs for the Zed extension API.

Syntax Highlighting with Semantic Tokens
Zed supports syntax highlighting using semantic tokens from the attached language servers. This is currently disabled by default, but can be enabled in your settings file:

{
  // Enable semantic tokens globally, backed by tree-sitter highlights for each language:
  "semantic_tokens": "combined",
  // Or, specify per-language:
  "languages": {
    "Rust": {
      // No tree-sitter, only LSP semantic tokens:
      "semantic_tokens": "full"
    }
  }
}
The semantic_tokens setting accepts the following values:

"off" (default): Do not request semantic tokens from language servers.
"combined": Use LSP semantic tokens together with tree-sitter highlighting.
"full": Use LSP semantic tokens exclusively, replacing tree-sitter highlighting.
Extension-Provided Semantic Token Rules
Language extensions can ship default semantic token rules for their language server's custom token types. To do this, place a semantic_token_rules.json file in the language directory alongside config.toml:

my-extension/
  languages/
    my-language/
      config.toml
      highlights.scm
      semantic_token_rules.json
The file uses the same format as the semantic_token_rules array in user settings — a JSON array of rule objects:

[
  {
    "token_type": "lifetime",
    "style": ["lifetime"]
  },
  {
    "token_type": "builtinType",
    "style": ["type"]
  },
  {
    "token_type": "selfKeyword",
    "style": ["variable.special"]
  }
]
This is useful when a language server reports custom (non-standard) semantic token types that aren't covered by Zed's built-in default rules. Extension-provided rules act as sensible defaults for that language — users can always override them via semantic_token_rules in their settings file, and built-in default rules are only used when neither user nor extension rules match.

Customizing Semantic Token Styles
Zed supports customizing the styles used for semantic tokens. You can define rules in your settings file, which customize how semantic tokens get mapped to styles in your theme.

{
  "global_lsp_settings": {
    "semantic_token_rules": [
      {
        // Highlight macros as keywords.
        "token_type": "macro",
        "style": ["syntax.keyword"]
      },
      {
        // Highlight unresolved references in bold red.
        "token_type": "unresolvedReference",
        "foreground_color": "#c93f3f",
        "font_weight": "bold"
      },
      {
        // Underline all mutable variables/references/etc.
        "token_modifiers": ["mutable"],
        "underline": true
      }
    ]
  }
}
All rules that match a given token_type and token_modifiers are applied. Earlier rules take precedence. If no rules match, the token is not highlighted.

Rules are applied in the following priority order (highest to lowest):

User settings — rules from semantic_token_rules in your settings file.
Extension rules — rules from semantic_token_rules.json in extension language directories.
Default rules — Zed's built-in rules for standard LSP token types.
Each rule in the semantic_token_rules array is defined as follows:

token_type: The semantic token type as defined by the LSP specification. If omitted, the rule matches all token types.
token_modifiers: A list of semantic token modifiers to match. All modifiers must be present to match.
style: A list of styles from the current syntax theme to use. The first style found is used. Any settings below override that style.
foreground_color: The foreground color to use for the token type, in hex format (e.g., "#ff0000").
background_color: The background color to use for the token type, in hex format (e.g., "#ff0000").
underline: A boolean or color to underline with, in hex format. If true, then the token will be underlined with the text color.
strikethrough: A boolean or color to strikethrough with, in hex format. If true, then the token have a strikethrough with the text color.
font_weight: One of "normal", "bold".
font_style: One of "normal", "italic".
Multi-Language Support
If your language server supports additional languages, you can use language_ids to map Zed languages to the desired LSP-specific languageId identifiers:


[language-servers.my-language-server]
name = "Whatever LSP"
languages = ["JavaScript", "HTML", "CSS"]

[language-servers.my-language-server.language_ids]
"JavaScript" = "javascript"
"TSX" = "typescriptreact"
"HTML" = "html"
"CSS" = "css"
Extension Capabilities
Debugger Extensions
Zed Industries
•
Back to Site
•
Releases
•
Roadmap
•
GitHub
•
Blog
•
Manage Site Cookies
On This Page

Language Metadata
Grammar
Tree-sitter Queries
Syntax highlighting
Fallback captures
Bracket matching
Code outline/structure
Auto-indentation
Code injections
Syntax overrides
Range inclusivity
Text objects
Text redactions
Runnable code detection
Language Servers
Syntax Highlighting with Semantic Tokens
Extension-Provided Semantic Token Rules
Customizing Semantic Token Styles
Multi-Language Support

Zed Industries

Search…

Getting Started
Installation
Update
Uninstall
Troubleshooting
Overview
Agent Panel
Tools
Tool Permissions
External Agents
Inline Assistant
Edit Prediction
Rules
Model Context Protocol
Configuration
LLM Providers
Agent Settings
Subscription
Models
Plans and Usage
Billing
Editing Code
Code Completions
Snippets
Diagnostics & Quick Fixes
Multibuffers
Finding & Navigating
Command Palette
Outline Panel
Tab Switcher
Running & Testing
Terminal
Tasks
Debugger
REPL
Git
Modelines
Overview
Channels
Contacts and Private Calls
Overview
Environment Variables
Dev Containers
macOS
Windows
Linux
Appearance
Themes
Icon Themes
Fonts & Visual Tweaks
Keybindings
Vim Mode
Helix Mode
All Languages
Configuring Languages
Toolchains
Semantic Tokens
Ansible
AsciiDoc
Astro
Bash
Biome
C
C++
C#
Clojure
CSS
Dart
Deno
Diff
Docker
Elixir
Elm
Emmet
Erlang
Fish
GDScript
Gleam
GLSL
Go
Groovy
Haskell
Helm
HTML
Java
JavaScript
Julia
JSON
Jsonnet
Kotlin
Lua
Luau
Makefile
Markdown
Nim
OCaml
OpenTofu
PHP
PowerShell
Prisma
Proto
PureScript
Python
R
Rego
ReStructuredText
Racket
Roc
Ruby
Rust
Scala
Scheme
Shell Script
SQL
Svelte
Swift
Tailwind CSS
Terraform
TOML
TypeScript
Uiua
Vue
XML
YAML
Yara
Yarn
Zig
Overview
Installing Extensions
Developing Extensions
Extension Capabilities
Language Extensions
Debugger Extensions
Theme Extensions
Icon Theme Extensions
Snippets Extensions
Agent Server Extensions
MCP Server Extensions
VS Code
IntelliJ IDEA
PyCharm
WebStorm
RustRover
All Settings
All Actions
CLI Reference
Authenticate
Roles
Privacy and Security
Worktree Trust
AI Improvement
Telemetry
Developing Zed
macOS
Linux
Windows
FreeBSD
Using Debuggers
Performance
Glossary
Release Notes
Debugging Crashes
Debugger Extensions
Debug Adapter Protocol Servers can be exposed as extensions for use in the debugger.

Defining Debugger Extensions
A given extension may provide one or more DAP servers. Each DAP server must be registered in the extension.toml:


[debug_adapters.my-debug-adapter]
# Optional relative path to the JSON schema for the debug adapter configuration schema. Defaults to `debug_adapter_schemas/$DEBUG_ADAPTER_NAME_ID.json`.
# Note that while this field is optional, a schema is mandatory.
schema_path = "relative/path/to/schema.json"
Then, in the Rust code for your extension, implement the get_dap_binary method on your extension:

impl zed::Extension for MyExtension {
    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        config: DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String>;
}
This method should return the command to start up a debug adapter protocol server, along with any arguments or environment variables necessary for it to function.

If you need to download the DAP server from an external source (GitHub Releases, npm, etc.), you can also do that in this function. Make sure to check for updates only periodically, as this function is called whenever a user spawns a new debug session with your debug adapter.

You must also implement dap_request_kind. This function is used to determine whether a given debug scenario will launch a new debuggee or attach to an existing one. We also use it to determine that a given debug scenario requires running a locator.

impl zed::Extension for MyExtension {
    fn dap_request_kind(
        &mut self,
        _adapter_name: String,
        _config: Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String>;
}
These two functions are sufficient to expose your debug adapter in debug.json-based user workflows, but you should strongly consider implementing dap_config_to_scenario as well.

impl zed::Extension for MyExtension {
    fn dap_config_to_scenario(
        &mut self,
        _adapter_name: DebugConfig,
    ) -> Result<DebugScenario, String>;
}
dap_config_to_scenario is used when the user spawns a session via new process modal UI. At a high level, it takes a generic debug configuration (that isn't specific to any debug adapter) and tries to turn it into a concrete debug scenario for your adapter. Put another way, it is supposed to answer the question: "Given a program, a list of arguments, current working directory and environment variables, what would the configuration for spawning this debug adapter look like?".

Defining Debug Locators
Zed offers an automatic way to create debug scenarios with debug locators. A locator locates the debug target and figures out how to spawn a debug session for it. Thanks to locators, we can automatically convert existing user tasks (e.g. cargo run) and convert them into debug scenarios (e.g. cargo build followed by spawning a debugger with target/debug/my_program as the program to debug).

Your extension can define its own debug locators even if it does not expose a debug adapter. We strongly recommend doing so when your extension already exposes language tasks, as it allows users to spawn a debug session without having to manually configure the debug adapter.

Locators can (but don't have to) be agnostic to the debug adapter they are used with. They are responsible for locating the debug target and figuring out how to spawn a debug session for it. This lets extensions share locator logic across adapters.

Your extension can define one or more debug locators. Each debug locator must be registered in the extension.toml:

[debug_locators.my-debug-locator]
Locators have two components. First, each locator is ran on each available task to figure out if any of the available locators can provide a debug scenario for a given task. This is done by calling dap_locator_create_scenario.

impl zed::Extension for MyExtension {
    fn dap_locator_create_scenario(
        &mut self,
        _locator_name: String,
        _build_task: TaskTemplate,
        _resolved_label: String,
        _debug_adapter_name: String,
    ) -> Option<DebugScenario>;
}
This function should return Some debug scenario when that scenario defines a debugging counterpart to a given user task. Note that a DebugScenario can include a build task. If there is one, we will execute run_dap_locator after a build task is finished successfully.

impl zed::Extension for MyExtension {
    fn run_dap_locator(
        &mut self,
        _locator_name: String,
        _build_task: TaskTemplate,
    ) -> Result<DebugRequest, String>;
}
run_dap_locator is useful in case you cannot determine a build target deterministically. Some build systems may produce artifacts whose names are not known up-front. Note however that you do not need to go through a 2-phase resolution; if you can determine the full debug configuration with just dap_locator_create_scenario, you can omit build property on a returned DebugScenario. Please also note that your locator will be called with tasks it's unlikely to accept; thus you should take some effort to return None early before performing any expensive operations.

Available Extensions
See DAP servers published as extensions on Zed's site.

Review their repositories to see common implementation patterns and structure.

Testing
To test your new Debug Adapter Protocol server extension, you can install it as a dev extension.

Language Extensions
Theme Extensions
Zed Industries
•
Back to Site
•
Releases
•
Roadmap
•
GitHub
•
Blog
•
Manage Site Cookies


Zed Industries

Search…

Getting Started
Installation
Update
Uninstall
Troubleshooting
Overview
Agent Panel
Tools
Tool Permissions
External Agents
Inline Assistant
Edit Prediction
Rules
Model Context Protocol
Configuration
LLM Providers
Agent Settings
Subscription
Models
Plans and Usage
Billing
Editing Code
Code Completions
Snippets
Diagnostics & Quick Fixes
Multibuffers
Finding & Navigating
Command Palette
Outline Panel
Tab Switcher
Running & Testing
Terminal
Tasks
Debugger
REPL
Git
Modelines
Overview
Channels
Contacts and Private Calls
Overview
Environment Variables
Dev Containers
macOS
Windows
Linux
Appearance
Themes
Icon Themes
Fonts & Visual Tweaks
Keybindings
Vim Mode
Helix Mode
All Languages
Configuring Languages
Toolchains
Semantic Tokens
Ansible
AsciiDoc
Astro
Bash
Biome
C
C++
C#
Clojure
CSS
Dart
Deno
Diff
Docker
Elixir
Elm
Emmet
Erlang
Fish
GDScript
Gleam
GLSL
Go
Groovy
Haskell
Helm
HTML
Java
JavaScript
Julia
JSON
Jsonnet
Kotlin
Lua
Luau
Makefile
Markdown
Nim
OCaml
OpenTofu
PHP
PowerShell
Prisma
Proto
PureScript
Python
R
Rego
ReStructuredText
Racket
Roc
Ruby
Rust
Scala
Scheme
Shell Script
SQL
Svelte
Swift
Tailwind CSS
Terraform
TOML
TypeScript
Uiua
Vue
XML
YAML
Yara
Yarn
Zig
Overview
Installing Extensions
Developing Extensions
Extension Capabilities
Language Extensions
Debugger Extensions
Theme Extensions
Icon Theme Extensions
Snippets Extensions
Agent Server Extensions
MCP Server Extensions
VS Code
IntelliJ IDEA
PyCharm
WebStorm
RustRover
All Settings
All Actions
CLI Reference
Authenticate
Roles
Privacy and Security
Worktree Trust
AI Improvement
Telemetry
Developing Zed
macOS
Linux
Windows
FreeBSD
Using Debuggers
Performance
Glossary
Release Notes
Debugging Crashes
Themes
The themes directory in an extension should contain one or more theme files.

Each theme file should adhere to the JSON schema specified at https://zed.dev/schema/themes/v0.2.0.json.

See this blog post for additional background on creating themes.

Theme JSON Structure
The structure of a Zed theme is defined in the Zed Theme JSON Schema.

A Zed theme consists of a Theme Family object including:

name: The name for the theme family
author: The name of the author of the theme family
themes: An array of Themes belonging to the theme family
The core components of a Theme object include:

Theme Metadata:

name: The name of the theme
appearance: Either "light" or "dark"
Style Properties under the style, such as:

background: The main background color
foreground: The main text color
accent: The accent color used for highlighting and emphasis
Syntax Highlighting:

syntax: An object containing color definitions for various syntax elements (e.g., keywords, strings, comments)
UI Elements:

Colors for various UI components such as:
element.background: Background color for UI elements
border: Border colors for different states (normal, focused, selected)
text: Text colors for different states (normal, muted, accent)
Editor-specific Colors:

Colors for editor-related elements such as:
editor.background: Editor background color
editor.gutter: Gutter colors
editor.line_number: Line number colors
Terminal Colors:

ANSI color definitions for the integrated terminal
Designing Your Theme
You can use Zed's Theme Builder to design your own custom theme based on an existing one.

This tool lets you fine-tune and preview how surfaces in Zed will look. You can then export the JSON and publish it in Zed's extension store.

Debugger Extensions
Icon Theme Extensions
Zed Industries
•
Back to Site
•
Releases
•
Roadmap
•
GitHub
•
Blog
•
Manage Site Cookies

Zed Industries

Search…

Getting Started
Installation
Update
Uninstall
Troubleshooting
Overview
Agent Panel
Tools
Tool Permissions
External Agents
Inline Assistant
Edit Prediction
Rules
Model Context Protocol
Configuration
LLM Providers
Agent Settings
Subscription
Models
Plans and Usage
Billing
Editing Code
Code Completions
Snippets
Diagnostics & Quick Fixes
Multibuffers
Finding & Navigating
Command Palette
Outline Panel
Tab Switcher
Running & Testing
Terminal
Tasks
Debugger
REPL
Git
Modelines
Overview
Channels
Contacts and Private Calls
Overview
Environment Variables
Dev Containers
macOS
Windows
Linux
Appearance
Themes
Icon Themes
Fonts & Visual Tweaks
Keybindings
Vim Mode
Helix Mode
All Languages
Configuring Languages
Toolchains
Semantic Tokens
Ansible
AsciiDoc
Astro
Bash
Biome
C
C++
C#
Clojure
CSS
Dart
Deno
Diff
Docker
Elixir
Elm
Emmet
Erlang
Fish
GDScript
Gleam
GLSL
Go
Groovy
Haskell
Helm
HTML
Java
JavaScript
Julia
JSON
Jsonnet
Kotlin
Lua
Luau
Makefile
Markdown
Nim
OCaml
OpenTofu
PHP
PowerShell
Prisma
Proto
PureScript
Python
R
Rego
ReStructuredText
Racket
Roc
Ruby
Rust
Scala
Scheme
Shell Script
SQL
Svelte
Swift
Tailwind CSS
Terraform
TOML
TypeScript
Uiua
Vue
XML
YAML
Yara
Yarn
Zig
Overview
Installing Extensions
Developing Extensions
Extension Capabilities
Language Extensions
Debugger Extensions
Theme Extensions
Icon Theme Extensions
Snippets Extensions
Agent Server Extensions
MCP Server Extensions
VS Code
IntelliJ IDEA
PyCharm
WebStorm
RustRover
All Settings
All Actions
CLI Reference
Authenticate
Roles
Privacy and Security
Worktree Trust
AI Improvement
Telemetry
Developing Zed
macOS
Linux
Windows
FreeBSD
Using Debuggers
Performance
Glossary
Release Notes
Debugging Crashes
Snippets
Extensions may provide snippets for one or more languages.

Each file containing snippets can be specified in the snippets field of the extensions.toml file.

The referenced path must be relative to the extension.toml.

Defining Snippets
A given extension may provide one or more snippets. Each snippet must be registered in the extension.toml.

Zed matches snippet files based on the lowercase name of the language (e.g. rust.json for Rust). You can use snippets.json as a file name to define snippets that will be available regardless of the current buffer language.

For example, here is an extension that provides snippets for Rust and TypeScript:

snippets = ["./snippets/rust.json", "./snippets/typescript.json"]
For more information on how to create snippets, see the Snippets documentation.

Icon Theme Extensions
Agent Server Extensions
Zed Industries
•
Back to Site
•
Releases
•
Roadmap
•
GitHub
•
Blog
•
Manage Site Cookies


Zed Industries

Search…

Getting Started
Installation
Update
Uninstall
Troubleshooting
Overview
Agent Panel
Tools
Tool Permissions
External Agents
Inline Assistant
Edit Prediction
Rules
Model Context Protocol
Configuration
LLM Providers
Agent Settings
Subscription
Models
Plans and Usage
Billing
Editing Code
Code Completions
Snippets
Diagnostics & Quick Fixes
Multibuffers
Finding & Navigating
Command Palette
Outline Panel
Tab Switcher
Running & Testing
Terminal
Tasks
Debugger
REPL
Git
Modelines
Overview
Channels
Contacts and Private Calls
Overview
Environment Variables
Dev Containers
macOS
Windows
Linux
Appearance
Themes
Icon Themes
Fonts & Visual Tweaks
Keybindings
Vim Mode
Helix Mode
All Languages
Configuring Languages
Toolchains
Semantic Tokens
Ansible
AsciiDoc
Astro
Bash
Biome
C
C++
C#
Clojure
CSS
Dart
Deno
Diff
Docker
Elixir
Elm
Emmet
Erlang
Fish
GDScript
Gleam
GLSL
Go
Groovy
Haskell
Helm
HTML
Java
JavaScript
Julia
JSON
Jsonnet
Kotlin
Lua
Luau
Makefile
Markdown
Nim
OCaml
OpenTofu
PHP
PowerShell
Prisma
Proto
PureScript
Python
R
Rego
ReStructuredText
Racket
Roc
Ruby
Rust
Scala
Scheme
Shell Script
SQL
Svelte
Swift
Tailwind CSS
Terraform
TOML
TypeScript
Uiua
Vue
XML
YAML
Yara
Yarn
Zig
Overview
Installing Extensions
Developing Extensions
Extension Capabilities
Language Extensions
Debugger Extensions
Theme Extensions
Icon Theme Extensions
Snippets Extensions
Agent Server Extensions
MCP Server Extensions
VS Code
IntelliJ IDEA
PyCharm
WebStorm
RustRover
All Settings
All Actions
CLI Reference
Authenticate
Roles
Privacy and Security
Worktree Trust
AI Improvement
Telemetry
Developing Zed
macOS
Linux
Windows
FreeBSD
Using Debuggers
Performance
Glossary
Release Notes
Debugging Crashes
Agent Server Extensions
Note that starting from v0.221.x, the ACP Registry is the preferred way to install external agents in Zed. You can learn more about it in the release blog post

At some point in the near future, Agent Server extensions will be deprecated.

Agent Servers are programs that provide AI agent implementations through the Agent Client Protocol (ACP). Agent Server Extensions let you package an Agent Server so users can install the extension and use your agent in Zed.

You can see the current Agent Server extensions either by opening the Extensions tab in Zed (execute the zed: extensions command) and changing the filter from All to Agent Servers, or by visiting the Zed website.

Defining Agent Server Extensions
An extension can register one or more agent servers in the extension.toml:

[agent_servers.my-agent]
name = "My Agent"

[agent_servers.my-agent.targets.darwin-aarch64]
archive = "https://github.com/owner/repo/releases/download/v1.0.0/agent-darwin-arm64.tar.gz"
cmd = "./agent"
args = ["--serve"]

[agent_servers.my-agent.targets.linux-x86_64]
archive = "https://github.com/owner/repo/releases/download/v1.0.0/agent-linux-x64.tar.gz"
cmd = "./agent"
args = ["--serve"]

[agent_servers.my-agent.targets.windows-x86_64]
archive = "https://github.com/owner/repo/releases/download/v1.0.0/agent-windows-x64.zip"
cmd = "./agent.exe"
args = ["--serve"]
Required Fields
name: A human-readable display name for the agent server (shown in menus)
targets: Platform-specific configurations for downloading and running the agent
Target Configuration
Each target key uses the format {os}-{arch} where:

os: darwin (macOS), linux, or windows
arch: aarch64 (ARM64) or x86_64
Each target must specify:

archive: URL to download the archive from (supports .tar.gz, .zip, etc.)
cmd: Command to run the agent server (relative to the extracted archive)
args: Command-line arguments to pass to the agent server (optional)
sha256: SHA-256 hash string of the archive's bytes (optional, but recommended for security)
env: Environment variables specific to this target (optional, overrides agent-level env vars with the same name)
Optional Fields
You can also optionally specify at the agent server level:

env: Environment variables to set in the agent's spawned process. These apply to all targets by default.
icon: Path to an SVG icon (relative to extension root) for display in menus.
Environment Variables
Environment variables can be configured at two levels:

Agent-level ([agent_servers.my-agent.env]): Variables that apply to all platforms
Target-level ([agent_servers.my-agent.targets.{platform}.env]): Variables specific to a platform
When both are specified, target-level environment variables override agent-level variables with the same name. Variables defined only at the agent level are inherited by all targets.

Complete Example
Here's a more complete example with all optional fields:

[agent_servers.example-agent]
name = "Example Agent"
icon = "icon/agent.svg"

[agent_servers.example-agent.env]
AGENT_LOG_LEVEL = "info"
AGENT_MODE = "production"

[agent_servers.example-agent.targets.darwin-aarch64]
archive = "https://github.com/example/agent/releases/download/v2.0.0/agent-darwin-arm64.tar.gz"
cmd = "./bin/agent"
args = ["serve", "--port", "8080"]
sha256 = "abc123def456..."

[agent_servers.example-agent.targets.linux-x86_64]
archive = "https://github.com/example/agent/releases/download/v2.0.0/agent-linux-x64.tar.gz"
cmd = "./bin/agent"
args = ["serve", "--port", "8080"]
sha256 = "def456abc123..."

[agent_servers.example-agent.targets.linux-x86_64.env]
AGENT_MEMORY_LIMIT = "2GB"  # Linux-specific override
Installation Process
When a user installs your extension and selects the agent server:

Zed downloads the appropriate archive for the user's platform
The archive is extracted to a cache directory
Zed launches the agent using the specified command and arguments
Environment variables are set as configured
The agent server runs in the background, ready to assist the user
Archives are cached locally, so subsequent launches are fast.

Distribution Best Practices
Use GitHub Releases
GitHub Releases are a reliable way to distribute agent server binaries:

Build your agent for each platform (macOS ARM64, macOS x86_64, Linux x86_64, Windows x86_64)
Package each build as a compressed archive (.tar.gz or .zip)
Create a GitHub release and upload the archives
Use the release URLs in your extension.toml
SHA-256 Hashes
For better supply-chain security, include SHA-256 hashes of your archives in extension.toml. Here's how to generate one:

macOS and Linux
shasum -a 256 agent-darwin-arm64.tar.gz
Windows
certutil -hashfile agent-windows-x64.zip SHA256
Then add that string to your target configuration:

[agent_servers.my-agent.targets.darwin-aarch64]
archive = "https://github.com/owner/repo/releases/download/v1.0.0/agent-darwin-arm64.tar.gz"
cmd = "./agent"
sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
Testing
To test your Agent Server Extension:

Install it as a dev extension
Open the Agent Panel
Select your Agent Server from the list
Verify that it downloads, installs, and launches correctly
Test its functionality by conversing with it and watching the ACP logs
Icon Guideline
If your agent server has a logo, add it as an SVG icon. For consistent rendering, follow these guidelines:

Resize your SVG to fit a 16x16 bounding box with around one or two pixels of padding
Keep the SVG markup clean by processing it through SVGOMG
Avoid gradients, which often increase SVG complexity and can render inconsistently
Note that we'll automatically convert your icon to monochrome to preserve Zed's design consistency. (You can still use opacity in different paths of your SVG to add visual layering.)

Publishing
Once your extension is ready, see Publishing your extension to learn how to submit it to the Zed extension registry.

Snippets Extensions
MCP Server Extensions
Zed Industries
•
Back to Site
•
Releases
•
Roadmap
•
GitHub
•
Blog
•
Manage Site Cookies


Zed Industries

Search…

Getting Started
Installation
Update
Uninstall
Troubleshooting
Overview
Agent Panel
Tools
Tool Permissions
External Agents
Inline Assistant
Edit Prediction
Rules
Model Context Protocol
Configuration
LLM Providers
Agent Settings
Subscription
Models
Plans and Usage
Billing
Editing Code
Code Completions
Snippets
Diagnostics & Quick Fixes
Multibuffers
Finding & Navigating
Command Palette
Outline Panel
Tab Switcher
Running & Testing
Terminal
Tasks
Debugger
REPL
Git
Modelines
Overview
Channels
Contacts and Private Calls
Overview
Environment Variables
Dev Containers
macOS
Windows
Linux
Appearance
Themes
Icon Themes
Fonts & Visual Tweaks
Keybindings
Vim Mode
Helix Mode
All Languages
Configuring Languages
Toolchains
Semantic Tokens
Ansible
AsciiDoc
Astro
Bash
Biome
C
C++
C#
Clojure
CSS
Dart
Deno
Diff
Docker
Elixir
Elm
Emmet
Erlang
Fish
GDScript
Gleam
GLSL
Go
Groovy
Haskell
Helm
HTML
Java
JavaScript
Julia
JSON
Jsonnet
Kotlin
Lua
Luau
Makefile
Markdown
Nim
OCaml
OpenTofu
PHP
PowerShell
Prisma
Proto
PureScript
Python
R
Rego
ReStructuredText
Racket
Roc
Ruby
Rust
Scala
Scheme
Shell Script
SQL
Svelte
Swift
Tailwind CSS
Terraform
TOML
TypeScript
Uiua
Vue
XML
YAML
Yara
Yarn
Zig
Overview
Installing Extensions
Developing Extensions
Extension Capabilities
Language Extensions
Debugger Extensions
Theme Extensions
Icon Theme Extensions
Snippets Extensions
Agent Server Extensions
MCP Server Extensions
VS Code
IntelliJ IDEA
PyCharm
WebStorm
RustRover
All Settings
All Actions
CLI Reference
Authenticate
Roles
Privacy and Security
Worktree Trust
AI Improvement
Telemetry
Developing Zed
macOS
Linux
Windows
FreeBSD
Using Debuggers
Performance
Glossary
Release Notes
Debugging Crashes
MCP Server Extensions
Model Context Protocol servers can be exposed as extensions for use in the Agent Panel.

Defining MCP Extensions
A given extension may provide one or more MCP servers. Each MCP server must be registered in the extension.toml:


[context_servers.my-context-server]
Then, in the Rust code for your extension, implement the context_server_command method on your extension:

impl zed::Extension for MyExtension {
    fn context_server_command(
        &mut self,
        context_server_id: &ContextServerId,
        project: &zed::Project,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: get_path_to_context_server_executable()?,
            args: get_args_for_context_server()?,
            env: get_env_for_context_server()?,
        })
    }
}
This method should return the command to start up an MCP server, along with any arguments or environment variables necessary for it to function.

If you need to download the MCP server from an external source (GitHub Releases, npm, etc.), you can also do that in this function.

Available Extensions
See MCP servers published as extensions on Zed's site.

Review their repositories to see common implementation patterns and structure.

Testing
To test your new MCP server extension, you can install it as a dev extension.

Agent Server Extensions
VS Code
Zed Industries
•
Back to Site
•
Releases
•
Roadmap
•
GitHub
•
Blog
•
Manage Site Cookies