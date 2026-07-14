# Setting up the statusLine collector

The app needs Claude Code to be configured to run this project's collector
script as its `statusLine.command`. The app's onboarding flow does this for
you automatically on first launch, but you can also do it manually:

1. Open `~/.claude/settings.json` (create it if it doesn't exist).
2. Add or edit the `statusLine` field:

```json
{
  "statusLine": {
    "type": "command",
    "command": "node /Applications/Claude Usage Monitor.app/Contents/Resources/scripts/claude-statusline-collector.cjs"
  }
}
```

The collector's stdout becomes your entire statusline text (Claude Code has no
separate "keep your old output" field — whatever the configured command
prints to stdout *is* the statusline). By default the collector just prints
the current model name. If you already have a custom `statusLine.command` and
want to keep its look, you have two options:

- Edit `scripts/claude-statusline-collector.cjs` to build whatever statusline
  string you want at the bottom of the script (you have the full input JSON:
  `model`, `cwd`, `cost`, `context_window`, etc. — see
  [statusline docs](https://code.claude.com/docs/en/statusline)).
- Or run both scripts in your `statusLine.command` (e.g. a small shell
  wrapper that pipes stdin to both, and prints your existing script's output).
  The collector always writes the snapshot file as a side effect regardless
  of what it prints.

3. Start (or resume) a Claude Code session and send at least one message —
   the `rate_limits` field only appears after the first API response.
4. Open the Claude Usage Monitor app; the tray icon should update within one
   polling interval (default 1 minute, configurable in Settings).
