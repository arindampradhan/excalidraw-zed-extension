BINARY     := excalidraw-preview
INSTALL    := $(HOME)/.local/bin/$(BINARY)
RELEASE    := target/release/$(BINARY)
DEBUG      := target/debug/$(BINARY)
WEBVIEW    := preview-binary/webview-src
DEV_FILE   ?= preview-binary/test.excalidraw

.PHONY: all build build-debug ui install dev clean symlink help

## Default: build UI + release binary
all: ui build

## Build the release binary (embeds current assets/)
build:
	cargo build -p excalidraw-preview-binary --release

## Build the debug binary
build-debug:
	cargo build -p excalidraw-preview-binary

## Build the Zed extension WASM
build-ext:
	cargo build -p excalidraw-preview --release --target wasm32-wasip1

## Build the webview (npm install + vite build → assets/)
ui:
	cd $(WEBVIEW) && npm install && npm run build

## Full release: UI + binary + extension WASM
release: ui build build-ext

## Symlink ~/.local/bin/excalidraw-preview → target/release (one-time setup)
symlink:
	@mkdir -p $(HOME)/.local/bin
	ln -sf $(abspath $(RELEASE)) $(INSTALL)
	@echo "Symlinked $(INSTALL) → $(abspath $(RELEASE))"

## Start Vite dev server (set DEV_FILE=path/to/file.excalidraw to change target)
dev-ui:
	cd $(WEBVIEW) && DEV_FILE=$(abspath $(DEV_FILE)) npm run dev

## Open WebView pointed at the Vite dev server (run make dev-ui first)
dev-window:
	GDK_BACKEND=wayland $(DEBUG) --dev

## Full dev mode: Vite server + WebView in parallel
dev:
	@$(MAKE) build-debug
	@trap 'kill 0' INT; \
	  $(MAKE) dev-ui & \
	  sleep 2 && GDK_BACKEND=wayland $(DEBUG) --dev; \
	  wait

## Clean build artifacts (keeps assets/ so the extension still works)
clean:
	cargo clean

## Show this help
help:
	@grep -E '^##' Makefile | sed 's/## //'
