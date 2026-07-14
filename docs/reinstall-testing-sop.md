# Reinstall & Testing SOP

Run this after any code change to do a clean reinstall and verify the core
behaviors (tray, window open/close, custom display items, popover live
refresh).

## 1. Clear old state

```sh
# Remove the previously installed app
rm -rf "/Applications/Claude Usage Monitor.app"

# Kill any process still running
pkill -f "claude-usage-monitor" 2>/dev/null

# Clear settings/cache (only needed to test a "fresh user" scenario;
# skip this if you're just retesting a change)
rm -rf "$HOME/Library/Application Support/claude-usage-monitor"
rm -rf "$HOME/Library/Application Support/dev.jasonchen.claude-usage-monitor"
```

## 2. Build the installer

```sh
cd /Users/jason/project/claude-usage-monitor
source "$HOME/.cargo/env"
pnpm dist
```

`pnpm dist` runs `tauri build`, which produces both the `.app` and a `.dmg`:

```
src-tauri/target/release/bundle/dmg/Claude Usage Monitor_<version>_aarch64.dmg
src-tauri/target/release/bundle/macos/Claude Usage Monitor.app
```

## 3. Install

Either:

- **Via the DMG** (mirrors the real user install flow)
  ```sh
  open "src-tauri/target/release/bundle/dmg/Claude Usage Monitor_0.1.0_aarch64.dmg"
  ```
  Drag the app into `Applications`, then eject the mounted volume.

- **Fast iteration** (skip the DMG, copy the `.app` directly)
  ```sh
  cp -R "src-tauri/target/release/bundle/macos/Claude Usage Monitor.app" /Applications/
  ```

## 4. Launch and check onboarding

```sh
open "/Applications/Claude Usage Monitor.app"
```

- Click the tray icon — the popover should appear.
- If `~/.claude/settings.json` doesn't already point at this project's
  collector script, the popover should show an onboarding prompt at the
  top. Click "Configure statusLine" and confirm `statusLine.command` in
  `~/.claude/settings.json` was actually written.

## 5. Seed test usage data

No need to run a real Claude Code session — just pipe fake data straight
into the collector script:

```sh
echo '{"model":{"display_name":"Opus"},"rate_limits":{"five_hour":{"used_percentage":42,"resets_at":'$(($(date +%s)+3600))'},"seven_day":{"used_percentage":18,"resets_at":'$(($(date +%s)+345600))'}}}' \
  | node scripts/claude-statusline-collector.cjs

cat "$HOME/Library/Application Support/claude-usage-monitor/snapshots/claude.json"
```

## 6. Verification checklist

- [ ] **Tray updates live**: change `used_percentage`/`resets_at` above and
      re-run the seed command, then wait one poll interval (or shorten the
      poll interval in Settings to test faster) and confirm the tray text
      changes.
- [ ] **Click tray to open/close**: click once to open the popover, click
      again to close (hide) it — the app process should stay running
      (`ps aux`), not disappear from the Dock or process list.
- [ ] **Window close button doesn't quit the app**: open the popover, click
      the window's red close button, confirm the app is still running
      (`ps aux | grep claude-usage-monitor` still shows it), and clicking
      the tray icon again brings the window back.
- [ ] **Custom tray display items**: in the popover's settings section,
      toggle any combination of 5h usage %, 5h countdown, weekly usage %,
      weekly countdown — confirm the tray text updates immediately to match
      the selection and order.
- [ ] **Popover live refresh**: after the tray text updates, switch the
      popover to the background for a bit then reopen it (or wait 60s while
      it's open), and confirm the percentages/countdowns inside the panel
      also update — not stuck on the data from when it was first opened.
- [ ] **Quit menu actually quits**: use the tray menu's Quit item and
      confirm the process disappears from `ps aux`.

## 7. Clean up (if you don't want to keep the test install)

```sh
pkill -f "claude-usage-monitor"
rm -rf "/Applications/Claude Usage Monitor.app"
```

If you don't want to keep the statusLine integration, remember to manually
restore or remove the `statusLine` field in `~/.claude/settings.json`.
