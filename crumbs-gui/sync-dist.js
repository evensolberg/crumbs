// Copy the hand-written frontend files into dist/, which Tauri bundles
// (frontendDist). Run by tauri.conf.json's beforeBuildCommand and
// beforeDevCommand so every build path, including tauri-action in the release
// workflow, ships the current sources. Node rather than `cp` because release
// builds also run on Windows and Linux.
import { copyFileSync, mkdirSync, realpathSync } from 'node:fs';
import { dirname, join } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

// Every file index.html loads or main.js imports must be listed here, or it
// will be missing from builds. dist/codemirror.bundle.js (built by
// `npm run build-cm6`) and dist/marked.min.js are committed separately.
export const FRONTEND_FILES = ['index.html', 'main.js', 'date.js', 'style.css'];

export function syncDist(srcDir, distDir) {
  mkdirSync(distDir, { recursive: true });
  for (const name of FRONTEND_FILES) {
    copyFileSync(join(srcDir, name), join(distDir, name));
  }
}

// Compare real paths so running through a symlinked path still syncs.
const here = fileURLToPath(import.meta.url);
if (process.argv[1] && realpathSync(process.argv[1]) === here) {
  const dir = dirname(here);
  syncDist(dir, join(dir, 'dist'));
  console.log(`sync-dist: copied ${FRONTEND_FILES.length} files into dist/`);
}
