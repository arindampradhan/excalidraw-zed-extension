import { useState, useEffect, useCallback, useRef } from "react";
import {
  Excalidraw,
  MainMenu,
  serializeAsJSON,
  exportToSvg,
  exportToBlob,
  hashElementsVersion,
} from "@excalidraw/excalidraw";
import "@excalidraw/excalidraw/index.css";

interface AppProps {
  initialData: ExcalidrawInitialDataState;
  theme: string;
  name: string;
  contentType: string;
  /** When true, saves to disk after every element change (debounced 600 ms). */
  autoSave: boolean;
  onApiReady: (api: ExcalidrawImperativeAPI) => void;
  /** Called after a successful save so the SSE listener can suppress the echo. */
  onSaved: (suppressUntil: number) => void;
  /**
   * Called once with a stable `reloadScene` function that the SSE handler can
   * call when an external file change arrives.  The function skips the update
   * when the user is actively editing a text element (prevents mid-edit
   * disruption) and only passes elements + files to updateScene so the
   * current viewport position and theme are never reset.
   */
  onReloadReady: (reload: (data: ExcalidrawInitialDataState) => void) => void;
}

function useOsTheme(preference: "auto" | "light" | "dark"): "light" | "dark" {
  const [theme, setTheme] = useState<"light" | "dark">(() => {
    if (preference !== "auto") return preference;
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  });

  useEffect(() => {
    if (preference !== "auto") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) =>
      setTheme(e.matches ? "dark" : "light");
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [preference]);

  return theme;
}

const SAVE_DEBOUNCE_MS = 600;

export default function App({
  initialData,
  theme,
  name,
  contentType,
  autoSave,
  onApiReady,
  onSaved,
  onReloadReady,
}: AppProps) {
  const resolvedTheme = useOsTheme(theme as "auto" | "light" | "dark");
  const apiRef = useRef<ExcalidrawImperativeAPI | null>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  // Tracks the element hash of the last-saved state so auto-save only fires
  // on real element changes, not on viewport pan / zoom / selection.
  const prevVersionRef = useRef<number>(-1);

  // Seed prevVersionRef from the initial scene so the first onChange
  // (which Excalidraw fires immediately on mount) doesn't trigger a spurious save.
  useEffect(() => {
    if (initialData?.elements) {
      prevVersionRef.current = hashElementsVersion(
        initialData.elements as readonly ExcalidrawElement[],
      );
    }
  }, []);

  // Stable reload function handed to the SSE handler in main.tsx.
  // - Skips if the user is mid-edit (prevents text-editor disruption).
  // - Only updates elements + files; never touches appState so the current
  //   viewport position and theme are preserved.
  const reloadScene = useCallback((newData: ExcalidrawInitialDataState) => {
    const api = apiRef.current;
    if (!api) return;
    if (api.getAppState().editingElement) return;
    api.updateScene({
      elements: newData.elements,
      files: newData.files ?? {},
    });
  }, []);

  useEffect(() => {
    onReloadReady(reloadScene);
  }, [onReloadReady, reloadScene]);

  /** Serializes the current scene and POSTs it to /data. */
  const doSave = useCallback(async () => {
    const api = apiRef.current;
    if (!api) return;

    const elements = api.getSceneElements();
    const appState = api.getAppState();
    const files = api.getFiles();

    try {
      let body: BodyInit;
      let contentTypeHeader: string;

      if (contentType === "image/svg+xml") {
        const nonDeleted = elements.filter((e) => !e.isDeleted);
        const svg = await exportToSvg({ elements: nonDeleted, appState, files });
        body = svg.outerHTML;
        contentTypeHeader = "image/svg+xml";
      } else if (contentType === "image/png") {
        const nonDeleted = elements.filter((e) => !e.isDeleted);
        const blob = await exportToBlob({
          elements: nonDeleted,
          appState,
          files,
          getDimensions(width: number, height: number) {
            const scale =
              (appState as { exportScale?: number }).exportScale ?? 2;
            return { width: width * scale, height: height * scale, scale };
          },
        });
        if (!blob) return;
        body = await blob.arrayBuffer();
        contentTypeHeader = "image/png";
      } else {
        body = serializeAsJSON(elements, appState, files, "local");
        contentTypeHeader = "application/json";
      }

      const res = await fetch("/data", {
        method: "POST",
        headers: { "Content-Type": contentTypeHeader },
        body,
      });
      if (res.ok) {
        onSaved(Date.now() + 2000);
      }
    } catch {
      // Silently ignore network errors (preview server may have shut down).
    }
  }, [contentType, onSaved]);

  /** Ctrl+S / Cmd+S — always triggers an immediate save. */
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "s") {
        e.preventDefault();
        if (saveTimer.current) {
          clearTimeout(saveTimer.current);
          saveTimer.current = null;
        }
        doSave();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [doSave]);

  /**
   * onChange — only active when autoSave is enabled.
   * Debounces saves so rapid edits collapse into a single write.
   * No-ops on viewport / selection-only events (hash unchanged).
   */
  const handleChange = useCallback(
    (elements: readonly ExcalidrawElement[]) => {
      if (!autoSave) return;
      const currentVersion = hashElementsVersion(elements);
      if (currentVersion === prevVersionRef.current) return;
      prevVersionRef.current = currentVersion;
      if (saveTimer.current) clearTimeout(saveTimer.current);
      saveTimer.current = setTimeout(doSave, SAVE_DEBOUNCE_MS);
    },
    [autoSave, doSave],
  );

  return (
    <div style={{ height: "100%" }}>
      <Excalidraw
        excalidrawAPI={(api) => {
          apiRef.current = api;
          onApiReady(api);
        }}
        initialData={{ ...initialData, scrollToContent: true }}
        theme={resolvedTheme}
        name={name}
        onChange={handleChange}
      >
        <MainMenu>
          <MainMenu.Item onSelect={doSave} shortcut="Ctrl+S">
            Save to file
          </MainMenu.Item>
          <MainMenu.Separator />
          <MainMenu.DefaultItems.LoadScene />
          <MainMenu.DefaultItems.SaveAsImage />
          <MainMenu.DefaultItems.Export />
          <MainMenu.Separator />
          <MainMenu.DefaultItems.ClearCanvas />
          <MainMenu.DefaultItems.ChangeCanvasBackground />
          <MainMenu.DefaultItems.ToggleTheme />
          <MainMenu.Separator />
          <MainMenu.DefaultItems.Help />
        </MainMenu>
      </Excalidraw>
    </div>
  );
}
