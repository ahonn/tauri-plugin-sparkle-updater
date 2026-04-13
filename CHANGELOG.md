# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.4](https://github.com/ahonn/tauri-plugin-sparkle-updater/compare/v0.2.3...v0.2.4) - 2026-04-13

### Fixed

- correct minimum macOS version requirement to 10.13+

## [0.2.3](https://github.com/ahonn/tauri-plugin-sparkle-updater/compare/v0.2.2...v0.2.3) - 2026-03-24

### Added

- add download-specific request headers API

### Fixed

- set DYLD_FRAMEWORK_PATH in pre-commit hook for Sparkle.framework
- avoid dispatch_sync deadlock when called from main thread
- *(ci)* set DYLD_FRAMEWORK_PATH for tests to find Sparkle.framework

### Other

- update devenv.lock
- *(deps-dev)* bump rollup from 4.53.4 to 4.54.0 in the npm-deps group
- add husky pre-commit hook for local checks
- add CI/CD workflows and dependabot configuration
