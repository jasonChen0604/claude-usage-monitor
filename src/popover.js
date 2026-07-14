import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

const WINDOW_TITLES = { "5h": "5-hour usage", "7d": "Weekly usage" };

function formatCountdown(resetsAt) {
  const remainingMs = new Date(resetsAt).getTime() - Date.now();
  if (remainingMs <= 0) return "resets now";
  const totalMinutes = Math.floor(remainingMs / 60000);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  return hours > 0 ? `resets in ${hours}h${minutes}m` : `resets in ${minutes}m`;
}

function renderWindow(win) {
  const el = document.createElement("div");
  el.className = "window";
  el.innerHTML = `
    <div class="window-label">
      <span>${WINDOW_TITLES[win.label] ?? win.label}</span>
      <span>${Math.round(win.used_percentage)}%</span>
    </div>
    <div class="bar"><div class="bar-fill" style="width:${win.used_percentage}%"></div></div>
    <div class="reset">${formatCountdown(win.resets_at)}</div>
  `;
  return el;
}

function renderProvider(snapshot, container) {
  const heading = document.createElement("h3");
  heading.textContent = snapshot.provider;
  container.appendChild(heading);
  for (const win of snapshot.windows) {
    container.appendChild(renderWindow(win));
  }
}

function renderRefreshButton(container) {
  const button = document.createElement("button");
  button.className = "refresh-now";
  button.textContent = "Refresh now";
  button.addEventListener("click", async () => {
    button.disabled = true;
    button.textContent = "Refreshing…";
    await invoke("refresh_now");
    await render();
  });
  container.appendChild(button);
}

const TRAY_ITEM_LABELS = {
  five_hour_percentage: "5-hour usage %",
  five_hour_countdown: "5-hour reset countdown",
  weekly_percentage: "Weekly usage %",
  weekly_countdown: "Weekly reset countdown",
};

async function renderSettings(container) {
  const settings = await invoke("get_settings");
  const enabled = new Set(settings.trayItems);

  const wrap = document.createElement("div");
  wrap.className = "settings";

  const checkboxes = Object.entries(TRAY_ITEM_LABELS)
    .map(
      ([value, label]) => `
      <label>
        <input type="checkbox" class="tray-item" value="${value}" ${enabled.has(value) ? "checked" : ""} />
        ${label}
      </label>`
    )
    .join("");

  wrap.innerHTML = `
    <label>Poll interval (minutes)
      <input type="number" min="1" id="poll-interval" value="${settings.pollIntervalMinutes}" style="width:50px" />
    </label>
    <p class="settings-heading">Status bar shows</p>
    ${checkboxes}
    <label>
      <input type="checkbox" id="launch-at-login" ${settings.launchAtLogin ? "checked" : ""} />
      Launch at login (minimized)
    </label>
  `;
  container.appendChild(wrap);

  const save = () => {
    const trayItems = [...wrap.querySelectorAll(".tray-item:checked")].map((el) => el.value);
    invoke("update_settings", {
      settings: {
        pollIntervalMinutes: Number(wrap.querySelector("#poll-interval").value) || 10,
        trayItems,
        launchAtLogin: wrap.querySelector("#launch-at-login").checked,
      },
    });
  };

  wrap.querySelectorAll("input").forEach((el) => el.addEventListener("change", save));
}

async function renderOnboarding(container) {
  const state = await invoke("check_statusline");
  if (state.configured) return;

  const box = document.createElement("div");
  box.className = "onboarding";
  const warning = state.existing_command
    ? `<p>Claude Code's statusLine is currently set to:<br><code>${state.existing_command}</code></p>
       <p>Continuing will replace it. See docs/setup-statusline.md to combine it with your existing script instead.</p>`
    : `<p>Claude Code needs its statusLine pointed at this app's collector script to report usage.</p>`;
  box.innerHTML = `
    ${warning}
    <button id="install-statusline">Configure statusLine</button>
  `;
  container.appendChild(box);

  box.querySelector("#install-statusline").addEventListener("click", async () => {
    await invoke("install_statusline");
    render();
  });
}

async function render() {
  const app = document.getElementById("app");
  app.innerHTML = "";

  await renderOnboarding(app);

  const snapshots = await invoke("get_snapshots");
  if (snapshots.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty";
    empty.textContent = "No usage data yet. Use Claude Code once to populate this.";
    app.appendChild(empty);
  } else {
    for (const snapshot of snapshots) {
      renderProvider(snapshot, app);
    }
  }

  renderRefreshButton(app);
  await renderSettings(app);
}

render();

// The window is hidden/shown, not reloaded, when toggled from the tray
// (see main.rs's CloseRequested handler), so a one-time render on page load
// would otherwise go stale. Re-render whenever the window regains focus,
// and periodically while it's left open so the countdown text stays fresh.
const currentWindow = getCurrentWindow();
currentWindow.listen("tauri://focus", () => render());
setInterval(() => {
  const editingSettings = document.activeElement?.closest(".settings");
  if (document.visibilityState === "visible" && !editingSettings) render();
}, 60_000);

// Fires after any refresh_tray call in Rust — the poll loop, the tray
// menu's "Refresh Now" item, or this panel's own button — so the popover
// stays in sync no matter which one triggered it.
listen("usage-refreshed", () => render());
