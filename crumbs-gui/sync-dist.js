// Copy the hand-written frontend files into dist/, which Tauri bundles
// (frontendDist). Run by tauri.conf.json's beforeBuildCommand and
// beforeDevCommand so every build path, including tauri-action in the release
// workflow, ships the current sources. Node rather than `cp` because release
// builds also run on Windows and Linux.
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import process from 'node:process';
import { fileURLToPath, pathToFileURL } from 'node:url';

// dist/codemirror.bundle.js (built by `npm run build-cm6`) and
// dist/marked.min.js are committed separately and are not listed here.
export const FRONTEND_FILES = ['index.html', 'main.js', 'date.js', 'style.css'];

export function syncDist(srcDir, distDir) {
  mkdirSync(distDir, { recursive: true });
  for (const name of FRONTEND_FILES) {
    copyFileSync(join(srcDir, name), join(distDir, name));
  }
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const here = dirname(fileURLToPath(import.meta.url));
  syncDist(here, join(here, 'dist'));
}
