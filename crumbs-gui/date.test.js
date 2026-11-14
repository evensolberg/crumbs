import { test } from 'node:test';
import assert from 'node:assert/strict';

// Pin a zone west of UTC so a late-evening local time falls on the next UTC
// day — the case where toISOString() reported tomorrow's date (cr-lvq).
process.env.TZ = 'America/Denver';

const { localDateISO } = await import('./date.js');

test('late evening reports the local date, not the UTC date', () => {
  const lateEvening = new Date(2026, 8, 14, 23, 30); // 2026-09-14 23:30 local
  assert.equal(localDateISO(lateEvening), '2026-09-14');
});

test('pads single-digit months and days', () => {
  assert.equal(localDateISO(new Date(2026, 0, 5, 12, 0)), '2026-01-05');
});

test('defaults to the current local date', () => {
  const now = new Date();
  assert.equal(localDateISO(), localDateISO(now));
});
