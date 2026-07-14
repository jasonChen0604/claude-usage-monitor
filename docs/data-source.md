# Where the usage data comes from

Anthropic does not publish an API for Claude Pro/Max **subscription** usage
(the 5-hour and weekly rate-limit windows shown by `/usage` in Claude Code).
The Console/Platform API's usage and rate-limit endpoints only cover
pay-as-you-go API key billing, which is a different thing.

There is an undocumented endpoint
(`GET https://api.anthropic.com/api/oauth/usage`) that returns this data, but
Anthropic's [legal and compliance terms](https://code.claude.com/docs/en/legal-and-compliance)
explicitly restrict OAuth tokens to "ordinary use of Claude Code and other
native Anthropic applications" and prohibit third-party tools from routing
requests through a user's Free/Pro/Max/Team/Enterprise credentials. This has
already been enforced server-side against third-party tools. This project
does not use that endpoint and will not accept contributions that add it.

## The supported path: `statusLine`

Claude Code has an official [statusLine](https://code.claude.com/docs/en/statusline)
feature: whatever command you configure in `~/.claude/settings.json` under
`statusLine.command` gets a JSON payload piped to its stdin on every render,
including (for Pro/Max accounts, after at least one API response in the
session):

```json
{
  "rate_limits": {
    "five_hour": { "used_percentage": 42, "resets_at": 1742651200 },
    "seven_day": { "used_percentage": 18, "resets_at": 1743120000 }
  }
}
```

This project's collector script (`scripts/claude-statusline-collector.cjs`) is
installed as that command. It extracts `rate_limits`, writes it out as a
`UsageSnapshot` file, and prints a minimal statusline (the current model
name) to stdout — see `docs/setup-statusline.md` for how to customize or
chain it with an existing statusline script.

### Known limitations

This only updates while Claude Code is running and actively rendering a
status line. It's a snapshot of "usage as of the last time Claude Code drew
its status line," not a live poll — there's no way around this without
using the prohibited private endpoint. If you haven't used Claude Code
recently, the numbers you see may be stale until your next session.

If you run **multiple Claude Code sessions at once** (e.g. several terminal
tabs), each one independently renders its own statusline and each invocation
overwrites the same shared `claude.json` snapshot file. Whichever session's
statusline rendered most recently wins — so the displayed percentage can
appear to jump between different sessions' momentary values rather than
monotonically increasing. This isn't a bug in this app's formatting or
polling logic; it's inherent to using a per-render statusline callback as
the data source instead of a single source of truth.
