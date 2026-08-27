import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip, Legend } from 'recharts';

interface SourceBreakdownProps {
  data: Array<{
    name: string;
    value: number;
  }>;
}

const COLORS = ['#3b82f6', '#22c55e', '#a855f7', '#f97316'];

export function SourceBreakdownChart({ data }: SourceBreakdownProps) {
  return (
    <div className="bg-zinc-900/50 border border-white/10 rounded-lg p-4">
      <h3 className="text-sm font-medium text-zinc-300 mb-4">Events by Source</h3>
      <ResponsiveContainer width="100%" height={200}>
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            innerRadius={50}
            outerRadius={80}
            paddingAngle={4}
            dataKey="value"
          >
            {data.map((_, index) => (
              <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
            ))}
          </Pie>
          <Tooltip
            contentStyle={{
              backgroundColor: '#18181b',
              border: '1px solid #3f3f46',
              borderRadius: '8px',
            }}
          />
          <Legend />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
}
