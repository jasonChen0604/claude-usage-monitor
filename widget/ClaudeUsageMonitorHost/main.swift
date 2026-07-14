// This target exists only so the WidgetKit extension has a host app to be
// embedded in and share an App Group with. The real menu bar UI is the
// Tauri binary built by `npm run tauri build` (see src-tauri/) — this stub
// has LSUIElement=true (no Dock icon) and does nothing but sit idle so
// macOS has an app to attribute the widget to.
import AppKit

NSApplication.shared.run()
