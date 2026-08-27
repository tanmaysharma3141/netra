import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Search as SearchIcon, AlertTriangle, FileText, User, ArrowRight } from 'lucide-react';
import { searchAll, SearchResult } from '@/api/search';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Skeleton } from '@/components/ui/skeleton';

const TYPE_ICONS: Record<string, typeof SearchIcon> = {
  entity: User,
  alert: AlertTriangle,
  case: FileText,
};

const TYPE_COLORS: Record<string, string> = {
  entity: 'text-blue-400',
  alert: 'text-orange-400',
  case: 'text-green-400',
};

export function SearchScreen() {
  const navigate = useNavigate();
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);
  const [filter, setFilter] = useState<string>('');

  const handleSearch = async () => {
    if (!query.trim()) return;
    setLoading(true);
    setSearched(true);
    try {
      const data = await searchAll(query, filter || undefined);
      setResults(data.results);
      setTotal(data.total);
    } catch (err) {
      console.error('Search failed:', err);
      setResults([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleSearch();
  };

  return (
    <div className="mx-auto max-w-4xl p-6">
      <header className="mb-6">
        <h1 className="text-lg font-semibold">Search</h1>
        <p className="text-sm text-muted-foreground">
          Search across all cases for entities, alerts, and case records.
        </p>
      </header>

      {/* Search bar */}
      <div className="mb-4 flex gap-2">
        <div className="relative flex-1">
          <SearchIcon className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="Search by phone number, IMEI, account, name, case title..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            className="pl-9"
          />
        </div>
        <Button onClick={handleSearch} disabled={loading || !query.trim()}>
          {loading ? 'Searching...' : 'Search'}
        </Button>
      </div>

      {/* Filters */}
      <div className="mb-6 flex gap-2">
        {['', 'entity', 'alert', 'case'].map((type) => (
          <Button
            key={type}
            variant={filter === type ? 'default' : 'outline'}
            size="sm"
            onClick={() => setFilter(type)}
          >
            {type || 'All'}
          </Button>
        ))}
        {searched && (
          <span className="ml-auto text-sm text-muted-foreground">
            {total} result{total !== 1 ? 's' : ''}
          </span>
        )}
      </div>

      {/* Results */}
      {loading ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <Skeleton key={i} className="h-20" />
          ))}
        </div>
      ) : results.length > 0 ? (
        <div className="space-y-2">
          {results.map((result, i) => (
            <SearchResultCard key={`${result.case_id}-${i}`} result={result} navigate={navigate} />
          ))}
        </div>
      ) : searched ? (
        <div className="py-12 text-center text-muted-foreground">
          No results found for "{query}"
        </div>
      ) : (
        <div className="py-12 text-center text-muted-foreground">
          <SearchIcon className="mx-auto mb-3 h-8 w-8 opacity-50" />
          <p>Enter a search query to find entities, alerts, and cases.</p>
        </div>
      )}
    </div>
  );
}

function SearchResultCard({
  result,
  navigate,
}: {
  result: SearchResult;
  navigate: ReturnType<typeof useNavigate>;
}) {
  const Icon = TYPE_ICONS[result.result_type] || FileText;
  const color = TYPE_COLORS[result.result_type] || 'text-zinc-400';

  const handleClick = () => {
    if (result.result_type === 'case') {
      navigate(`/cases/${result.case_id}`);
    } else {
      navigate(`/cases/${result.case_id}`);
    }
  };

  return (
    <div
      className="flex items-center gap-4 rounded-lg border border-white/10 bg-zinc-900/50 p-4 hover:bg-zinc-800/50 cursor-pointer transition-colors"
      onClick={handleClick}
    >
      <Icon className={`${color} h-5 w-5 shrink-0`} />
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <Badge variant="outline" className="text-[10px] uppercase">
            {result.result_type}
          </Badge>
          <span className="text-sm font-medium truncate">{result.identifier}</span>
        </div>
        <p className="text-xs text-muted-foreground truncate">
          Case: {result.case_title}
        </p>
      </div>
      <ArrowRight className="h-4 w-4 text-muted-foreground shrink-0" />
    </div>
  );
}
