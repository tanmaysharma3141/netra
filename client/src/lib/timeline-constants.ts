import type { EventType, SourceType } from "@/api/types"

export const SOURCE_TYPES: readonly SourceType[] = ["CDR", "IPDR", "BANK", "SOCIAL"]

export const EVENT_TYPES: readonly EventType[] = [
  "CALL",
  "SMS",
  "DATA",
  "TXN",
  "POST",
  "LOGIN",
  "OTHER",
]
