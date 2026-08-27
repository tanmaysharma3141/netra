import { useEffect, useRef } from "react"
import * as d3 from "d3"
import type { GraphEdge, GraphNode, EntityType } from "@/api/types"

/**
 * Pure D3 force-directed graph — no data fetching, fully controlled by props.
 * Node color by EntityType, edge width by evidence_count, dashed edges for
 * non-high tiers, drag + zoom + hover-neighbor highlight.
 */

export interface ForceGraphNode extends GraphNode, d3.SimulationNodeDatum {}

export interface ForceGraphProps {
  nodes: ForceGraphNode[]
  edges: GraphEdge[]
  selectedId: string | null
  onNodeClick?: (id: string) => void
}

export const ENTITY_COLORS: Record<EntityType, string> = {
  PHONE: "var(--color-chart-1)",
  IMEI: "var(--color-chart-5)",
  BANK_ACC: "var(--color-chart-3)",
  IP: "var(--color-chart-2)",
  HANDLE: "var(--color-chart-4)",
}

export const ENTITY_LABELS: Record<EntityType, string> = {
  PHONE: "Phone",
  IMEI: "Device",
  BANK_ACC: "Bank A/C",
  IP: "IP",
  HANDLE: "Handle",
}

interface SimLink extends d3.SimulationLinkDatum<ForceGraphNode> {
  // d3 mutates source/target from ids into node refs during simulation.
  source: ForceGraphNode | string | number
  target: ForceGraphNode | string | number
  edge: GraphEdge
}

function refId(ref: ForceGraphNode | string | number): string {
  return typeof ref === "string" ? ref : (ref as ForceGraphNode).id
}

function nodeRadius(type: EntityType, degree: number): number {
  const base = type === "IMEI" ? 8 : type === "PHONE" ? 7 : 6
  return base + Math.min(6, Math.log2(degree + 1))
}

export function ForceGraph({ nodes, edges, selectedId, onNodeClick }: ForceGraphProps) {
  const containerRef = useRef<HTMLDivElement>(null)

  // Store refs that persist across renders without restarting the simulation
  const simRef = useRef<d3.Simulation<ForceGraphNode, SimLink> | null>(null)
  const nodeSelRef = useRef<d3.Selection<SVGGElement, ForceGraphNode, SVGGElement, unknown> | null>(null)
  const edgeSelRef = useRef<d3.Selection<SVGLineElement, SimLink, SVGGElement, unknown> | null>(null)
  const adjacencyRef = useRef<Map<string, Set<string>>>(new Map())
  const dataKeyRef = useRef<string>("")

  // Initialize / rebuild simulation ONLY when nodes or edges actually change
  useEffect(() => {
    const container = containerRef.current
    if (!container || nodes.length === 0) return

    const dataKey = `${nodes.length}-${edges.length}-${nodes[0]?.id ?? ""}`
    if (dataKey === dataKeyRef.current) return
    dataKeyRef.current = dataKey

    // Clean up old simulation
    if (simRef.current) simRef.current.stop()

    const { width, height } = container.getBoundingClientRect()

    const svg = d3
      .select(container)
      .selectAll<SVGSVGElement, null>("svg")
      .data([null])
      .join("svg")
      .attr("width", width)
      .attr("height", height)
      .attr("role", "img")
      .attr("aria-label", "Entity relationship graph")

    const root = svg
      .selectAll<SVGGElement, unknown>("g.viewport")
      .data([null])
      .join("g")
      .attr("class", "viewport")

    svg
      .call(
        d3
          .zoom<SVGSVGElement, null>()
          .scaleExtent([0.15, 4])
          .on("zoom", (event) => {
            root.attr("transform", event.transform.toString())
          }),
      )
      .on("dblclick.zoom", null)

    const degreeById = new Map<string, number>()
    for (const edge of edges) {
      degreeById.set(edge.source, (degreeById.get(edge.source) ?? 0) + 1)
      degreeById.set(edge.target, (degreeById.get(edge.target) ?? 0) + 1)
    }

    const simNodes: ForceGraphNode[] = nodes.map((n) => ({ ...n }))
    const simLinks: SimLink[] = edges.map((edge) => ({
      source: edge.source,
      target: edge.target,
      edge,
    }))

    const simulation = d3
      .forceSimulation<ForceGraphNode>(simNodes)
      .force(
        "link",
        d3
          .forceLink<ForceGraphNode, SimLink>(simLinks)
          .id((n) => n.id)
          .distance((l) => 90 - Math.min(50, Math.log2(l.edge.evidence_count + 1) * 8))
          .strength(0.5),
      )
      .force("charge", d3.forceManyBody().strength(-220))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force(
        "collide",
        d3.forceCollide<ForceGraphNode>((n) => nodeRadius(n.type, degreeById.get(n.id) ?? 0) + 4),
      )
      .alphaDecay(0.03)  // Slower decay = smoother settling
      .velocityDecay(0.4) // More friction = less jitter

    const adjacency = new Map<string, Set<string>>()
    for (const link of simLinks) {
      const s = refId(link.source)
      const t = refId(link.target)
      if (!adjacency.has(s)) adjacency.set(s, new Set())
      if (!adjacency.has(t)) adjacency.set(t, new Set())
      adjacency.get(s)!.add(t)
      adjacency.get(t)!.add(s)
    }
    adjacencyRef.current = adjacency

    const edgeSel = root
      .selectAll<SVGLineElement, SimLink>("line.edge")
      .data(simLinks)
      .join("line")
      .attr("class", "edge")
      .attr("stroke", "var(--color-border)")
      .attr("stroke-width", (l) => Math.max(1, Math.min(5, 1 + Math.log2(l.edge.evidence_count + 1) * 0.8)))
      .attr("stroke-dasharray", (l) => (l.edge.tier === "high" ? "" : "4 3"))
      .attr("stroke-opacity", 0.55)
    edgeSelRef.current = edgeSel

    const nodeSel = root
      .selectAll<SVGGElement, ForceGraphNode>("g.node")
      .data(simNodes, (n) => n.id)
      .join("g")
      .attr("class", "node cursor-pointer")
    nodeSelRef.current = nodeSel

    nodeSel.selectAll("circle").remove()
    nodeSel
      .append("circle")
      .attr("r", (n) => nodeRadius(n.type, degreeById.get(n.id) ?? 0))
      .attr("fill", (n) => ENTITY_COLORS[n.type] ?? "var(--color-muted)")
      .attr("fill-opacity", 0.9)
      .attr("stroke", "var(--color-background)")
      .attr("stroke-width", 1.5)

    nodeSel.selectAll("text").remove()
    nodeSel
      .filter((n) => (degreeById.get(n.id) ?? 0) >= 3)
      .append("text")
      .text((n) => (n.label.length > 18 ? `${n.label.slice(0, 17)}…` : n.label))
      .attr("text-anchor", "middle")
      .attr("y", (n) => nodeRadius(n.type, degreeById.get(n.id) ?? 0) + 12)
      .attr("fill", "var(--color-muted-foreground)")
      .attr("font-size", 10)
      .attr("font-family", "var(--font-mono)")
      .attr("pointer-events", "none")

    nodeSel.on("click", (event: MouseEvent, n) => {
      event.stopPropagation()
      onNodeClick?.(n.id)
    })

    nodeSel.call(
      d3
        .drag<SVGGElement, ForceGraphNode>()
        .on("start", (event, n) => {
          if (!event.active) simulation.alphaTarget(0.25).restart()
          n.fx = n.x
          n.fy = n.y
        })
        .on("drag", (event, n) => {
          n.fx = event.x
          n.fy = event.y
        })
        .on("end", (event, n) => {
          if (!event.active) simulation.alphaTarget(0)
          n.fx = null
          n.fy = null
        }),
    )

    nodeSel.on("mouseenter", (_event: unknown, n) => {
      const neighbors = adjacency.get(n.id) ?? new Set<string>()
      nodeSel
        .attr("opacity", (d) => (d.id === n.id || neighbors.has(d.id)) ? 1 : 0.2)
        .select("circle")
        .attr("stroke", (d) => (d.id === n.id) ? "var(--color-primary)" : "var(--color-background)")
      edgeSel.attr("stroke-opacity", (l) => {
        const s = refId(l.source)
        const t = refId(l.target)
        return s === n.id || t === n.id ? 0.95 : 0.06
      })
    })
    nodeSel.on("mouseleave", () => {
      nodeSel
        .attr("opacity", 1)
        .select("circle")
        .attr("stroke", "var(--color-background)")
      edgeSel.attr("stroke-opacity", 0.55)
    })

    simulation.on("tick", () => {
      edgeSel
        .attr("x1", (l: SimLink) => (l.source as ForceGraphNode).x ?? 0)
        .attr("y1", (l: SimLink) => (l.source as ForceGraphNode).y ?? 0)
        .attr("x2", (l: SimLink) => (l.target as ForceGraphNode).x ?? 0)
        .attr("y2", (l: SimLink) => (l.target as ForceGraphNode).y ?? 0)
      nodeSel.attr("transform", (n) => `translate(${n.x ?? 0},${n.y ?? 0})`)
    })

    simRef.current = simulation

    return () => {
      simulation.stop()
      dataKeyRef.current = ""
    }
  }, [nodes, edges, onNodeClick])

  // Selection highlight — runs separately, does NOT restart simulation
  useEffect(() => {
    const nodeSel = nodeSelRef.current
    const edgeSel = edgeSelRef.current
    const adjacency = adjacencyRef.current
    if (!nodeSel || !edgeSel) return

    if (!selectedId) {
      nodeSel.attr("opacity", 1).select("circle").attr("stroke", "var(--color-background)")
      edgeSel.attr("stroke-opacity", 0.55)
      return
    }

    const neighbors = adjacency.get(selectedId) ?? new Set<string>()
    nodeSel
      .attr("opacity", (n) => (n.id === selectedId || neighbors.has(n.id)) ? 1 : 0.2)
      .select("circle")
      .attr("stroke", (n) => (n.id === selectedId) ? "var(--color-primary)" : "var(--color-background)")
    edgeSel.attr("stroke-opacity", (l) => {
      const s = refId(l.source)
      const t = refId(l.target)
      return s === selectedId || t === selectedId ? 0.95 : 0.06
    })
  }, [selectedId])

  return <div ref={containerRef} className="h-full w-full" />
}
