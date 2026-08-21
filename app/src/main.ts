import { invoke } from "@tauri-apps/api/core";
import "./style.css";

type View =
  | "dashboard"
  | "effects"
  | "rgb"
  | "lcd"
  | "profiles"
  | "diagnostics";

interface DeviceStatus {
  connected: boolean;
  devnode: string | null;
  matrixScanHz: number | null;
  matrixScanIntervalUs: number | null;
  rgbCoreEnabled: boolean | null;
  overlayEnabled: boolean | null;
  overlayReportsRgbCore: boolean | null;
  error: string | null;
}

interface Capabilities {
  api: number;
  daemonVersion: string;
  firmwareMode: string;
  matrixScan: boolean;
  rgbRuntime: boolean;
  overlay: boolean;
  lcdOsd: boolean;
  audioWatch: boolean;
  profiles: boolean;
  extensionManifest: string;
  persistentWrite: boolean;
  eepromWrite: boolean;
  qmkFlash: boolean;
  scanHz: number | null;
  rgbState: boolean | null;
  overlayState: boolean | null;
  overlayRgbState: boolean | null;
  error: string | null;
}

function requireAppRoot(): HTMLDivElement {
  const element = document.querySelector<HTMLDivElement>("#app");

  if (element === null) {
    throw new Error("#app element was not found");
  }

  return element;
}

const app = requireAppRoot();

let view: View = "dashboard";
let status: DeviceStatus | null = null;
let capabilities: Capabilities | null = null;
let busy = false;
let notice = "";

function esc(value: unknown): string {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function yesNo(value: boolean | null | undefined): string {
  if (value === true) return "Yes";
  if (value === false) return "No";
  return "—";
}

function onOff(value: boolean | null | undefined): string {
  if (value === true) return "ON";
  if (value === false) return "OFF";
  return "—";
}

function navButton(id: View, label: string): string {
  return `
    <button
      class="nav-item ${view === id ? "active" : ""}"
      data-view="${id}"
      type="button"
    >
      ${label}
    </button>
  `;
}

function metric(
  label: string,
  value: string,
  detail = "",
): string {
  return `
    <article class="metric-card">
      <div class="metric-label">${esc(label)}</div>
      <div class="metric-value">${esc(value)}</div>
      ${detail ? `<div class="metric-detail">${esc(detail)}</div>` : ""}
    </article>
  `;
}

function badge(
  label: string,
  tone: "good" | "neutral" | "warn" = "neutral",
): string {
  return `<span class="badge ${tone}">${esc(label)}</span>`;
}

function renderDashboard(): string {
  const scanHz = status?.matrixScanHz
    ? `${status.matrixScanHz} Hz`
    : "—";

  const interval = status?.matrixScanIntervalUs
    ? `${status.matrixScanIntervalUs.toFixed(1)} µs`
    : "—";

  return `
    <section class="page">
      <div class="page-heading">
        <div>
          <p class="eyebrow">Hardware overview</p>
          <h1>Dashboard</h1>
          <p>
            Live state comes from al80d, the single Raw HID owner.
          </p>
        </div>
        <button class="secondary-btn" id="refresh" type="button">
          Refresh
        </button>
      </div>

      <div class="metric-grid">
        ${metric("Connection", status?.connected ? "Connected" : "Offline",
          status?.devnode ?? "No device")}
        ${metric("Matrix Scan", scanHz, interval)}
        ${metric("RGB Engine", onOff(status?.rgbCoreEnabled),
          "Runtime / volatile")}
        ${metric("Snake / Overlay", onOff(status?.overlayEnabled),
          capabilities?.firmwareMode ?? "Unknown firmware")}
      </div>

      <article class="panel">
        <div class="panel-title-row">
          <div>
            <p class="eyebrow">Architecture</p>
            <h2>Single-owner runtime</h2>
          </div>
          ${badge("al80d", "good")}
        </div>

        <div class="architecture-flow">
          <span>YUNZII AL80</span>
          <span>→</span>
          <span>al80d</span>
          <span>→</span>
          <span>AL80 Studio</span>
        </div>

        <p class="muted">
          Volume OSD, LCD, RGB, Snake and the GUI share one serialized
          hardware owner instead of competing for hidraw replies.
        </p>
      </article>
    </section>
  `;
}

function renderEffects(): string {
  const supported =
    capabilities?.firmwareMode === "EXTENDED" &&
    capabilities?.overlay === true;

  const active = status?.overlayEnabled === true;

  return `
    <section class="page">
      <div class="page-heading">
        <div>
          <p class="eyebrow">Customization</p>
          <h1>Effects</h1>
          <p>
            Effects are enabled only when the connected firmware reports the
            capabilities they require.
          </p>
        </div>
        ${badge(
          capabilities?.extensionManifest === "V1"
            ? "Extension Manifest V1"
            : "Manifest unavailable",
          capabilities?.extensionManifest === "V1" ? "good" : "warn",
        )}
      </div>

      <div class="effect-grid">
        <article class="effect-card ${active ? "effect-active" : ""}">
          <div class="effect-preview snake-preview" aria-hidden="true">
            <div class="snake-dot s1"></div>
            <div class="snake-dot s2"></div>
            <div class="snake-dot s3"></div>
            <div class="snake-dot s4"></div>
            <div class="snake-dot s5"></div>
          </div>

          <div class="effect-body">
            <div class="panel-title-row">
              <div>
                <p class="eyebrow">Firmware effect</p>
                <h2>Snake</h2>
              </div>
              ${badge(
                supported ? "Compatible" : "Unavailable",
                supported ? "good" : "warn",
              )}
            </div>

            <p>
              The first AL80 Studio extended-firmware effect discovered and
              controlled through the open capability layer.
            </p>

            <div class="chip-row">
              ${badge("Extended firmware")}
              ${badge("RGB runtime")}
              ${badge("Overlay")}
              ${badge("No EEPROM", "good")}
              ${badge("No flash", "good")}
            </div>

            <div class="effect-actions">
              <div>
                <span class="state-label">Current state</span>
                <strong>${active ? "Enabled" : "Disabled"}</strong>
              </div>

              <button
                id="snake-toggle"
                class="primary-btn"
                type="button"
                ${!supported || busy ? "disabled" : ""}
              >
                ${active ? "Disable Snake" : "Enable Snake"}
              </button>
            </div>
          </div>
        </article>

        <article class="effect-card placeholder-card">
          <div class="placeholder-plus">+</div>
          <div>
            <p class="eyebrow">Community extensions</p>
            <h2>More effects</h2>
            <p>
              This slot will be populated from extension manifests as the
              developer SDK grows.
            </p>
          </div>
        </article>
      </div>
    </section>
  `;
}

function renderRgb(): string {
  const supported = capabilities?.rgbRuntime === true;
  const enabled = status?.rgbCoreEnabled === true;

  return `
    <section class="page">
      <div class="page-heading">
        <div>
          <p class="eyebrow">Lighting</p>
          <h1>RGB</h1>
          <p>
            Runtime controls are volatile and routed through al80d.
          </p>
        </div>
        ${badge(supported ? "Supported" : "Unavailable",
          supported ? "good" : "warn")}
      </div>

      <article class="panel control-panel">
        <div>
          <p class="eyebrow">RGB core</p>
          <h2>${enabled ? "Lighting enabled" : "Lighting disabled"}</h2>
          <p class="muted">
            Turning the RGB core off also stops visible overlay output until
            RGB is enabled again.
          </p>
        </div>

        <button
          id="rgb-toggle"
          class="primary-btn"
          type="button"
          ${!supported || busy ? "disabled" : ""}
        >
          ${enabled ? "Turn RGB off" : "Turn RGB on"}
        </button>
      </article>

      <article class="panel">
        <h2>Coming next</h2>
        <div class="feature-list">
          <span>Color</span>
          <span>Brightness</span>
          <span>Speed</span>
          <span>Built-in effects</span>
          <span>Custom effect parameters</span>
        </div>
      </article>
    </section>
  `;
}

function renderLcd(): string {
  const supported = capabilities?.lcdOsd === true;

  return `
    <section class="page">
      <div class="page-heading">
        <div>
          <p class="eyebrow">Display</p>
          <h1>LCD</h1>
          <p>
            Safe volatile preview controls for the protocol already validated
            on the physical keyboard.
          </p>
        </div>
        ${badge(supported ? "OSD supported" : "Unavailable",
          supported ? "good" : "warn")}
      </div>

      <article class="panel">
        <div class="panel-title-row">
          <div>
            <p class="eyebrow">Volume OSD preview</p>
            <h2>Test the LCD</h2>
          </div>
          ${badge("Volatile only", "good")}
        </div>

        <p class="muted">
          These buttons change only the keyboard display. They do not change
          Fedora's actual volume.
        </p>

        <div class="button-row">
          ${[25, 50, 75, 100]
            .map(
              (n) => `
                <button
                  class="secondary-btn lcd-volume"
                  data-percent="${n}"
                  type="button"
                  ${!supported || busy ? "disabled" : ""}
                >
                  ${n}%
                </button>
              `,
            )
            .join("")}

          <button
            id="lcd-mute"
            class="secondary-btn"
            type="button"
            ${!supported || busy ? "disabled" : ""}
          >
            MUTE
          </button>

          <button
            id="lcd-home"
            class="primary-btn"
            type="button"
            ${!supported || busy ? "disabled" : ""}
          >
            HOME
          </button>
        </div>
      </article>

      <article class="panel">
        <h2>Future LCD widgets</h2>
        <p class="muted">
          Images, text, widgets and animations will use capability-gated
          extension manifests rather than direct uncontrolled HID access.
        </p>
      </article>
    </section>
  `;
}

function renderProfiles(): string {
  const supported = capabilities?.profiles === true;

  return `
    <section class="page">
      <div class="page-heading">
        <div>
          <p class="eyebrow">Configuration</p>
          <h1>Profiles</h1>
          <p>
            Save combinations of RGB, effects, LCD behavior and future input
            mappings.
          </p>
        </div>
        ${badge(
          supported ? "Supported" : "Planned",
          supported ? "good" : "neutral",
        )}
      </div>

      <article class="panel empty-state">
        <div class="placeholder-plus">+</div>
        <h2>Profiles are the next runtime capability</h2>
        <p class="muted">
          The daemon currently advertises profiles=NO, so Studio correctly
          keeps profile writes disabled instead of pretending they exist.
        </p>
      </article>
    </section>
  `;
}

function renderDiagnostics(): string {
  return `
    <section class="page">
      <div class="page-heading">
        <div>
          <p class="eyebrow">Developer tools</p>
          <h1>Diagnostics</h1>
          <p>
            Exact capability contract and safety boundaries reported by al80d.
          </p>
        </div>
        ${badge(`API ${capabilities?.api ?? "—"}`, "good")}
      </div>

      <div class="diagnostic-grid">
        ${metric("Firmware mode",
          capabilities?.firmwareMode ?? "Unknown")}
        ${metric("Daemon",
          capabilities?.daemonVersion ?? "Unknown")}
        ${metric("Manifest",
          capabilities?.extensionManifest ?? "None")}
        ${metric("Raw HID owner", "al80d",
          status?.devnode ?? "No device")}
      </div>

      <article class="panel">
        <h2>Safety contract</h2>
        <div class="safety-grid">
          <div>
            <span>Persistent write</span>
            <strong>${yesNo(capabilities?.persistentWrite)}</strong>
          </div>
          <div>
            <span>EEPROM write</span>
            <strong>${yesNo(capabilities?.eepromWrite)}</strong>
          </div>
          <div>
            <span>QMK flash</span>
            <strong>${yesNo(capabilities?.qmkFlash)}</strong>
          </div>
          <div>
            <span>Audio watcher</span>
            <strong>${yesNo(capabilities?.audioWatch)}</strong>
          </div>
        </div>
      </article>

      <article class="panel code-panel">
        <h2>Live capability object</h2>
        <pre>${esc(JSON.stringify(capabilities, null, 2))}</pre>
      </article>
    </section>
  `;
}

function renderPage(): string {
  switch (view) {
    case "effects":
      return renderEffects();
    case "rgb":
      return renderRgb();
    case "lcd":
      return renderLcd();
    case "profiles":
      return renderProfiles();
    case "diagnostics":
      return renderDiagnostics();
    default:
      return renderDashboard();
  }
}

function render(): void {
  const connected = status?.connected === true;

  app.innerHTML = `
    <div class="studio-shell">
      <aside class="sidebar">
        <div class="brand">
          <div class="brand-mark">A</div>
          <div>
            <strong>AL80 Studio</strong>
            <span>Open hardware control</span>
          </div>
        </div>

        <nav class="nav">
          ${navButton("dashboard", "Dashboard")}
          ${navButton("effects", "Effects")}
          ${navButton("rgb", "RGB")}
          ${navButton("lcd", "LCD")}
          ${navButton("profiles", "Profiles")}
          ${navButton("diagnostics", "Diagnostics")}
        </nav>

        <div class="sidebar-footer">
          <div class="connection-row">
            <span class="connection-dot ${connected ? "online" : ""}"></span>
            <span>${connected ? "AL80 connected" : "AL80 offline"}</span>
          </div>
          <small>al80d single-owner runtime</small>
        </div>
      </aside>

      <main class="workspace">
        ${
          notice
            ? `<div class="notice">${esc(notice)}</div>`
            : ""
        }

        ${renderPage()}
      </main>
    </div>
  `;

  bindEvents();
}

async function refresh(message = ""): Promise<void> {
  try {
    const [nextStatus, nextCaps] = await Promise.all([
      invoke<DeviceStatus>("get_device_status"),
      invoke<Capabilities>("get_capabilities"),
    ]);

    status = nextStatus;
    capabilities = nextCaps;
    notice = message || nextStatus.error || nextCaps.error || "";
  } catch (error) {
    notice = String(error);
  }

  render();
}

async function action(
  operation: () => Promise<void>,
): Promise<void> {
  if (busy) return;

  busy = true;
  notice = "Applying…";
  render();

  try {
    await operation();
  } catch (error) {
    busy = false;
    notice = String(error);
    render();
    return;
  }

  busy = false;
  await refresh("Applied successfully.");
}

function bindEvents(): void {
  document
    .querySelectorAll<HTMLButtonElement>("[data-view]")
    .forEach((button) => {
      button.addEventListener("click", () => {
        view = button.dataset.view as View;
        notice = "";
        render();
      });
    });

  document
    .querySelector<HTMLButtonElement>("#refresh")
    ?.addEventListener("click", () => {
      void refresh("Refreshed.");
    });

  document
    .querySelector<HTMLButtonElement>("#rgb-toggle")
    ?.addEventListener("click", () => {
      const target = !(status?.rgbCoreEnabled === true);

      void action(async () => {
        await invoke<boolean>("set_rgb_core_runtime", {
          enabled: target,
        });
      });
    });

  document
    .querySelector<HTMLButtonElement>("#snake-toggle")
    ?.addEventListener("click", () => {
      const target = !(status?.overlayEnabled === true);

      void action(async () => {
        await invoke<boolean>("set_overlay_runtime", {
          enabled: target,
        });
      });
    });

  document
    .querySelectorAll<HTMLButtonElement>(".lcd-volume")
    .forEach((button) => {
      button.addEventListener("click", () => {
        const percent = Number(button.dataset.percent);

        void action(async () => {
          await invoke<string>("lcd_preview", {
            percent,
            muted: false,
          });
        });
      });
    });

  document
    .querySelector<HTMLButtonElement>("#lcd-mute")
    ?.addEventListener("click", () => {
      void action(async () => {
        await invoke<string>("lcd_preview", {
          percent: 50,
          muted: true,
        });
      });
    });

  document
    .querySelector<HTMLButtonElement>("#lcd-home")
    ?.addEventListener("click", () => {
      void action(async () => {
        await invoke<void>("lcd_home");
      });
    });
}

render();
void refresh();
