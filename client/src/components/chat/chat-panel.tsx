import { useRef, useState } from "react"
import { toast } from "sonner"
import { Bot, Send, User, RefreshCw } from "lucide-react"
import { API_BASE_URL } from "@/lib/env"
import { getToken } from "@/lib/secureStore"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"

interface ChatMessage {
  role: "user" | "assistant"
  content: string
  sources?: string[]
}

export function ChatPanel({ caseId }: { caseId: string }) {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [input, setInput] = useState("")
  const [isStreaming, setIsStreaming] = useState(false)
  const scrollRef = useRef<HTMLDivElement>(null)

  async function sendMessage() {
    const question = input.trim()
    if (!question || isStreaming) return

    setInput("")
    setMessages((prev) => [...prev, { role: "user", content: question }])
    setIsStreaming(true)

    // Add placeholder for assistant response
    const assistantIndex = messages.length + 1
    setMessages((prev) => [...prev, { role: "assistant", content: "" }])

    try {
      const token = await getToken()
      const res = await fetch(`${API_BASE_URL}/cases/${encodeURIComponent(caseId)}/chat`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify({ question }),
      })

      if (!res.ok) {
        throw new Error(`HTTP ${res.status}`)
      }

      const reader = res.body?.getReader()
      if (!reader) throw new Error("No response body")

      const decoder = new TextDecoder()
      let buffer = ""
      let sources: string[] = []

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })

        // Process SSE lines
        const lines = buffer.split("\n")
        buffer = lines.pop() ?? ""

        for (const line of lines) {
          const trimmed = line.trim()
          if (!trimmed || !trimmed.startsWith("data: ")) continue

          const jsonStr = trimmed.slice(6)
          try {
            const frame = JSON.parse(jsonStr)

            if (frame.delta) {
              setMessages((prev) => {
                const updated = [...prev]
                const last = updated[assistantIndex]
                if (last) {
                  updated[assistantIndex] = {
                    ...last,
                    content: last.content + frame.delta,
                  }
                }
                return updated
              })
            }

            if (frame.sources) {
              sources = frame.sources
              setMessages((prev) => {
                const updated = [...prev]
                const last = updated[assistantIndex]
                if (last) {
                  updated[assistantIndex] = { ...last, sources }
                }
                return updated
              })
            }

            if (frame.done) {
              // Stream complete
            }
          } catch {
            // Skip malformed frames
          }
        }
      }

      // Scroll to bottom
      setTimeout(() => {
        scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: "smooth" })
      }, 50)
    } catch (err) {
      toast.error("Chat request failed", {
        description: err instanceof Error ? err.message : "Unexpected error.",
      })
      // Remove the empty assistant message on error
      setMessages((prev) => prev.slice(0, -1))
    } finally {
      setIsStreaming(false)
    }
  }

  return (
    <div className="flex h-[calc(100vh-22rem)] min-h-96 flex-col">
      {/* Messages */}
      <div ref={scrollRef} className="min-h-0 flex-1 overflow-y-auto space-y-3 p-1">
        {messages.length === 0 ? (
          <div className="flex h-full flex-col items-center justify-center text-center">
            <Bot className="mb-3 size-8 text-muted-foreground" aria-hidden />
            <p className="text-sm font-medium">Case Copilot</p>
            <p className="mt-1 max-w-sm text-xs text-muted-foreground">
              Ask questions about this case — the AI will search events, entities,
              and alerts to answer.
            </p>
          </div>
        ) : (
          messages.map((msg, i) => (
            <div
              key={i}
              className={`flex gap-2 ${msg.role === "user" ? "justify-end" : "justify-start"}`}
            >
              {msg.role === "assistant" ? (
                <div className="bg-secondary flex size-7 shrink-0 items-center justify-center rounded-sm">
                  <Bot className="size-4 text-muted-foreground" aria-hidden />
                </div>
              ) : null}
              <div
                className={`max-w-[80%] rounded-sm px-3 py-2 text-sm ${
                  msg.role === "user"
                    ? "bg-primary text-primary-foreground"
                    : "bg-secondary"
                }`}
              >
                <p className="whitespace-pre-wrap">{msg.content || (isStreaming && i === messages.length - 1 ? "…" : "")}</p>
                {msg.sources && msg.sources.length > 0 ? (
                  <div className="mt-2 flex flex-wrap gap-1">
                    <span className="font-mono text-[9px] text-muted-foreground">Sources:</span>
                    {msg.sources.map((src) => (
                      <Badge
                        key={src}
                        variant="outline"
                        className="font-mono text-[9px]"
                      >
                        {src.slice(0, 8)}
                      </Badge>
                    ))}
                  </div>
                ) : null}
              </div>
              {msg.role === "user" ? (
                <div className="bg-primary flex size-7 shrink-0 items-center justify-center rounded-sm">
                  <User className="size-4 text-primary-foreground" aria-hidden />
                </div>
              ) : null}
            </div>
          ))
        )}
      </div>

      {/* Input */}
      <div className="border-border mt-3 flex gap-2 border-t pt-3">
        <Input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault()
              void sendMessage()
            }
          }}
          placeholder="Ask about this case…"
          disabled={isStreaming}
          className="flex-1"
          aria-label="Chat message"
        />
        <Button
          size="icon"
          onClick={() => void sendMessage()}
          disabled={!input.trim() || isStreaming}
        >
          {isStreaming ? (
            <RefreshCw className="size-4 animate-spin" aria-hidden />
          ) : (
            <Send className="size-4" aria-hidden />
          )}
        </Button>
      </div>
    </div>
  )
}
