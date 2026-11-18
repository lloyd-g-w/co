interface ImportMetaEnv {
  /** WebSocket URL of the co server, e.g. wss://co.example.com/ws */
  readonly VITE_CO_SERVER_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
