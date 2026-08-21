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

interface ExtensionRequirements {
  firmwareMode: "stock" | "extended" | "any";
  capabilities: string[];
}

interface ExtensionActivation {
  enableCommand?: string;
  disableCommand?: string;
  stateField?: "overlayEnabled" | "rgbCoreEnabled";
}

interface ExtensionSafety {
  firmwareFlash: boolean;
  eepromWrite: boolean;
  persistentLcdWrite: boolean;
}

interface ExtensionParameter {
  id: string;
  label: string;
  kind: "range" | "toggle" | "select";
  runtimeBinding: "unavailable" | "future";
  min?: number;
  max?: number;
  step?: number;
  options?: string[];
}

interface ExtensionManifest {
  schemaVersion: 1;
  id: string;
  name: string;
  kind:
    | "runtime-feature"
    | "firmware-effect"
    | "lcd-widget"
    | "profile";
  description: string;
  requires: ExtensionRequirements;
  activation?: ExtensionActivation;
  parameters?: ExtensionParameter[];
  safety: ExtensionSafety;
  source: string;
}

interface ExtensionRegistry {
  schemaVersion: 1;
  generatedBy: string;
  extensions: ExtensionManifest[];
}

interface HostProfile {
  id: string;
  name: string;
  rgbEnabled: boolean;
  overlayEnabled: boolean;
  createdAt: string;
}

const PROFILE_KEY = "al80-studio.host-profiles.v1";

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
let registry: ExtensionRegistry | null = null;
let profiles: HostProfile[] = loadProfiles();
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

function loadProfiles(): HostProfile[] {
  try {
    const raw = localStorage.getItem(PROFILE_KEY);

    if (!raw) {
      return [];
    }

    const parsed = JSON.parse(raw) as unknown;

    if (!Array.isArray(parsed)) {
      return [];
    }

    return parsed.filter((item): item is HostProfile => {
      if (typeof item !== "object" || item === null) {
        return false;
      }

      const profile = item as Partial<HostProfile>;

      return (
        typeof profile.id === "string" &&
        typeof profile.name === "string" &&
        typeof profile.rgbEnabled === "boolean" &&
        typeof profile.overlayEnabled === "boolean" &&
        typeof profile.createdAt === "string"
      );
    });
  } catch {
    return [];
  }
}

function saveProfiles(): void {
  localStorage.setItem(PROFILE_KEY, JSON.stringify(profiles));
}

async function loadRegistry(): Promise<ExtensionRegistry> {
  const response = await fetch("./extensions/registry.json", {
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error(
      `Extension registry failed: HTTP ${response.status}`,
    );
  }

  const value = (await response.json()) as ExtensionRegistry;

  if (value.schemaVersion !== 1 || !Array.isArray(value.extensions)) {
    throw new Error("Invalid extension registry");
  }

  return value;
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

function capabilityValue(name: string): boolean {
  if (!capabilities) return false;

  switch (name) {
    case "matrix_scan":
      return capabilities.matrixScan;
    case "rgb_runtime":
      return capabilities.rgbRuntime;
    case "overlay":
      return capabilities.overlay;
    case "lcd_osd":
      return capabilities.lcdOsd;
    case "audio_watch":
      return capabilities.audioWatch;
    case "profiles":
      return capabilities.profiles;
    default:
      return false;
  }
}

function extensionCompatible(ext: ExtensionManifest): boolean {
  if (!capabilities || capabilities.error) {
    return false;
  }

  const firmware = ext.requires.firmwareMode;

  if (
    firmware !== "any" &&
    firmware.toUpperCase() !== capabilities.firmwareMode.toUpperCase()
  ) {
    return false;
  }

  return ext.requires.capabilities.every(capabilityValue);
}

function extensionSafe(ext: ExtensionManifest): boolean {
  return (
    ext.safety.firmwareFlash === false &&
    ext.safety.eepromWrite === false &&
    ext.safety.persistentLcdWrite === false
  );
}

function extensionActive(ext: ExtensionManifest): boolean {
  switch (ext.activation?.stateField) {
    case "overlayEnabled":
      return status?.overlayEnabled === true;
    case "rgbCoreEnabled":
      return status?.rgbCoreEnabled === true;
    default:
      return false;
  }
}

function extensionCards(): string {
  const effects = (registry?.extensions ?? []).filter(
    (ext) =>
      ext.kind === "firmware-effect" ||
      ext.kind === "runtime-feature",
  );

  if (effects.length === 0) {
    return `
      <article class="panel empty-state">
        <div class="placeholder-plus">+</div>
        <h2>No extensions found</h2>
        <p class="muted">
          Add a manifest under extensions/ and rebuild the registry.
        </p>
      </article>
    `;
  }

  return effects
    .map((ext) => {
      const compatible = extensionCompatible(ext);
      const safe = extensionSafe(ext);
      const active = extensionActive(ext);
      const canToggle =
        compatible &&
        safe &&
        Boolean(ext.activation?.enableCommand) &&
        Boolean(ext.activation?.disableCommand);

      const parameters = ext.parameters ?? [];

      return `
        <article
          class="effect-card ${active ? "effect-active" : ""}"
          data-extension-id="${esc(ext.id)}"
        >
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
                <p class="eyebrow">${esc(ext.kind)}</p>
                <h2>${esc(ext.name)}</h2>
              </div>

              ${badge(
                compatible && safe ? "Compatible" : "Unavailable",
                compatible && safe ? "good" : "warn",
              )}
            </div>

            <p>${esc(ext.description)}</p>

            <div class="chip-row">
              ${ext.requires.capabilities
                .map((cap) => badge(cap))
                .join("")}
              ${badge(ext.requires.firmwareMode)}
              ${safe ? badge("Safe runtime", "good") : badge("Risky", "warn")}
            </div>

            ${
              parameters.length > 0
                ? `
                  <div class="parameter-note">
                    ${parameters.length} parameter definition(s) found.
                    Runtime parameter binding is intentionally not enabled
                    until a hardware command is reverse-engineered and gated.
                  </div>
                `
                : ""
            }

            <div class="manifest-source">
              ${esc(ext.source)}
            </div>

            <div class="effect-actions">
              <div>
                <span class="state-label">Current state</span>
                <strong>${active ? "Enabled" : "Disabled"}</strong>
              </div>

              <button
                class="primary-btn extension-toggle"
                data-extension-id="${esc(ext.id)}"
                type="button"
                ${!canToggle || busy ? "disabled" : ""}
              >
                ${active ? `Disable ${esc(ext.name)}` : `Enable ${esc(ext.name)}`}
              </button>
            </div>
          </div>
        </article>
      `;
    })
    .join("");
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
        ${metric(
          "Connection",
          status?.connected ? "Connected" : "Offline",
          status?.devnode ?? "No device",
        )}
        ${metric("Matrix Scan", scanHz, interval)}
        ${metric(
          "RGB Engine",
          onOff(status?.rgbCoreEnabled),
          "Runtime / volatile",
        )}
        ${metric(
          "Extensions",
          String(registry?.extensions.length ?? 0),
          registry?.generatedBy ?? "Registry unavailable",
        )}
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
          Effects are now described by manifests instead of being permanently
          hardcoded into the frontend.
        </p>
      </article>
    </section>
  `;
}

function renderEffects(): string {
  return `
    <section class="page">
      <div class="page-heading">
        <div>
          <p class="eyebrow">Manifest-driven customization</p>
          <h1>Effects</h1>
          <p>
            Compatible effects are loaded from the generated extension
            registry and capability-checked against the connected keyboard.
          </p>
        </div>

        ${badge(
          `${registry?.extensions.length ?? 0} extension(s)`,
          registry ? "good" : "warn",
        )}
      </div>

      <div class="effect-grid dynamic-effect-grid">
        ${extensionCards()}
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
        ${badge(
          supported ? "Supported" : "Unavailable",
          supported ? "good" : "warn",
        )}
      </div>

      <article class="panel control-panel">
        <div>
          <p class="eyebrow">RGB core</p>
          <h2>${enabled ? "Lighting enabled" : "Lighting disabled"}</h2>
          <p class="muted">
            RGB runtime remains independent from persistent firmware settings.
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
        <h2>Parameter framework</h2>
        <p class="muted">
          Manifest V1 can already describe future range, toggle and select
          parameters. Studio will not send a parameter until its hardware
          command has been reverse-engineered and explicitly allowlisted.
        </p>
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
            Safe volatile previews for the validated display protocol.
          </p>
        </div>
        ${badge(
          supported ? "OSD supported" : "Unavailable",
          supported ? "good" : "warn",
        )}
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
          These buttons change only the keyboard display.
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
    </section>
  `;
}

function renderProfiles(): string {
  const cards =
    profiles.length === 0
      ? `
        <article class="panel empty-state">
          <div class="placeholder-plus">+</div>
          <h2>No host profiles yet</h2>
          <p class="muted">
            Save the current RGB and effect state to create your first one.
          </p>
        </article>
      `
      : profiles
          .map(
            (profile) => `
              <article class="profile-card">
                <div>
                  <p class="eyebrow">Host profile</p>
                  <h2>${esc(profile.name)}</h2>
                  <div class="chip-row">
                    ${badge(`RGB ${profile.rgbEnabled ? "ON" : "OFF"}`)}
                    ${badge(`Snake ${profile.overlayEnabled ? "ON" : "OFF"}`)}
                  </div>
                  <small>${esc(profile.createdAt)}</small>
                </div>

                <div class="profile-actions">
                  <button
                    class="primary-btn profile-apply"
                    data-profile-id="${esc(profile.id)}"
                    type="button"
                    ${busy ? "disabled" : ""}
                  >
                    Apply
                  </button>

                  <button
                    class="secondary-btn profile-delete"
                    data-profile-id="${esc(profile.id)}"
                    type="button"
                    ${busy ? "disabled" : ""}
                  >
                    Delete
                  </button>
                </div>
              </article>
            `,
          )
          .join("");

  return `
    <section class="page">
      <div class="page-heading">
        <div>
          <p class="eyebrow">Configuration</p>
          <h1>Profiles</h1>
          <p>
            Host Profiles V1 save safe runtime state locally in AL80 Studio.
            They do not write EEPROM or persist settings into keyboard flash.
          </p>
        </div>

        <button
          id="profile-save"
          class="primary-btn"
          type="button"
          ${!status?.connected || busy ? "disabled" : ""}
        >
          Save current state
        </button>
      </div>

      <div class="profile-grid">
        ${cards}
      </div>

      <article class="panel">
        <p class="muted">
          al80d still reports <code>profiles=NO</code> because firmware-side
          profiles do not exist yet. Host Profiles are intentionally a
          separate safe layer.
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
            Capability contract, extension registry and safety boundaries.
          </p>
        </div>
        ${badge(`API ${capabilities?.api ?? "—"}`, "good")}
      </div>

      <div class="diagnostic-grid">
        ${metric(
          "Firmware mode",
          capabilities?.firmwareMode ?? "Unknown",
        )}
        ${metric(
          "Daemon",
          capabilities?.daemonVersion ?? "Unknown",
        )}
        ${metric(
          "Extensions",
          String(registry?.extensions.length ?? 0),
          registry?.generatedBy ?? "Unavailable",
        )}
        ${metric(
          "Raw HID owner",
          "al80d",
          status?.devnode ?? "No device",
        )}
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
            <span>Arbitrary extension code</span>
            <strong>No</strong>
          </div>
        </div>
      </article>

      <article class="panel code-panel">
        <h2>Live extension registry</h2>
        <pre>${esc(JSON.stringify(registry, null, 2))}</pre>
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
          <small>
            ${registry?.extensions.length ?? 0} manifest extension(s)
          </small>
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
    const [nextStatus, nextCaps, nextRegistry] = await Promise.all([
      invoke<DeviceStatus>("get_device_status"),
      invoke<Capabilities>("get_capabilities"),
      loadRegistry(),
    ]);

    status = nextStatus;
    capabilities = nextCaps;
    registry = nextRegistry;
    profiles = loadProfiles();

    notice =
      message ||
      nextStatus.error ||
      nextCaps.error ||
      "";
  } catch (error) {
    notice = String(error);
  }

  render();
}

async function action(
  operation: () => Promise<void>,
  successMessage = "Applied successfully.",
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
  await refresh(successMessage);
}

function findExtension(id: string): ExtensionManifest | undefined {
  return registry?.extensions.find((ext) => ext.id === id);
}

function findProfile(id: string): HostProfile | undefined {
  return profiles.find((profile) => profile.id === id);
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
    .querySelectorAll<HTMLButtonElement>(".extension-toggle")
    .forEach((button) => {
      button.addEventListener("click", () => {
        const id = button.dataset.extensionId;

        if (!id) return;

        const ext = findExtension(id);

        if (!ext || !ext.activation) {
          notice = "Extension activation metadata is missing.";
          render();
          return;
        }

        const active = extensionActive(ext);
        const command = active
          ? ext.activation.disableCommand
          : ext.activation.enableCommand;

        if (!command) {
          notice = "This extension does not expose a safe toggle command.";
          render();
          return;
        }

        void action(async () => {
          await invoke<string>("run_safe_extension_command", {
            command,
          });
        }, `${ext.name} updated.`);
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

  document
    .querySelector<HTMLButtonElement>("#profile-save")
    ?.addEventListener("click", () => {
      if (!status?.connected) return;

      const suggested = `Profile ${profiles.length + 1}`;
      const name = window.prompt("Profile name", suggested)?.trim();

      if (!name) return;

      const profile: HostProfile = {
        id: crypto.randomUUID(),
        name,
        rgbEnabled: status.rgbCoreEnabled === true,
        overlayEnabled: status.overlayEnabled === true,
        createdAt: new Date().toLocaleString(),
      };

      profiles = [...profiles, profile];
      saveProfiles();
      notice = `Saved ${name}.`;
      render();
    });

  document
    .querySelectorAll<HTMLButtonElement>(".profile-apply")
    .forEach((button) => {
      button.addEventListener("click", () => {
        const id = button.dataset.profileId;

        if (!id) return;

        const profile = findProfile(id);

        if (!profile) return;

        void action(async () => {
          if (profile.rgbEnabled) {
            await invoke<boolean>("set_rgb_core_runtime", {
              enabled: true,
            });

            await invoke<boolean>("set_overlay_runtime", {
              enabled: profile.overlayEnabled,
            });
          } else {
            await invoke<boolean>("set_overlay_runtime", {
              enabled: profile.overlayEnabled,
            });

            await invoke<boolean>("set_rgb_core_runtime", {
              enabled: false,
            });
          }
        }, `Applied ${profile.name}.`);
      });
    });

  document
    .querySelectorAll<HTMLButtonElement>(".profile-delete")
    .forEach((button) => {
      button.addEventListener("click", () => {
        const id = button.dataset.profileId;

        if (!id) return;

        const profile = findProfile(id);

        if (!profile) return;

        if (!window.confirm(`Delete "${profile.name}"?`)) {
          return;
        }

        profiles = profiles.filter((item) => item.id !== id);
        saveProfiles();
        notice = `Deleted ${profile.name}.`;
        render();
      });
    });
}

render();
void refresh();
