# JavaScript API changelog

Versions in this file apply to `tauri-plugin-sparkle-updater-api`, independently of the Rust crates.

## 0.3.0

### Added

- `downloadRequestHeaders` and `setDownloadRequestHeaders` for update-download-specific HTTP headers. Use with `tauri-plugin-sparkle-updater` 0.3.0 or newer, which includes their command permissions.
- Structured no-update reasons, including OS and Apple Silicon hardware requirements.
- Optional failure reasons, recovery suggestions, and underlying errors on update errors.

### Changed

- `DidNotFindUpdatePayload` now describes `NoUpdateInfo` rather than an empty object. Code constructing this type must supply the reason, reason code, and whether the check was user initiated.

## 0.2.2

Previously published JavaScript API baseline.
