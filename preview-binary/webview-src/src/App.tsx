import { useState, useEffect, useCallback, useRef } from "react";
import {
  Excalidraw,
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

  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  // Track the element hash of the last-saved state so we only POST when
  // elements actually change (not on viewport pan / zoom / selection).
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

  const handleChange = useCallback(
    (
      elements: readonly ExcalidrawElement[],
      appState: AppState,
      files: BinaryFiles,
    ) => {
      // Only proceed when elements actually changed — skip viewport/selection-only events.
      const currentVersion = hashElementsVersion(elements);
      if (currentVersion === prevVersionRef.current) return;
      prevVersionRef.current = currentVersion;

      if (saveTimer.current) clearTimeout(saveTimer.current);
      saveTimer.current = setTimeout(async () => {
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
                const scale = (appState as { exportScale?: number }).exportScale ?? 2;
                return { width: width * scale, height: height * scale, scale };
              },
            });
            if (!blob) return;
            body = await blob.arrayBuffer();
            contentTypeHeader = "image/png";
          } else {
            // application/json — default Excalidraw format
            body = serializeAsJSON(elements, appState, files, "local");
            contentTypeHeader = "application/json";
          }

          const res = await fetch("/data", {
            method: "POST",
            headers: { "Content-Type": contentTypeHeader },
            body,
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
    [contentType, onSaved],
  );

  return (
    <div style={{ height: "100%" }}>
      <Excalidraw
        excalidrawAPI={onApiReady}
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
      />
    </div>
  );
}
