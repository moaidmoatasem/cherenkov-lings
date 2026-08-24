const API_BASE: string = import.meta.env.VITE_API_BASE ?? 'http://localhost:8081';

export function apiUrl(path: string): string {
  return `${API_BASE}${path}`;
}
