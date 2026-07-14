import WidgetKit
import SwiftUI

struct UsageEntry: TimelineEntry {
    let date: Date
    let snapshots: [UsageSnapshot]
}

struct Provider: TimelineProvider {
    func placeholder(in context: Context) -> UsageEntry {
        UsageEntry(date: .now, snapshots: [])
    }

    func getSnapshot(in context: Context, completion: @escaping (UsageEntry) -> Void) {
        completion(UsageEntry(date: .now, snapshots: SharedSnapshotStore.readAll()))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<UsageEntry>) -> Void) {
        let entry = UsageEntry(date: .now, snapshots: SharedSnapshotStore.readAll())
        // Widgets don't poll on their own schedule; ask iOS/macOS to refresh
        // again in 10 minutes, matching the app's default poll interval.
        let nextUpdate = Calendar.current.date(byAdding: .minute, value: 10, to: .now)!
        completion(Timeline(entries: [entry], policy: .after(nextUpdate)))
    }
}

struct WindowRow: View {
    let window: UsageWindow

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack {
                Text(window.label == "5h" ? "5-hour" : window.label == "7d" ? "Weekly" : window.label)
                    .font(.caption)
                Spacer()
                Text("\(Int(window.usedPercentage))%")
                    .font(.caption.bold())
            }
            ProgressView(value: window.usedPercentage, total: 100)
                .tint(window.usedPercentage > 80 ? .red : .accentColor)
        }
    }
}

struct ClaudeUsageWidgetView: View {
    var entry: Provider.Entry

    var body: some View {
        let claude = entry.snapshots.first { $0.provider == "claude" }
        VStack(alignment: .leading, spacing: 8) {
            Text("Claude Usage")
                .font(.headline)
            if let claude, !claude.windows.isEmpty {
                ForEach(claude.windows, id: \.label) { WindowRow(window: $0) }
            } else {
                Text("No data yet")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding()
    }
}

struct ClaudeUsageWidget: Widget {
    let kind: String = "ClaudeUsageWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: Provider()) { entry in
            ClaudeUsageWidgetView(entry: entry)
        }
        .configurationDisplayName("Claude Usage")
        .description("Shows your Claude 5-hour and weekly usage.")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

@main
struct ClaudeUsageWidgetBundle: WidgetBundle {
    var body: some Widget {
        ClaudeUsageWidget()
    }
}
