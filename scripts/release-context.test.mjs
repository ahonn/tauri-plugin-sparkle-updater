import assert from 'node:assert/strict';
import test from 'node:test';
import { isReleaseMerge, parsePullPages } from './release-context.mjs';

const context = {
  repository: 'ahonn/tauri-plugin-sparkle-updater',
  branch: 'master',
  sha: 'a'.repeat(40),
};

function releasePull() {
  return {
    merged_at: '2026-09-19T07:06:47Z',
    merge_commit_sha: context.sha,
    base: { ref: 'master', repo: { full_name: context.repository } },
    head: { ref: 'release-plz-2026-09-19T07-06-46Z', repo: { full_name: context.repository } },
  };
}

test('recognizes a merged release PR, including retries of the same commit', () => {
  const pulls = [releasePull()];
  assert.equal(isReleaseMerge(pulls, context), true);
  assert.equal(isReleaseMerge(pulls, context), true);
});

test('does not publish a commit without an associated release PR', () => {
  assert.equal(isReleaseMerge([], context), false);
  assert.equal(isReleaseMerge([{}], context), false);
});

for (const [name, change] of [
  ['an unmerged PR', (pull) => { pull.merged_at = null; }],
  ['a different merge commit', (pull) => { pull.merge_commit_sha = 'b'.repeat(40); }],
  ['a different base branch', (pull) => { pull.base.ref = 'develop'; }],
  ['a different base repository', (pull) => { pull.base.repo.full_name = 'someone/other'; }],
  ['a fork', (pull) => { pull.head.repo.full_name = 'someone/tauri-plugin-sparkle-updater'; }],
  ['a deleted source repository', (pull) => { pull.head.repo = null; }],
  ['a feature PR', (pull) => { pull.head.ref = 'feat/release-policy'; }],
  ['a similar but invalid branch prefix', (pull) => { pull.head.ref = 'release-plz'; }],
]) {
  test(`rejects ${name}`, () => {
    const pull = releasePull();
    change(pull);
    assert.equal(isReleaseMerge([pull], context), false);
  });
}

test('finds a release PR across paginated results', () => {
  const feature = releasePull();
  feature.head.ref = 'feat/example';
  const pulls = parsePullPages(JSON.stringify([[feature], [releasePull()]]));
  assert.equal(pulls.length, 2);
  assert.equal(isReleaseMerge(pulls, context), true);
});

test('accepts empty paginated results', () => {
  assert.deepEqual(parsePullPages('[[]]'), []);
});

test('rejects malformed API responses instead of allowing publication', () => {
  for (const output of ['not json', '{}', '[{}]', '[[null]]', '[[[]]]', '[["unexpected"]]']) {
    assert.throws(() => parsePullPages(output));
  }
  assert.throws(() => isReleaseMerge({}, context));
});
