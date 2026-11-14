import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { FRONTEND_FILES, syncDist } from './sync-dist.js';

function makeFrontend() {
  const root = mkdtempSync(join(tmpdir(), 'crumbs-sync-'));
  for (const name of FRONTEND_FILES) writeFileSync(join(root, name), `source ${name}`);
  return root;
}

test('copies every frontend file into a missing dist/', () => {
  const root = makeFrontend();
  syncDist(root, join(root, 'dist'));
  for (const name of FRONTEND_FILES) {
    assert.equal(readFileSync(join(root, 'dist', name), 'utf8'), `source ${name}`);
  }
});

test('overwrites stale copies and leaves other dist/ files alone', () => {
  const root = makeFrontend();
  const dist = join(root, 'dist');
  mkdirSync(dist);
  writeFileSync(join(dist, 'main.js'), 'stale');
  writeFileSync(join(dist, 'codemirror.bundle.js'), 'bundle');
  syncDist(root, dist);
  assert.equal(readFileSync(join(dist, 'main.js'), 'utf8'), 'source main.js');
  assert.equal(readFileSync(join(dist, 'codemirror.bundle.js'), 'utf8'), 'bundle');
});

test('fails loudly when a frontend file is missing', () => {
  const root = mkdtempSync(join(tmpdir(), 'crumbs-sync-'));
  assert.throws(() => syncDist(root, join(root, 'dist')), /ENOENT/);
});
