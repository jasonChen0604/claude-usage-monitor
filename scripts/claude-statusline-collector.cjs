#!/usr/bin/env node
// Reads Claude Code's statusLine JSON from stdin, writes a UsageSnapshot for
// the "claude" provider, and forwards the normal statusline text unchanged.
// See docs/data-source.md and docs/usage-snapshot-schema.md.

const fs = require("fs");
const os = require("os");
const path = require("path");

const SNAPSHOT_DIR = path.join(
  os.homedir(),
  "Library",
  "Application Support",
  "claude-usage-monitor",
  "snapshots"
);

function toIso(unixSeconds) {
  return new Date(unixSeconds * 1000).toISOString();
}

function buildSnapshot(rateLimits) {
  const windows = [];
  if (rateLimits.five_hour) {
    windows.push({
      label: "5h",
      used_percentage: rateLimits.five_hour.used_percentage,
      resets_at: toIso(rateLimits.five_hour.resets_at),
    });
  }
  if (rateLimits.seven_day) {
    windows.push({
      label: "7d",
      used_percentage: rateLimits.seven_day.used_percentage,
      resets_at: toIso(rateLimits.seven_day.resets_at),
    });
  }
  return {
    provider: "claude",
    updated_at: new Date().toISOString(),
    windows,
  };
}

let input = "";
process.stdin.on("data", (chunk) => {
  input += chunk;
});

process.stdin.on("end", () => {
  let payload;
  try {
    payload = JSON.parse(input);
  } catch {
    // Not JSON we understand — forward input untouched and exit quietly.
    process.stdout.write(input);
    return;
  }

  if (payload.rate_limits) {
    try {
      fs.mkdirSync(SNAPSHOT_DIR, { recursive: true });
      fs.writeFileSync(
        path.join(SNAPSHOT_DIR, "claude.json"),
        JSON.stringify(buildSnapshot(payload.rate_limits), null, 2)
      );
    } catch (err) {
      process.stderr.write(`claude-usage-monitor: failed to write snapshot: ${err.message}\n`);
    }
  }

  // Claude Code has no separate "output" field — whatever this script prints
  // to stdout IS the statusline text. We print a minimal default so the
  // statusline isn't left blank; see docs/setup-statusline.md for how to
  // chain this with an existing custom statusline script instead.
  const model = payload.model && payload.model.display_name;
  process.stdout.write(model ? `${model}` : "");
});
