/**
 * Format a date as `YYYY-MM-DD` in the user's local time zone.
 *
 * `Date#toISOString()` reports UTC, so in the evening west of Greenwich it
 * returns tomorrow's date. The CLI uses `chrono::Local`; this keeps the GUI
 * consistent with it.
 *
 * @param {Date} [date] - defaults to now
 * @returns {string}
 */
export function localDateISO(date = new Date()) {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, '0');
  const d = String(date.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}
