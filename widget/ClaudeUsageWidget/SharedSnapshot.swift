import Foundation

/// Mirrors the UsageSnapshot JSON schema (docs/usage-snapshot-schema.md).
/// The main app writes the latest snapshot for each provider into the
/// App Group container; this widget only ever reads it.
struct UsageWindow: Codable {
    let label: String
    let usedPercentage: Double
    let resetsAt: Date

    enum CodingKeys: String, CodingKey {
        case label
        case usedPercentage = "used_percentage"
        case resetsAt = "resets_at"
    }
}

struct UsageSnapshot: Codable {
    let provider: String
    let updatedAt: Date
    let windows: [UsageWindow]

    enum CodingKeys: String, CodingKey {
        case provider
        case updatedAt = "updated_at"
        case windows
    }
}

enum SharedSnapshotStore {
    static let appGroupId = "group.dev.jasonchen.claude-usage-monitor"

    static func readAll() -> [UsageSnapshot] {
        guard let containerURL = FileManager.default
            .containerURL(forSecurityApplicationGroupIdentifier: appGroupId)
        else { return [] }

        let snapshotsDir = containerURL.appendingPathComponent("snapshots")
        guard let files = try? FileManager.default.contentsOfDirectory(
            at: snapshotsDir, includingPropertiesForKeys: nil
        ) else { return [] }

        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601

        return files
            .filter { $0.pathExtension == "json" }
            .compactMap { url in
                guard let data = try? Data(contentsOf: url) else { return nil }
                return try? decoder.decode(UsageSnapshot.self, from: data)
            }
    }
}
