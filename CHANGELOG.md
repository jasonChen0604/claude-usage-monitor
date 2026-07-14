# Changelog

All notable changes to this project are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com); versioning follows
[Semantic Versioning](https://semver.org) (see `CLAUDE.md`).

## [Unreleased]

## [0.2.0] - 2026-07-14

### Added
- Launch at login option, via a new checkbox in the settings panel. The app
  can now start automatically at login, minimized to the tray, backed by
  tauri-plugin-autostart.

## [0.1.1] - 2026-07-14

### Fixed
- Fixed a broken code signature that caused macOS to report the app as
  "damaged" and refuse to open it. Caused by Tauri signing the .app before
  this project's custom resource (the collector script) finished copying
  into place, invalidating the signature's resource seal. The DMG is now
  built by a dedicated script that re-signs the app after all resources are
  in place.

## [0.1.0] - 2026-07-14

### Added
- Initial project scaffold: Tauri menu bar app, statusLine collector script
  for Claude, WidgetKit widget, DMG packaging.
- Tray icon with configurable display items (5-hour/weekly usage percentage
  and reset countdown, independently toggleable), popover panel, and a
  manual "Refresh Now" action in both the popover and the tray's right-click
  menu.
- statusLine onboarding flow to configure `~/.claude/settings.json`.
- Documentation covering the statusLine data-source rationale, the
  UsageSnapshot schema for future provider extensibility, and a
  reinstall/testing SOP.
