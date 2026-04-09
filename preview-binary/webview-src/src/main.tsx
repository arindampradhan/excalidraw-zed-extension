import ReactDOM from "react-dom/client";
import App from "./App";

interface Config {
  contentType: string;
  name: string;
  theme: string;
}

function showError(message: string) {
  const el = document.getElementById("error");
  if (el) {
    el.textContent = message;
    el.style.display = "block";
  }
}

function reorderFallbacks(primary: string): string[] {
  const all = ["application/json", "image/svg+xml", "image/png"];
  return [primary, ...all.filter((t) => t !== primary)];
}

function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  ms: number,
): (...args: Parameters<T>) => void {
  let id: ReturnType<typeof setTimeout>;
  return (...args) => {
    clearTimeout(id);
    id = setTimeout(() => fn(...args), ms);
  };
}

async function main() {
  try {
    const configRes = await fetch("/config");
    if (!configRes.ok)
      throw new Error(`Failed to fetch config: ${configRes.status}`);
    const config: Config = await configRes.json();

    const dataRes = await fetch("/data");
    if (!dataRes.ok) throw new Error(`Failed to fetch data: ${dataRes.status}`);
    const bytes = await dataRes.arrayBuffer();

    const { loadFromBlob } = await import("@excalidraw/excalidraw");

    let initialData: ExcalidrawInitialDataState | null = null;
    for (const type of reorderFallbacks(config.contentType)) {
      try {
        initialData = await loadFromBlob(
          new Blob([bytes], { type }),
          null,
          null,
        );
        break;
      } catch {
        // try next format
      }
    }

    if (!initialData) {
      showError("Failed to load file: all format fallbacks failed");
      return;
    }

    // Shared mutable state between App callbacks and the SSE handler.
    let excalidrawApi: ExcalidrawImperativeAPI | null = null;
    // Timestamp after which SSE reload events are no longer suppressed.
    let ignoreSseUntil = 0;

    const root = document.getElementById("root");
    if (!root) return;

    ReactDOM.createRoot(root).render(
      <App
        initialData={initialData}
        theme={config.theme}
        name={config.name}
        contentType={config.contentType}
        onApiReady={(api) => {
          excalidrawApi = api;
        }}
        onSaved={(until) => {
          ignoreSseUntil = until;
        }}
      />,
    );

    // SSE live-reload: triggered by external file changes (e.g. edits in Zed).
    // Suppressed for 2 s after the WebView itself POSTs a save to avoid echo.
    const es = new EventSource("/events");
    es.onmessage = debounce(async () => {
      if (Date.now() < ignoreSseUntil) return;
      try {
        const res = await fetch("/data");
        if (!res.ok) return;
        const newBytes = await res.arrayBuffer();
        const newData = await loadFromBlob(
          new Blob([newBytes], { type: config.contentType }),
          null,
          null,
        );
        excalidrawApi?.updateScene(newData);
      } catch (e) {
        console.error("Failed to reload:", e);
      }
    }, 150);
  } catch (e) {
    showError(`Error: ${e instanceof Error ? e.message : String(e)}`);
  }
}

main();
