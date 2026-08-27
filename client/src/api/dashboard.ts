import { Alert } from './types';

export interface DashboardStats {
  total_cases: number;
  active_cases: number;
  alerts_by_severity: {
    critical: number;
    high: number;
    medium: number;
    low: number;
  };
  recent_alerts: Alert[];
  events_this_week: number;
  entities_count: number;
}

export async function getDashboard(): Promise<DashboardStats> {
  const token = localStorage.getItem('netra_token');
  const res = await fetch('/api/v1/dashboard', {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!res.ok) throw new Error(`Dashboard fetch failed: ${res.status}`);
  return res.json();
}
