import { useState, useEffect, useCallback, useRef } from "react";
import { Excalidraw, serializeAsJSON } from "@excalidraw/excalidraw";
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
  // Only JSON files support round-trip editing; SVG/PNG are view-only.
  const editable = contentType === "application/json";

  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleChange = useCallback(
    (
      elements: readonly ExcalidrawElement[],
      appState: AppState,
      files: BinaryFiles,
    ) => {
      if (!editable) return;
      if (saveTimer.current) clearTimeout(saveTimer.current);
      saveTimer.current = setTimeout(async () => {
        const json = serializeAsJSON(elements, appState, files, "local");
        try {
          const res = await fetch("/data", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: json,
          });
          if (res.ok) {
            // Suppress the SSE echo from the file-watcher for 2 seconds.
            onSaved(Date.now() + 2000);
          }
        } catch {
          // Silently ignore network errors (preview server may have shut down).
        }
      }, SAVE_DEBOUNCE_MS);
    },
    [editable, onSaved],
  );

  return (
    <div style={{ height: "100%" }}>
      <Excalidraw
        excalidrawAPI={onApiReady}
        initialData={{ ...initialData, scrollToContent: true }}
        viewModeEnabled={!editable}
        theme={resolvedTheme}
        name={name}
        onChange={editable ? handleChange : undefined}
        UIOptions={{
          canvasActions: {
            loadScene: false,
            saveToActiveFile: false,
            export: false,
          },
        }}
      />
    </div>
  );
}
