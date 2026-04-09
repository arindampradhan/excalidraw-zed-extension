import { useEffect, useState } from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

interface Config {
  contentType: string;
  name: string;
  theme: string;
}

function showError(message: string) {
  const el = document.getElementById('error');
  if (el) {
    el.textContent = message;
    el.style.display = 'block';
  }
}

function reorderFallbacks(primary: string): string[] {
  const all = ['application/json', 'image/svg+xml', 'image/png'];
  return [primary, ...all.filter((t) => t !== primary)];
}

function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  ms: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), ms);
  };
}

async function main() {
  try {
    const configRes = await fetch('/config');
    if (!configRes.ok) {
      throw new Error(`Failed to fetch config: ${configRes.status}`);
    }
    const config: Config = await configRes.json();

    const dataRes = await fetch('/data');
    if (!dataRes.ok) {
      throw new Error(`Failed to fetch data: ${dataRes.status}`);
    }
    const bytes = await dataRes.arrayBuffer();

    const { loadFromBlob } = await import('@excalidraw/excalidraw');

    let initialData: ExcalidrawInitialDataState | null = null;
    const types = reorderFallbacks(config.contentType);
    for (const type of types) {
      try {
        initialData = await loadFromBlob(
          new Blob([bytes], { type }),
          null,
          null
        );
        break;
      } catch {
        // Try next format
      }
    }

    if (!initialData) {
      showError('Failed to load file: all format fallbacks failed');
      return;
    }

    const root = document.getElementById('root');
    if (!root) return;

    ReactDOM.createRoot(root).render(
      <App
        initialData={initialData}
        viewModeEnabled={true}
        theme={config.theme}
        name={config.name}
      />
    );

    let excalidrawApi: ExcalidrawImperativeAPI | null = null;

    const es = new EventSource('/events');
    es.onmessage = debounce(async () => {
      try {
        const newDataRes = await fetch('/data');
        if (!newDataRes.ok) return;
        const newBytes = await newDataRes.arrayBuffer();

        const newData = await loadFromBlob(
          new Blob([newBytes], { type: config.contentType }),
          null,
          null
        );

        if (excalidrawApi) {
          excalidrawApi.updateScene(newData);
        }
      } catch (e) {
        console.error('Failed to reload:', e);
      }
    }, 150);

  } catch (e) {
    showError(`Error: ${e instanceof Error ? e.message : String(e)}`);
  }
}

main();