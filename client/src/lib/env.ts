/** REST base — contract: http://<server>:8420/api/v1 */
export const API_BASE_URL: string =
  import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8420/api/v1"

/** WebSocket endpoint — contract: ws://<server>:8420/ws */
export const WS_URL: string = import.meta.env.VITE_WS_BASE_URL ?? "ws://localhost:8420/ws"
