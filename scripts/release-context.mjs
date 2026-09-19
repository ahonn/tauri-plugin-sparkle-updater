import { execFileSync } from 'node:child_process';
import { appendFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function parsePullPages(output) {
  const pages = JSON.parse(output);
  if (!Array.isArray(pages) || !pages.every(Array.isArray)) {
    throw new Error('Expected paginated arrays of pull requests from GitHub');
  }
  const pulls = pages.flat();
  if (!pulls.every((pull) => pull !== null && typeof pull === 'object' && !Array.isArray(pull))) {
    throw new Error('Expected pull request objects from GitHub');
  }
  return pulls;
}

export function isReleaseMerge(pulls, { repository, branch, sha }) {
  if (!Array.isArray(pulls)) {
    throw new Error('Expected an array of pull requests');
  }
  return pulls.some((pull) => (
    typeof pull?.merged_at === 'string'
    && pull.merged_at.length > 0
    && pull.merge_commit_sha === sha
    && pull.base?.ref === branch
    && pull.base?.repo?.full_name === repository
    && pull.head?.repo?.full_name === repository
    && typeof pull.head?.ref === 'string'
    && pull.head.ref.startsWith('release-plz-')
  ));
}

function main() {
  const repository = process.env.GITHUB_REPOSITORY;
  const branch = process.env.GITHUB_REF_NAME;
  const sha = process.env.GITHUB_SHA;
  if (!repository || !/^[\w.-]+\/[\w.-]+$/.test(repository) || !branch || !/^[a-f\d]{40}$/i.test(sha ?? '')) {
    throw new Error('GITHUB_REPOSITORY, GITHUB_REF_NAME, and GITHUB_SHA must identify the workflow commit');
  }
  const output = execFileSync('gh', [
    'api', `repos/${repository}/commits/${sha}/pulls`, '--paginate', '--slurp',
  ], { encoding: 'utf8', timeout: 60_000, maxBuffer: 5 * 1024 * 1024 });
  const release = isReleaseMerge(parsePullPages(output), { repository, branch, sha });
  console.log(`${sha}: ${release ? 'merged release PR; publication allowed' : 'not a merged release PR; skipping publication'}`);
  if (process.env.GITHUB_OUTPUT) {
    appendFileSync(process.env.GITHUB_OUTPUT, `release=${release}\n`);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
