import { WS_URL } from "@/lib/env"
import { getToken } from "@/lib/secureStore"

/**
 * WebSocket connection manager — contract: ws://<server>:8420/ws
 *
 * Browsers cannot set custom headers on WebSocket, so we authenticate via the
 * `?token=` fallback the contract provides. Auto-reconnects with capped backoff;
 * re-subscribes active topics after every reconnect.
 */

export interface WsEnvelope {
  topic: string
  event: string
  payload: unknown
}

type Handler = (envelope: WsEnvelope) => void

class WsClient {
  private ws: WebSocket | null = null
  private handlers = new Set<Handler>()
  private topics = new Set<string>()
  private reconnectDelayMs = 1_000
  private connecting = false
  private stopped = false

  start(): void {
    this.stopped = false
    if (this.ws || this.connecting) return
    void this.connect()
  }

  stop(): void {
    this.stopped = true
    this.ws?.close()
    this.ws = null
  }

  /** Subscribe to topics; returns an unsubscribe function for the handler. */
  subscribe(topics: readonly string[], handler: Handler): () => void {
    const fresh = topics.filter((t) => !this.topics.has(t))
    for (const topic of topics) {
      this.topics.add(topic)
      this.handlers.add(handler)
    }
    if (this.isOpen() && fresh.length > 0) this.sendSubscribe()
    return () => {
      this.handlers.delete(handler)
      // Topic-level unsubscribe is skipped intentionally: other consumers may
      // still need the frames; idle topics are harmless server-side.
    }
  }

  private isOpen(): boolean {
    return this.ws?.readyState === WebSocket.OPEN
  }

  private async connect(): Promise<void> {
    if (this.connecting || this.stopped) return
    this.connecting = true
    try {
      const token = await getToken()
      const url = token ? `${WS_URL}?token=${encodeURIComponent(token)}` : WS_URL
      const ws = new WebSocket(url)
      this.ws = ws

      ws.onopen = () => {
        this.reconnectDelayMs = 1_000
        this.connecting = false
        this.sendSubscribe()
      }

      ws.onmessage = (msg: MessageEvent<string>) => {
        try {
          const parsed: unknown = JSON.parse(msg.data)
          if (
            typeof parsed === "object" &&
            parsed !== null &&
            "topic" in parsed &&
            "event" in parsed
          ) {
            const envelope = parsed as WsEnvelope
            for (const handler of this.handlers) handler(envelope)
          }
        } catch {
          // Malformed frame — ignore rather than kill the socket.
        }
      }

      ws.onclose = () => {
        this.connecting = false
        if (this.stopped) return
        window.setTimeout(() => void this.connect(), this.reconnectDelayMs)
        this.reconnectDelayMs = Math.min(this.reconnectDelayMs * 2, 15_000)
      }

      ws.onerror = () => ws.close()
    } catch {
      this.connecting = false
      window.setTimeout(() => void this.connect(), this.reconnectDelayMs)
    }
  }

  private sendSubscribe(): void {
    if (!this.isOpen() || this.topics.size === 0) return
    this.ws?.send(JSON.stringify({ type: "subscribe", topics: [...this.topics] }))
  }
}

export const wsClient = new WsClient()
