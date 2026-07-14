# ClaudeUsageWidget (WidgetKit extension)

Swift source for the Notification Center / desktop widget. It's a plain
`.systemSmall`/`.systemMedium` WidgetKit extension with no dependencies
beyond WidgetKit and SwiftUI.

## Project layout (generated via xcodegen)

`project.yml` defines two targets, built with `xcodegen generate` into
`ClaudeUsageMonitorWidget.xcodeproj` (gitignored — regenerate it, don't
commit it):

- **ClaudeUsageMonitorHost** — a minimal `LSUIElement` (no Dock icon) host
  app that exists only so macOS has something to attribute the widget to.
  The real menu bar UI is still the Tauri binary from `src-tauri/`.
- **ClaudeUsageWidgetExtension** — this widget, embedded in the host app's
  `Contents/PlugIns/` at build time.

Both targets declare the same App Group
(`group.dev.jasonchen.claude-usage-monitor`) in their `.entitlements` files.
The Tauri app mirrors the latest per-provider snapshot into that App
Group's shared container (see `src-tauri/src/snapshot.rs`); this widget's
`SharedSnapshotStore` only reads from it. Neither side needs IPC — the
filesystem is the interface, same as the collector-script-to-app
relationship.

## Building it (one-time setup + rebuilds)

```sh
brew install xcodegen   # once
cd widget
xcodegen generate
open ClaudeUsageMonitorWidget.xcodeproj
```

In Xcode, for **both** the `ClaudeUsageMonitorHost` and
`ClaudeUsageWidgetExtension` targets:

1. **Signing & Capabilities** → set your Team (a free personal Apple ID
   works for local testing; a paid Developer account is needed to
   distribute). Xcode will auto-manage provisioning.
2. Confirm the **App Groups** capability shows
   `group.dev.jasonchen.claude-usage-monitor` checked — Xcode adds this
   automatically from the `.entitlements` files above, but it needs a real
   Team ID to actually provision the group.

Then Product → Run (▸) on the `ClaudeUsageMonitorHost` scheme, or Product →
Build for a release build to embed in the final DMG.

### Why this can't be scripted further

This project was verified end-to-end via `xcodegen generate` +
`xcodebuild build`, which succeeds and produces
`ClaudeUsageMonitorHost.app` with `ClaudeUsageWidgetExtension.appex`
correctly embedded in `Contents/PlugIns/`. But an **ad-hoc signed** build
(no Team ID) does not get picked up by `pluginkit`/WidgetKit — App Group
entitlements only take effect with a real provisioning profile, which
requires being signed in to Xcode with an actual Apple ID. That one step —
picking your Team in Signing & Capabilities — has to happen in the Xcode
GUI; there's no CLI-only way to attach your personal signing identity.

Once signed with a real Team ID and run once, the widget appears in the
macOS widget gallery like any other.

## Data contract

`SharedSnapshot.swift` mirrors `docs/usage-snapshot-schema.md` exactly. If
that schema changes, update the `CodingKeys` here to match.
