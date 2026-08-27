import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts';

interface AlertTrendProps {
  data: Array<{
    date: string;
    critical: number;
    high: number;
    medium: number;
    low: number;
  }>;
}

export function AlertTrendChart({ data }: AlertTrendProps) {
  return (
    <div className="bg-zinc-900/50 border border-white/10 rounded-lg p-4">
      <h3 className="text-sm font-medium text-zinc-300 mb-4">Alert Trend (30 days)</h3>
      <ResponsiveContainer width="100%" height={200}>
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
          <XAxis
            dataKey="date"
            stroke="#71717a"
            fontSize={11}
            tickLine={false}
          />
          <YAxis stroke="#71717a" fontSize={11} tickLine={false} />
          <Tooltip
            contentStyle={{
              backgroundColor: '#18181b',
              border: '1px solid #3f3f46',
              borderRadius: '8px',
            }}
          />
          <Legend />
          <Line
            type="monotone"
            dataKey="critical"
            stroke="#ef4444"
            strokeWidth={2}
            dot={false}
          />
          <Line
            type="monotone"
            dataKey="high"
            stroke="#f97316"
            strokeWidth={2}
            dot={false}
          />
          <Line
            type="monotone"
            dataKey="medium"
            stroke="#eab308"
            strokeWidth={2}
            dot={false}
          />
          <Line
            type="monotone"
            dataKey="low"
            stroke="#71717a"
            strokeWidth={2}
            dot={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
