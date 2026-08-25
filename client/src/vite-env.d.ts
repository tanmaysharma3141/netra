/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** REST base, e.g. http://<server>:8420/api/v1 */
  readonly VITE_API_BASE_URL?: string
  /** WebSocket endpoint, e.g. ws://<server>:8420/ws */
  readonly VITE_WS_BASE_URL?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
