# Releasing this repository

## Package versions and tags

| Package | Version source | Git tag | GitHub release title | Latest |
| --- | --- | --- | --- | --- |
| `sparkle-updater` | `crates/sparkle-updater/Cargo.toml` | `sparkle-updater-vX.Y.Z` | `sparkle-updater vX.Y.Z` | No |
| `tauri-plugin-sparkle-updater` | Root `Cargo.toml` | `vX.Y.Z` | `tauri-plugin-sparkle-updater vX.Y.Z` | Yes |
| `tauri-plugin-sparkle-updater-api` | `package.json` | No separate Git tag | No separate GitHub release | npm manages its own dist-tag |

The three versions are independent. Changing core does not require copying its version into either adapter. Existing published tags identify immutable package sources.

## Prepare and review

1. Merge feature/fix PRs into `master`. Validation runs, then release-plz creates or updates a release PR. Feature PRs cannot publish packages, including a newly added crate.
2. Review the Rust versions, dependency updates, compatibility report, and changelogs in the release PR.
3. Review JS API changes since the npm release. When the JS API or its required plugin support changes, update `package.json` and `CHANGELOG.npm.md` in the release PR. Choose the npm version independently; do not increment it for core-only changes. The current pnpm lockfile does not store the root package's version, so a version-only bump needs no lockfile edit.
4. Run `pnpm install --frozen-lockfile`, `pnpm build`, and `npm pack --dry-run`. Inspect exports and TypeScript declarations. Mention the planned npm version in the PR description. Recheck these manual additions whenever release-plz refreshes the PR.
5. Review CI and merge the release PR. Keep its `release-plz-` branch prefix: it identifies an approved release change for the automation.

For an npm-only release, create a PR from a `release-plz-npm-...` branch with the npm version/changelog changes. It uses the same validation and publication gate. PRs created using `GITHUB_TOKEN` may not trigger pull-request CI; run the CI workflow manually on the release branch when needed. Validation also runs on the exact merged commit before any publication.

## Publish and retry

After merging a release PR, the Release workflow validates the commit, then confirms that it is the merge commit of a merged PR from this repository's `release-plz-` branch targeting `master`. The Rust job also enforces `release_always = false` in release-plz.

Release-plz publishes eligible Rust crates in dependency order and creates each crate's own tag/release. Only the Tauri plugin is marked GitHub Latest. The npm job waits for Rust publication to succeed, skips an npm version that already exists, and fails on registry or publication errors.

If npm publication fails after the Rust crates succeed, rerun the failed jobs or the original workflow run for the same commit. Eligibility is based on the merged release PR, not on whether Rust published something during that attempt. A manual dispatch on a later feature commit cannot publish the release retroactively.

Required secrets are `CARGO_REGISTRY_TOKEN` (including permission to publish new crates when needed) and `NPM_TOKEN`. Published versions are never overwritten; corrections require a new version prepared through another release PR.
