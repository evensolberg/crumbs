// Local-time YYYY-MM-DD. toISOString() is UTC, which is already tomorrow in
// the evening west of Greenwich; the CLI uses chrono::Local to match.
export function localDateISO(date = new Date()) {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}
