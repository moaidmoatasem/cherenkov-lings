import React, { useState } from 'react';

const FALLBACK_CATALOG: Record<string, string[]> = {
  p: ['Python', 'PHP', 'Perl', 'PostgreSQL', 'PowerShell'],
  pl: ['Playwright', 'Platform', 'Plugin', 'Playlist'],
  pla: ['Playwright', 'Playwright TypeScript', 'Playwright Python', 'Playground'],
  play: ['Playwright', 'Playwright TypeScript', 'Playwright Python', 'Playground', 'Playbook'],
  playwright: ['Playwright', 'Playwright TypeScript', 'Playwright Python', 'Playwright Java', 'Playwright C#'],
};

export const SearchPage: React.FC = () => {
  const [inputValue, setInputValue] = useState<string>('');
  const [activeQuery, setActiveQuery] = useState<string>('');
  const [results, setResults] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [requestLog, setRequestLog] = useState<Array<{ id: number; query: string; durationMs: number; status: string }>>([]);

  const executeSearch = async (query: string) => {
    const trimmed = query.trim();
    if (!trimmed) {
      setActiveQuery('');
      setResults([]);
      return;
    }

    setIsLoading(true);
    const start = performance.now();
    const reqId = Date.now() + Math.floor(Math.random() * 1000);

    try {
      const res = await fetch(`http://localhost:8081/search?q=${encodeURIComponent(trimmed)}`);
      if (res.ok) {
        const data = await res.json();
        const duration = Math.round(performance.now() - start);

        // Intentionally flawed handler: No sequence token or abort controller.
        // Clobbers activeQuery and results with whichever response arrives latest.
        setActiveQuery(data.query);
        setResults(data.results || []);

        setRequestLog((prev) => [
          { id: reqId, query: data.query, durationMs: duration, status: 'Resolved' },
          ...prev.slice(0, 7),
        ]);
      }
    } catch {
      // Offline fallback simulating backend out-of-order latency
      const duration = trimmed.length <= 2 ? 800 : 50;
      setTimeout(() => {
        const queryLower = trimmed.toLowerCase();
        const matched = FALLBACK_CATALOG[queryLower] || [
          'Playwright',
          'Playwright TypeScript',
          'Playwright Python',
        ].filter((item) => item.toLowerCase().includes(queryLower));

        setActiveQuery(trimmed);
        setResults(matched);
        setRequestLog((prev) => [
          { id: reqId, query: trimmed, durationMs: duration, status: 'Resolved (Simulated)' },
          ...prev.slice(0, 7),
        ]);
      }, duration);
    } finally {
      setIsLoading(false);
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setInputValue(value);

    if (!value.trim()) {
      setActiveQuery('');
      setResults([]);
      return;
    }

    // When typing or filling, trigger out-of-order race condition by firing both prefix and full query
    if (value.length > 2) {
      // Fire delayed prefix request ('p') which takes 500ms
      executeSearch(value.slice(0, 1));
    }

    // Fire actual target query (which takes 50ms)
    executeSearch(value);
  };

  return (
    <div className="page-container" data-testid="search-page">
      <div className="page-header">
        <span className="badge danger">Concurrency Hazard: Out-of-Order Response Race</span>
        <h1>Autocomplete Search</h1>
        <p className="page-description">
          Demonstrates race conditions in debounced asynchronous search inputs. Responses for short queries
          take longer than longer queries, causing stale responses to clobber newer results if not synchronized.
        </p>
      </div>

      <div className="grid-2-col">
        <div className="card search-card">
          <div className="search-box-wrapper">
            <label htmlFor="search-box" className="search-label">
              Search QA & SDET Courses:
            </label>
            <div className="search-input-container">
              <span className="search-icon">🔍</span>
              <input
                id="search-box"
                data-testid="search-input"
                type="text"
                aria-label="Search QA & SDET topics"
                value={inputValue}
                onChange={handleInputChange}
                placeholder="Type 'playwright'..."
                className="search-input"
                autoComplete="off"
              />
              {isLoading && <span className="spinner-sm"></span>}
            </div>
          </div>

          <div className="query-status-bar">
            <span className="status-label">Active Query Tag:</span>
            <span id="active-query" data-testid="active-query" className="query-tag">
              {activeQuery || 'none'}
            </span>
          </div>

          <div className="results-container">
            <div className="results-header">
              <span className="results-title">Search Results ({results.length})</span>
            </div>

            <ul id="search-results" data-testid="search-results" className="results-list">
              {results.length > 0 ? (
                results.map((result, idx) => (
                  <li key={`${result}-${idx}`} data-testid="result-item" className="result-item">
                    <span className="result-icon">📘</span>
                    <span className="result-text">{result}</span>
                  </li>
                ))
              ) : (
                <li className="no-results" data-testid="no-results">
                  {inputValue ? 'No matching topics found' : 'Start typing to see suggestions...'}
                </li>
              )}
            </ul>
          </div>
        </div>

        <div className="card diagnostic-card">
          <h2 className="card-title">Network Concurrency Inspector</h2>
          <div className="state-indicators">
            <div className="indicator-row">
              <span>Input Field Value:</span>
              <code>"{inputValue}"</code>
            </div>
            <div className="indicator-row">
              <span>Rendered Active Query:</span>
              <code>"{activeQuery}"</code>
            </div>
            <div className="indicator-row">
              <span>Results Rendered:</span>
              <code>{results.length} items</code>
            </div>
          </div>

          <div className="request-log-box">
            <h4>Recent Network Responses (Arrival Order)</h4>
            {requestLog.length === 0 ? (
              <p className="log-empty">No queries executed yet.</p>
            ) : (
              <ul className="log-list">
                {requestLog.map((log) => (
                  <li key={log.id} className="log-item">
                    <span className="log-query">q="{log.query}"</span>
                    <span className="log-duration">{log.durationMs}ms</span>
                    <span className="log-status">{log.status}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="code-hint">
            <h4>Race Condition Solution</h4>
            <p><strong>Anti-pattern:</strong> <code>await page.waitForTimeout(100);</code> (Races with delayed prefix response arriving at 500ms).</p>
            <p><strong>Fix:</strong> <code>await expect(page.getByTestId('active-query')).toHaveText('playwright');</code></p>
          </div>
        </div>
      </div>
    </div>
  );
};
