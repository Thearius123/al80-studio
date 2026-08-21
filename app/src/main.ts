import { invoke } from "@tauri-apps/api/core";
import "./style.css";

type DeviceStatus = {
  connected: boolean;
  devnode: string | null;
  matrixScanHz: number | null;
  matrixScanIntervalUs: number | null;
  rgbCoreEnabled: boolean | null;
  overlayEnabled: boolean | null;
  overlayReportsRgbCore: boolean | null;
  error: string | null;
};

const app = document.querySelector<HTMLDivElement>("#app");

if (!app) {
  throw new Error("#app not found");
}

app.innerHTML = `
  <div class="shell">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark">A</div>
        <div>
          <div class="brand-name">AL80 Studio</div>
          <div class="brand-version">0.3.0-alpha</div>
        </div>
      </div>

      <nav class="nav">
        <button class="nav-item active">
          <span>⌂</span>
          Dashboard
        </button>

        <button class="nav-item" disabled>
          <span>⌨</span>
          Keyboard
        </button>

        <button id="nav-rgb" class="nav-item">
          <span>✦</span>
          RGB
        </button>

        <button class="nav-item" disabled>
          <span>▣</span>
          LCD
        </button>

        <button class="nav-item" disabled>
          <span>◉</span>
          Knob
        </button>

        <button class="nav-item" disabled>
          <span>◇</span>
          Profiles
        </button>

        <button class="nav-item" disabled>
          <span>◌</span>
          Diagnostics
        </button>
      </nav>

      <div class="sidebar-footer">
        <div class="safety-chip">
          Runtime-safe preview
        </div>
        <div class="footer-copy">
          No persistent writes in this screen
        </div>
      </div>
    </aside>

    <main class="content">
      <header class="topbar">
        <div>
          <div class="eyebrow">YUNZII AL80</div>
          <h1>Dashboard</h1>
          <p>
            Live keyboard status through the AL80 Studio Rust core.
          </p>
        </div>

        <div class="top-actions">
          <div
            id="connection-pill"
            class="connection-pill loading"
          >
            <span class="dot"></span>
            <span id="connection-text">
              Checking…
            </span>
          </div>

          <button
            id="refresh"
            class="refresh-button"
          >
            Refresh
          </button>
        </div>
      </header>

      <section class="hero">
        <div>
          <div class="hero-kicker">
            Real hardware telemetry
          </div>

          <div class="hero-title">
            Your AL80, visible.
          </div>

          <div class="hero-copy">
            The first AL80 Studio interface is now wired directly
            to the same Rust core validated against your keyboard.
          </div>
        </div>

        <div class="keyboard-visual">
          <div class="keyboard-glow"></div>
          <div class="keyboard-board">
            <div class="key-row">
              ${Array.from({ length: 14 }, () => `<span></span>`).join("")}
            </div>
            <div class="key-row">
              ${Array.from({ length: 14 }, () => `<span></span>`).join("")}
            </div>
            <div class="key-row">
              ${Array.from({ length: 13 }, () => `<span></span>`).join("")}
            </div>
            <div class="key-row">
              ${Array.from({ length: 12 }, () => `<span></span>`).join("")}
            </div>
            <div class="key-row bottom">
              ${Array.from({ length: 8 }, () => `<span></span>`).join("")}
              <span class="space"></span>
              ${Array.from({ length: 4 }, () => `<span></span>`).join("")}
            </div>
          </div>
        </div>
      </section>

      <section class="metrics">
        <article class="metric-card">
          <div class="metric-label">
            Matrix scan
          </div>
          <div class="metric-value">
            <span id="scan-hz">—</span>
            <small>Hz</small>
          </div>
          <div
            id="scan-detail"
            class="metric-detail"
          >
            Waiting for keyboard
          </div>
        </article>

        <article class="metric-card">
          <div class="metric-label">
            RGB engine
          </div>
          <div
            id="rgb-value"
            class="metric-status"
          >
            —
          </div>
          <div class="metric-detail">
            Raw HID 0x48 query
          </div>
        </article>

        <article class="metric-card">
          <div class="metric-label">
            Snake / overlay
          </div>
          <div
            id="overlay-value"
            class="metric-status"
          >
            —
          </div>
          <div class="metric-detail">
            Raw HID 0x49 query
          </div>
        </article>

        <article class="metric-card">
          <div class="metric-label">
            Raw HID
          </div>
          <div
            id="hid-value"
            class="metric-status small"
          >
            —
          </div>
          <div class="metric-detail">
            FF60:61
          </div>
        </article>
      </section>

      <section id="rgb-controls" class="control-panel">
        <div class="control-copy">
          <div class="section-eyebrow">
            RGB RUNTIME CONTROL
          </div>
          <h2>Lighting engine</h2>
          <p>
            Toggle the AL80 RGB core at runtime. This command is
            volatile and does not write RGB state to EEPROM.
          </p>
        </div>

        <div class="control-action">
          <div id="rgb-control-state" class="control-state">
            Waiting for keyboard
          </div>

          <button
            id="rgb-toggle"
            class="runtime-toggle"
            disabled
          >
            Loading…
          </button>

          <div class="volatile-note">
            VOLATILE · NO EEPROM WRITE
          </div>
        </div>
      </section>

      <section class="status-panel">
        <div class="panel-title-row">
          <div>
            <div class="section-eyebrow">
              DEVICE
            </div>
            <h2>Connection details</h2>
          </div>

          <div
            id="last-updated"
            class="last-updated"
          >
            Not queried yet
          </div>
        </div>

        <div class="detail-grid">
          <div class="detail">
            <span>Device</span>
            <strong>YUNZII AL80</strong>
          </div>

          <div class="detail">
            <span>VID : PID</span>
            <strong>28E9 : 30AF</strong>
          </div>

          <div class="detail">
            <span>Transport</span>
            <strong>Linux hidraw</strong>
          </div>

          <div class="detail">
            <span>Device node</span>
            <strong id="devnode">—</strong>
          </div>

          <div class="detail">
            <span>RGB core report</span>
            <strong id="rgb-report">—</strong>
          </div>

          <div class="detail">
            <span>Persistent write</span>
            <strong class="safe">Disabled</strong>
          </div>
        </div>

        <div
          id="error-box"
          class="error-box hidden"
        ></div>
      </section>
    </main>
  </div>
`;

const pill =
  document.querySelector<HTMLDivElement>("#connection-pill");
const connectionText =
  document.querySelector<HTMLSpanElement>("#connection-text");
const refresh =
  document.querySelector<HTMLButtonElement>("#refresh");

const scanHz =
  document.querySelector<HTMLSpanElement>("#scan-hz");
const scanDetail =
  document.querySelector<HTMLDivElement>("#scan-detail");

const rgbValue =
  document.querySelector<HTMLDivElement>("#rgb-value");
const overlayValue =
  document.querySelector<HTMLDivElement>("#overlay-value");
const hidValue =
  document.querySelector<HTMLDivElement>("#hid-value");

const devnode =
  document.querySelector<HTMLElement>("#devnode");
const rgbReport =
  document.querySelector<HTMLElement>("#rgb-report");
const lastUpdated =
  document.querySelector<HTMLDivElement>("#last-updated");
const errorBox =
  document.querySelector<HTMLDivElement>("#error-box");

const navRgb =
  document.querySelector<HTMLButtonElement>("#nav-rgb");
const rgbControls =
  document.querySelector<HTMLElement>("#rgb-controls");
const rgbToggle =
  document.querySelector<HTMLButtonElement>("#rgb-toggle");
const rgbControlState =
  document.querySelector<HTMLDivElement>("#rgb-control-state");

let currentRgbState: boolean | null = null;

function boolLabel(value: boolean | null): string {
  if (value === true) return "ON";
  if (value === false) return "OFF";
  return "—";
}

function resetLoading(): void {
  pill?.classList.remove("connected", "error");
  pill?.classList.add("loading");

  if (connectionText) {
    connectionText.textContent = "Checking…";
  }

  if (refresh) {
    refresh.disabled = true;
    refresh.textContent = "Refreshing…";
  }
}

function render(status: DeviceStatus): void {
  pill?.classList.remove("loading", "connected", "error");

  if (status.connected) {
    pill?.classList.add("connected");

    if (connectionText) {
      connectionText.textContent = "AL80 Connected";
    }
  } else {
    pill?.classList.add("error");

    if (connectionText) {
      connectionText.textContent = "AL80 Offline";
    }
  }

  if (scanHz) {
    scanHz.textContent =
      status.matrixScanHz?.toString() ?? "—";
  }

  if (scanDetail) {
    scanDetail.textContent =
      status.matrixScanIntervalUs !== null
        ? `${status.matrixScanIntervalUs.toFixed(1)} µs per scan`
        : "No scan telemetry";
  }

  if (rgbValue) {
    rgbValue.textContent =
      boolLabel(status.rgbCoreEnabled);

    rgbValue.dataset.state =
      status.rgbCoreEnabled === true
        ? "on"
        : status.rgbCoreEnabled === false
          ? "off"
          : "unknown";
  }

  currentRgbState = status.rgbCoreEnabled;

  if (rgbControlState) {
    rgbControlState.textContent =
      status.rgbCoreEnabled === true
        ? "RGB engine is ON"
        : status.rgbCoreEnabled === false
          ? "RGB engine is OFF"
          : "RGB state unavailable";
  }

  if (rgbToggle) {
    const usable =
      status.connected
      && status.rgbCoreEnabled !== null;

    rgbToggle.disabled = !usable;

    rgbToggle.textContent =
      status.rgbCoreEnabled === true
        ? "Turn RGB off"
        : status.rgbCoreEnabled === false
          ? "Turn RGB on"
          : "Unavailable";

    rgbToggle.dataset.state =
      status.rgbCoreEnabled === true
        ? "on"
        : status.rgbCoreEnabled === false
          ? "off"
          : "unknown";
  }

  if (overlayValue) {
    overlayValue.textContent =
      boolLabel(status.overlayEnabled);

    overlayValue.dataset.state =
      status.overlayEnabled === true
        ? "on"
        : status.overlayEnabled === false
          ? "off"
          : "unknown";
  }

  if (hidValue) {
    hidValue.textContent =
      status.connected
        ? "Connected"
        : "Offline";
  }

  if (devnode) {
    devnode.textContent =
      status.devnode ?? "—";
  }

  if (rgbReport) {
    rgbReport.textContent =
      boolLabel(status.overlayReportsRgbCore);
  }

  if (lastUpdated) {
    lastUpdated.textContent =
      `Updated ${new Date().toLocaleTimeString()}`;
  }

  if (errorBox) {
    if (status.error) {
      errorBox.textContent = status.error;
      errorBox.classList.remove("hidden");
    } else {
      errorBox.textContent = "";
      errorBox.classList.add("hidden");
    }
  }

  if (refresh) {
    refresh.disabled = false;
    refresh.textContent = "Refresh";
  }
}

async function loadStatus(): Promise<void> {
  resetLoading();

  try {
    const status =
      await invoke<DeviceStatus>("get_device_status");

    render(status);
  } catch (error) {
    render({
      connected: false,
      devnode: null,
      matrixScanHz: null,
      matrixScanIntervalUs: null,
      rgbCoreEnabled: null,
      overlayEnabled: null,
      overlayReportsRgbCore: null,
      error: String(error),
    });
  }
}

refresh?.addEventListener(
  "click",
  () => {
    void loadStatus();
  },
);

navRgb?.addEventListener(
  "click",
  () => {
    rgbControls?.scrollIntoView({
      behavior: "smooth",
      block: "center",
    });
  },
);

rgbToggle?.addEventListener(
  "click",
  async () => {
    if (currentRgbState === null) {
      return;
    }

    const wanted = !currentRgbState;

    rgbToggle.disabled = true;
    rgbToggle.textContent =
      wanted
        ? "Turning on…"
        : "Turning off…";

    try {
      const actual =
        await invoke<boolean>(
          "set_rgb_core_runtime",
          {
            enabled: wanted,
          },
        );

      currentRgbState = actual;

      await loadStatus();
    } catch (error) {
      if (errorBox) {
        errorBox.textContent =
          `RGB runtime command failed: ${String(error)}`;
        errorBox.classList.remove("hidden");
      }

      await loadStatus();
    }
  },
);

void loadStatus();
