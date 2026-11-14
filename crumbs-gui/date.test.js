import { test } from 'node:test';
import assert from 'node:assert/strict';
import process from 'node:process';
import { localDateISO } from './web/date.js';

// Pin a zone west of UTC so a late-evening local time falls on the next UTC
// day — the case where toISOString() reported tomorrow's date (cr-lvq).
// Setting TZ after the import works because date.js creates no Date at load
// time; keep it that way or move this above a dynamic import.
process.env.TZ = 'America/Denver';

test('late evening reports the local date, not the UTC date', () => {
  const lateEvening = new Date(2026, 8, 14, 23, 30); // 2026-09-14 23:30 local
  assert.equal(localDateISO(lateEvening), '2026-09-14');
});

test('pads single-digit months and days', () => {
  assert.equal(localDateISO(new Date(2026, 0, 5, 12, 0)), '2026-01-05');
});
