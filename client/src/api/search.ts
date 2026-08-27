export interface SearchResult {
  result_type: 'entity' | 'alert' | 'case';
  case_id: string;
  case_title: string;
  identifier: string;
  detail: Record<string, unknown>;
}

export interface SearchResults {
  results: SearchResult[];
  total: number;
}

export async function searchAll(
  query: string,
  type?: string,
  limit = 50
): Promise<SearchResults> {
  const token = localStorage.getItem('netra_token');
  const params = new URLSearchParams({ q: query, limit: String(limit) });
  if (type) params.set('search_type', type);

  const res = await fetch(`/api/v1/search?${params}`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!res.ok) throw new Error(`Search failed: ${res.status}`);
  return res.json();
}
