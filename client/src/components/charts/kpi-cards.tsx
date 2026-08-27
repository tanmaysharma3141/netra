import { AlertTriangle, FileText, Users, Activity } from 'lucide-react';

interface KpiCardsProps {
  totalCases: number;
  activeCases: number;
  alertsCount: number;
  entitiesCount: number;
}

export function KpiCards({
  totalCases,
  activeCases,
  alertsCount,
  entitiesCount,
}: KpiCardsProps) {
  const cards = [
    {
      label: 'Total Cases',
      value: totalCases,
      icon: FileText,
      color: 'text-blue-400',
      bg: 'bg-blue-500/10',
    },
    {
      label: 'Active Cases',
      value: activeCases,
      icon: Activity,
      color: 'text-green-400',
      bg: 'bg-green-500/10',
    },
    {
      label: 'Open Alerts',
      value: alertsCount,
      icon: AlertTriangle,
      color: 'text-orange-400',
      bg: 'bg-orange-500/10',
    },
    {
      label: 'Entities',
      value: entitiesCount,
      icon: Users,
      color: 'text-purple-400',
      bg: 'bg-purple-500/10',
    },
  ];

  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
      {cards.map((card) => (
        <div
          key={card.label}
          className={`${card.bg} border border-white/10 rounded-lg p-4`}
        >
          <div className="flex items-center gap-3">
            <card.icon className={`${card.color} h-5 w-5`} />
            <span className="text-sm text-zinc-400">{card.label}</span>
          </div>
          <div className="mt-2 text-2xl font-bold text-white">
            {card.value.toLocaleString()}
          </div>
        </div>
      ))}
    </div>
  );
}
