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
  onApiReady: (api: ExcalidrawImperativeAPI) => void;
  /** Called after a successful save so the SSE listener can suppress the echo. */
  onSaved: (suppressUntil: number) => void;
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
  onApiReady,
  onSaved,
}: AppProps) {
  const resolvedTheme = useOsTheme(theme as "auto" | "light" | "dark");
  const apiRef = useRef<ExcalidrawImperativeAPI | null>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const prevVersionRef = useRef<number>(-1);

  useEffect(() => {
    if (initialData?.elements) {
      prevVersionRef.current = hashElementsVersion(
        initialData.elements as readonly ExcalidrawElement[],
      );
    }
  }, []);

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

  /** Ctrl+S / Cmd+S — cancel the debounce and save immediately. */
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

  /** onChange — debounced auto-save triggered by element changes. */
  const handleChange = useCallback(
    (elements: readonly ExcalidrawElement[]) => {
      const currentVersion = hashElementsVersion(elements);
      if (currentVersion === prevVersionRef.current) return;
      prevVersionRef.current = currentVersion;

      if (saveTimer.current) clearTimeout(saveTimer.current);
      saveTimer.current = setTimeout(doSave, SAVE_DEBOUNCE_MS);
    },
    [doSave],
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
        UIOptions={{
          canvasActions: {
            loadScene: false,
            saveToActiveFile: false,
            export: false,
          },
        }}
      >
        <MainMenu>
          <MainMenu.Item onSelect={doSave} shortcut="Ctrl+S">
            Save to file
          </MainMenu.Item>
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
