import { appendFileSync, readFileSync } from 'node:fs';

const { name, version } = JSON.parse(readFileSync('package.json', 'utf8'));
const url = `https://registry.npmjs.org/${encodeURIComponent(name)}/${encodeURIComponent(version)}`;
const response = await fetch(url, { signal: AbortSignal.timeout(30_000) });
let publish;
if (response.status === 404) {
  publish = true;
} else if (response.ok) {
  const published = await response.json();
  if (published.name !== name || published.version !== version) {
    throw new Error(`Unexpected npm registry metadata for ${name}@${version}`);
  }
  publish = false;
} else {
  throw new Error(`npm registry returned HTTP ${response.status} for ${name}@${version}`);
}
console.log(`${name}@${version}: ${publish ? 'ready to publish' : 'already published; skipping'}`);
if (process.env.GITHUB_OUTPUT) {
  appendFileSync(process.env.GITHUB_OUTPUT, `publish=${publish}\n`);
}
