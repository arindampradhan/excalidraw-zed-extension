import { useState, useEffect } from 'react';
import { Excalidraw } from '@excalidraw/excalidraw';
import '@excalidraw/excalidraw/index.css';

interface AppProps {
  initialData: ExcalidrawInitialDataState;
  viewModeEnabled: boolean;
  theme: string;
  name: string;
}

function useOsTheme(preference: 'auto' | 'light' | 'dark'): 'light' | 'dark' {
  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    if (preference !== 'auto') return preference;
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  });

  useEffect(() => {
    if (preference !== 'auto') return;

    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (e: MediaQueryListEvent) => setTheme(e.matches ? 'dark' : 'light');
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, [preference]);

  return theme;
}

export default function App({ initialData, viewModeEnabled, theme, name }: AppProps) {
  const [api, setApi] = useState<ExcalidrawImperativeAPI | null>(null);
  const resolvedTheme = useOsTheme(theme as 'auto' | 'light' | 'dark');

  return (
    <div style={{ height: '100%' }}>
      <Excalidraw
        excalidrawAPI={setApi}
        initialData={{ ...initialData, scrollToContent: true }}
        viewModeEnabled={viewModeEnabled}
        theme={resolvedTheme}
        name={name}
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