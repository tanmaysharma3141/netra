import { Construction } from "lucide-react"
import { Card, CardContent } from "@/components/ui/card"

interface PlaceholderScreenProps {
  title: string
  description: string
  phase: string
}

export function PlaceholderScreen({ title, description, phase }: PlaceholderScreenProps) {
  return (
    <div className="mx-auto max-w-6xl p-6">
      <header className="mb-6">
        <h1 className="text-lg font-semibold">{title}</h1>
        <p className="text-sm text-muted-foreground">{description}</p>
      </header>
      <Card className="border-dashed">
        <CardContent className="flex flex-col items-center justify-center py-16 text-center">
          <Construction className="mb-3 size-8 text-muted-foreground" aria-hidden />
          <p className="font-mono text-sm">{phase}</p>
          <p className="mt-1 max-w-md text-sm text-muted-foreground">
            This screen ships in a later phase of docs/PLAN_FRONTEND.md.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
