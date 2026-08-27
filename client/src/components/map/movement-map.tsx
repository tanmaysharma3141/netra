import { useEffect, useRef } from "react"
import L from "leaflet"
import "leaflet/dist/leaflet.css"

/**
 * Pure Leaflet map — no data fetching, fully controlled by props.
 * Tile layer is env-driven so demo machines can point at a bundled offline
 * tile directory instead of the internet (air-gapped requirement).
 */

const TRAIL_COLORS = ["#22d3ee", "#a78bfa", "#34d399", "#f472b6", "#fbbf24"]

export interface MovementMapProps {
  /** One polyline per trail; points must be in chronological order. */
  trails: { entityId: string; label: string; points: { lat: number; lng: number; timestamp: string }[] }[]
  /** Playback index per render: only points up to this global fraction show. 0..1 */
  playbackFraction: number
}

export function MovementMap({ trails, playbackFraction }: MovementMapProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const mapRef = useRef<L.Map | null>(null)
  const layerRef = useRef<L.LayerGroup | null>(null)

  // Init map once.
  useEffect(() => {
    if (!containerRef.current || mapRef.current) return
    const map = L.map(containerRef.current, {
      center: [30.7333, 76.7794],
      zoom: 11,
      zoomControl: true,
      attributionControl: false,
    })
    // Prefer local offline tiles; fall back to online tile providers
    const localTiles = "/tiles/{z}/{x}/{y}.png"
    const envTiles = import.meta.env.VITE_TILE_URL as string | undefined
    const tileUrl = envTiles || localTiles
    L.tileLayer(tileUrl, {
      maxZoom: 19,
      errorTileUrl: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='256' height='256'%3E%3Crect fill='%231a1a2e' width='256' height='256'/%3E%3C/svg%3E",
    }).addTo(map)
    mapRef.current = map
    layerRef.current = L.layerGroup().addTo(map)
    return () => {
      map.remove()
      mapRef.current = null
      layerRef.current = null
    }
  }, [])

  // Redraw trails when data or playback position changes.
  useEffect(() => {
    const group = layerRef.current
    if (!group) return
    group.clearLayers()

    let bounds: L.LatLngBounds | null = null

    trails.forEach((trail, trailIndex) => {
      if (trail.points.length === 0) return
      const color = TRAIL_COLORS[trailIndex % TRAIL_COLORS.length]
      const cutoff = Math.max(2, Math.ceil(trail.points.length * playbackFraction))
      const visible = trail.points.slice(0, cutoff)

      const line = L.polyline(
        visible.map((p) => [p.lat, p.lng] as [number, number]),
        { color, weight: 3, opacity: 0.85 },
      ).addTo(group)

      bounds ??= line.getBounds()

      visible.forEach((point, i) => {
        const isLast = i === visible.length - 1
        L.circleMarker([point.lat, point.lng], {
          radius: isLast ? 6 : 3.5,
          color,
          fillColor: color,
          fillOpacity: isLast ? 1 : 0.7,
          weight: 1.5,
        })
          .bindTooltip(
            `<span style="font-family:var(--font-mono);font-size:10px">${trail.label}<br/>${new Date(point.timestamp).toLocaleString("en-IN")}${isLast ? "<br/>LATEST" : ""}</span>`,
            { direction: "top" },
          )
          .addTo(group)
        bounds ??= L.latLngBounds([point.lat, point.lng], [point.lat, point.lng])
        bounds.extend([point.lat, point.lng])
      })
    })

    if (bounds) {
      mapRef.current?.fitBounds(bounds, { padding: [30, 30], maxZoom: 14 })
    }
  }, [trails, playbackFraction])

  return <div ref={containerRef} className="h-full w-full" aria-label="Movement map" />
}
