import { useRef, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { toast } from "sonner"
import { Bot, ExternalLink, Send, User, RefreshCw } from "lucide-react"
import { API_BASE_URL } from "@/lib/env"
import { getToken } from "@/lib/secureStore"
import { apiFetch } from "@/api/client"
import type { Event } from "@/api/types"
import { SOURCE_LABELS } from "@/lib/severity"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import { Skeleton } from "@/components/ui/skeleton"

interface ChatMessage {
  role: "user" | "assistant"
  content: string
  sources?: string[]
}

const SUGGESTED_PROMPTS = [
  "What are the most suspicious entities in this case?",
  "Show me all calls between the top 3 entities",
  "Summarize the alert patterns found",
  "What timeline anomalies exist?",
]

export function ChatPanel({ caseId }: { caseId: string }) {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [input, setInput] = useState("")
  const [isStreaming, setIsStreaming] = useState(false)
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null)
  const scrollRef = useRef<HTMLDivElement>(null)

  function scrollToBottom() {
    setTimeout(() => {
      scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: "smooth" })
    }, 50)
  }

  async function sendMessage(question?: string) {
    const text = (question ?? input).trim()
    if (!text || isStreaming) return

    setInput("")
    setMessages((prev) => [...prev, { role: "user", content: text }])
    setIsStreaming(true)

    const assistantIndex = messages.length + 1
    setMessages((prev) => [...prev, { role: "assistant", content: "" }])
    scrollToBottom()

    try {
      const token = await getToken()
      const res = await fetch(`${API_BASE_URL}/cases/${encodeURIComponent(caseId)}/chat`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify({ question: text }),
      })

      if (!res.ok) throw new Error(`HTTP ${res.status}`)

      const reader = res.body?.getReader()
      if (!reader) throw new Error("No response body")

      const decoder = new TextDecoder()
      let buffer = ""

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split("\n")
        buffer = lines.pop() ?? ""

        for (const line of lines) {
          const trimmed = line.trim()
          if (!trimmed || !trimmed.startsWith("data: ")) continue

          try {
            const frame = JSON.parse(trimmed.slice(6))

            if (frame.delta) {
              setMessages((prev) => {
                const updated = [...prev]
                const last = updated[assistantIndex]
                if (last) {
                  updated[assistantIndex] = { ...last, content: last.content + frame.delta }
                }
                return updated
              })
              scrollToBottom()
            }

            if (frame.sources) {
              setMessages((prev) => {
                const updated = [...prev]
                const last = updated[assistantIndex]
                if (last) {
                  updated[assistantIndex] = { ...last, sources: frame.sources }
                }
                return updated
              })
            }
          } catch {
            // Skip malformed frames
          }
        }
      }
    } catch (err) {
      toast.error("Chat request failed", {
        description: err instanceof Error ? err.message : "Unexpected error.",
      })
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
            <div className="mt-4 flex max-w-md flex-wrap justify-center gap-2">
              {SUGGESTED_PROMPTS.map((prompt) => (
                <Button
                  key={prompt}
                  variant="outline"
                  size="sm"
                  className="h-auto max-w-[200px] text-wrap text-left text-xs"
                  onClick={() => void sendMessage(prompt)}
                  disabled={isStreaming}
                >
                  {prompt}
                </Button>
              ))}
            </div>
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
                <p className="whitespace-pre-wrap">
                  {msg.content || (isStreaming && i === messages.length - 1 ? (
                    <span className="inline-flex gap-0.5">
                      <span className="animate-pulse">●</span>
                      <span className="animate-pulse [animation-delay:0.2s]">●</span>
                      <span className="animate-pulse [animation-delay:0.4s]">●</span>
                    </span>
                  ) : "")}
                </p>
                {msg.sources && msg.sources.length > 0 ? (
                  <div className="mt-2 flex flex-wrap gap-1">
                    <span className="font-mono text-[9px] text-muted-foreground">Sources:</span>
                    {msg.sources.map((src) => (
                      <button
                        key={src}
                        onClick={() => setSelectedEventId(src)}
                        className="inline-flex items-center gap-0.5 font-mono text-[9px] text-chart-1 underline-offset-2 hover:underline"
                      >
                        {src.slice(0, 8)}
                        <ExternalLink className="size-2.5" aria-hidden />
                      </button>
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

      {/* Event detail sheet for source citations */}
      <EventDetailSheet
        eventId={selectedEventId}
        onClose={() => setSelectedEventId(null)}
      />
    </div>
  )
}

function EventDetailSheet({
  eventId,
  onClose,
}: {
  eventId: string | null
  onClose: () => void
}) {
  const eventQuery = useQuery({
    queryKey: ["event", eventId],
    queryFn: () => apiFetch<Event>(`/events/${encodeURIComponent(eventId!)}`),
    enabled: eventId !== null,
  })

  return (
    <Sheet open={eventId !== null} onOpenChange={(open) => !open && onClose()}>
      <SheetContent className="w-full overflow-y-auto sm:max-w-lg">
        {eventQuery.isPending ? (
          <div className="space-y-3">
            <Skeleton className="h-6 w-48" />
            <Skeleton className="h-4 w-32" />
            <Skeleton className="h-32 w-full" />
          </div>
        ) : eventQuery.isError ? (
          <div className="text-sm text-muted-foreground">
            Failed to load event.
          </div>
        ) : eventQuery.data ? (
          <>
            <SheetHeader>
              <SheetTitle className="font-mono text-sm">
                {eventQuery.data.event_type} · {eventQuery.data.entity_type}
              </SheetTitle>
              <SheetDescription className="font-mono text-xs">
                {eventQuery.data.id}
              </SheetDescription>
            </SheetHeader>
            <dl className="mt-4 space-y-2">
              <Field
                label="Timestamp"
                value={new Date(eventQuery.data.timestamp).toLocaleString("en-IN")}
                mono
              />
              <Field label="Entity" value={`${eventQuery.data.entity_type} — ${eventQuery.data.entity_id}`} mono />
              <Field label="Source" value={SOURCE_LABELS[eventQuery.data.source_type]} />
              {eventQuery.data.value !== null ? (
                <Field label="Value" value={eventQuery.data.value.toLocaleString("en-IN")} mono />
              ) : null}
              {eventQuery.data.location ? (
                <Field
                  label="Location"
                  value={`${eventQuery.data.location.lat.toFixed(5)}, ${eventQuery.data.location.lng.toFixed(5)}`}
                  mono
                />
              ) : null}
            </dl>
            <div className="mt-4">
              <p className="mb-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                Raw record
              </p>
              <pre className="border-border bg-card max-h-60 overflow-auto rounded-sm border p-3 font-mono text-[11px] leading-relaxed">
                {JSON.stringify(eventQuery.data.raw, null, 2)}
              </pre>
            </div>
          </>
        ) : null}
      </SheetContent>
    </Sheet>
  )
}

function Field({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="flex gap-3">
      <dt className="text-muted-foreground w-24 shrink-0 text-xs">{label}</dt>
      <dd className={`min-w-0 break-all text-xs ${mono ? "font-mono" : ""}`}>{value}</dd>
    </div>
  )
}
