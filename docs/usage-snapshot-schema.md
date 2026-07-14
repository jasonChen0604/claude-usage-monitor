# UsageSnapshot schema

Every provider writes one JSON file to the shared snapshot directory:

```
~/Library/Application Support/claude-usage-monitor/snapshots/<provider>.json
```

Format:

```json
{
  "provider": "claude",
  "updated_at": "2026-07-14T10:32:00Z",
  "windows": [
    { "label": "5h", "used_percentage": 42, "resets_at": "2026-07-14T13:00:00Z" },
    { "label": "7d", "used_percentage": 18, "resets_at": "2026-07-18T00:00:00Z" }
  ]
}
```

- `provider` — free-form string, used as the display group in the UI. Not a
  fixed enum — adding a new provider needs no app-core changes.
- `updated_at` — ISO 8601 UTC timestamp of when the collector last wrote this
  file.
- `windows` — list of rate-limit windows. `label` is free-form (`"5h"`,
  `"7d"`, or whatever a future provider uses); `used_percentage` is 0-100;
  `resets_at` is an ISO 8601 UTC timestamp.

## Adding a new provider

1. Write a collector script that produces a file matching this schema at
   `snapshots/<your-provider>.json`.
2. Wire it up however that provider exposes usage data (a CLI hook, a local
   log file, etc. — whatever is documented and ToS-safe for that provider).
3. Nothing in the Tauri app, tray, popover, or widget needs to change — they
   already read every `*.json` file in the snapshots directory.
