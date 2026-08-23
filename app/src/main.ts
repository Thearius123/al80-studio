import { invoke } from "@tauri-apps/api/core";
import "./style.css";
import {
  bindInputDesignerEvents,
  refreshInputDesigner,
  renderInputDesigner,
} from "./input-designer";

type View =
  | "dashboard"
  | "effects"
  | "creator"
  | "inputs"
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
  lcdFeedback: boolean;
  audioWatch: boolean;
  profiles: boolean;
  extensionManifest: string;
  perKeyRgb: boolean;
  creatorScene: boolean;
  rgbLeds: number;
  keyRgbLeds: number;
  accentRgbLeds: number;
  creatorSceneState: boolean | null;
  inputRouter: boolean;
  inputBindings: number;
  inputActions: number;
  inputRouterState: boolean | null;
  inputEventBridgeHost: boolean;
  inputEventFirmware: boolean;
  inputEventAutoLcd: boolean;
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

interface CreatorKey {
  ledIndex: number;
  matrix: [number, number];
  code: string;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
  rgbX: number;
  rgbY: number;
}

interface CreatorAccent {
  ledIndex: number;
  label: string;
  rgbX: number;
  rgbY: number;
}

interface CreatorControl {
  matrix: [number, number];
  code: string;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
  hasRgb: false;
}

interface CreatorLayout {
  schemaVersion: 1;
  device: string;
  ledCount: 82;
  keyLedCount: 79;
  accentLedCount: 3;
  layoutWidth: number;
  layoutHeight: number;
  keys: CreatorKey[];
  accents: CreatorAccent[];
  controls: CreatorControl[];
}

interface SavedCreatorScene {
  id: string;
  name: string;
  colors: string[];
  createdAt: string;
}

type CreatorTool = "paint" | "select";

interface HostProfile {
  id: string;
  name: string;
  rgbEnabled: boolean;
  overlayEnabled: boolean;
  createdAt: string;
}

const PROFILE_KEY = "al80-studio.host-profiles.v1";
const CREATOR_SCENE_KEY = "al80-studio.creator-scenes.v1";

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
let creatorLayout: CreatorLayout | null = null;
let creatorColors: string[] = Array.from({ length: 82 }, () => "#ffffff");
let creatorPaintColor = "#7c83ff";
let creatorTool: CreatorTool = "paint";
let creatorSelected = new Set<number>();
let creatorHistory: string[][] = [];
let savedCreatorScenes: SavedCreatorScene[] = loadCreatorScenes();
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

function normalizeHexColor(value: string): string {
  const normalized = value.trim().toLowerCase();
  return /^#[0-9a-f]{6}$/.test(normalized) ? normalized : "#000000";
}

function loadCreatorScenes(): SavedCreatorScene[] {
  try {
    const raw = localStorage.getItem(CREATOR_SCENE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((item): item is SavedCreatorScene => {
      if (typeof item !== "object" || item === null) return false;
      const scene = item as Partial<SavedCreatorScene>;
      return typeof scene.id === "string"
        && typeof scene.name === "string"
        && typeof scene.createdAt === "string"
        && Array.isArray(scene.colors)
        && scene.colors.length === 82
        && scene.colors.every((c) => typeof c === "string" && /^#[0-9a-fA-F]{6}$/.test(c));
    }).map((scene) => ({ ...scene, colors: scene.colors.map(normalizeHexColor) }));
  } catch {
    return [];
  }
}

function saveCreatorScenes(): void {
  localStorage.setItem(CREATOR_SCENE_KEY, JSON.stringify(savedCreatorScenes));
}

function creatorSnapshot(): void {
  creatorHistory.push([...creatorColors]);
  if (creatorHistory.length > 30) creatorHistory.shift();
}

function creatorUndo(): void {
  const previous = creatorHistory.pop();
  if (!previous) {
    notice = "Nothing to undo.";
    render();
    return;
  }
  creatorColors = previous;
  notice = "Undo.";
  render();
}

async function loadCreatorLayout(): Promise<CreatorLayout> {
  const response = await fetch("./devices/al80/layout.json", { cache: "no-store" });
  if (!response.ok) throw new Error(`Creator layout failed: HTTP ${response.status}`);
  const value = (await response.json()) as CreatorLayout;
  if (value.schemaVersion !== 1 || value.ledCount !== 82 || value.keyLedCount !== 79 || value.accentLedCount !== 3) {
    throw new Error("Invalid AL80 Creator physical layout");
  }
  return value;
}

function paintCreatorLed(ledIndex: number, color = creatorPaintColor): void {
  if (!Number.isInteger(ledIndex) || ledIndex < 0 || ledIndex >= 82) return;
  creatorColors[ledIndex] = normalizeHexColor(color);
}

function creatorWasdDemo(): void {
  if (!creatorLayout) return;
  creatorSnapshot();
  creatorColors = Array.from({ length: 82 }, () => "#000000");
  const map = new Map(creatorLayout.keys.map((key) => [key.code, key.ledIndex]));
  const demo: Array<[string, string]> = [
    ["KC_W", "#0050ff"], ["KC_A", "#ff0000"], ["KC_S", "#00ff00"], ["KC_D", "#ffbe00"],
    ["KC_LEFT", "#ff0050"], ["KC_DOWN", "#00ff78"], ["KC_RIGHT", "#008cff"], ["KC_UP", "#ffffff"],
  ];
  for (const [code, color] of demo) {
    const led = map.get(code);
    if (led !== undefined) paintCreatorLed(led, color);
  }
  const accentColors = ["#ff00ff", "#00ffff", "#ff5000"];
  creatorLayout.accents.forEach((accent, index) => paintCreatorLed(accent.ledIndex, accentColors[index] ?? "#ffffff"));
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

function renderCreator(): string {
  const supported = capabilities?.perKeyRgb === true
    && capabilities?.creatorScene === true
    && capabilities?.rgbLeds === 82;

  if (!creatorLayout) {
    return `<section class="page"><div class="page-heading"><div><p class="eyebrow">Creator Mode</p><h1>Keyboard Painter</h1><p>Loading AL80 physical LED map…</p></div></div></section>`;
  }

  const width = creatorLayout.layoutWidth;
  const height = creatorLayout.layoutHeight;
  const keys = creatorLayout.keys.map((key) => {
    const color = creatorColors[key.ledIndex] ?? "#000000";
    const selected = creatorSelected.has(key.ledIndex);
    return `<button class="creator-key ${selected ? "selected" : ""}" data-creator-led="${key.ledIndex}" title="${esc(`${key.label} · ${key.code} · LED ${key.ledIndex}`)}" type="button" style="left:${(key.x / width) * 100}%;top:${(key.y / height) * 100}%;width:${(key.w / width) * 100}%;height:${(key.h / height) * 100}%;--creator-key-color:${esc(color)}"><span>${esc(key.label)}</span><small>${key.ledIndex}</small></button>`;
  }).join("");

  const controls = creatorLayout.controls.map((control) => `<div class="creator-key creator-key-no-rgb" title="${esc(`${control.label} · no RGB LED`)}" style="left:${(control.x / width) * 100}%;top:${(control.y / height) * 100}%;width:${(control.w / width) * 100}%;height:${(control.h / height) * 100}%"><span>${esc(control.label)}</span><small>No RGB</small></div>`).join("");

  const accents = creatorLayout.accents.map((accent) => {
    const color = creatorColors[accent.ledIndex] ?? "#000000";
    const selected = creatorSelected.has(accent.ledIndex);
    return `<button class="creator-accent ${selected ? "selected" : ""}" data-creator-led="${accent.ledIndex}" type="button" style="--creator-key-color:${esc(color)}">${esc(accent.label)}<small>LED ${accent.ledIndex}</small></button>`;
  }).join("");

  const saved = savedCreatorScenes.length
    ? savedCreatorScenes.map((scene) => `<article class="creator-saved-scene"><div><strong>${esc(scene.name)}</strong><small>${esc(scene.createdAt)}</small></div><div class="profile-actions"><button class="primary-btn creator-scene-load" data-scene-id="${esc(scene.id)}" type="button" ${busy ? "disabled" : ""}>Load</button><button class="secondary-btn creator-scene-delete" data-scene-id="${esc(scene.id)}" type="button" ${busy ? "disabled" : ""}>Delete</button></div></article>`).join("")
    : `<div class="creator-empty-scenes">No saved scenes yet.</div>`;

  return `<section class="page">
    <div class="page-heading"><div><p class="eyebrow">Per-key RGB Creator</p><h1>Keyboard Painter</h1><p>Paint any of the 79 key LEDs and 3 accent LEDs. Upload is atomic through the physically validated 0x4A protocol and remains RAM-only.</p></div>${badge(supported ? "Creator Protocol Ready" : "Creator unavailable", supported ? "good" : "warn")}</div>
    <article class="panel creator-toolbar"><label class="creator-color-control"><span>Color</span><input id="creator-color" type="color" value="${esc(creatorPaintColor)}"/><code>${esc(creatorPaintColor)}</code></label><div class="creator-tool-group"><button class="secondary-btn creator-tool ${creatorTool === "paint" ? "tool-active" : ""}" data-creator-tool="paint" type="button">Paint</button><button class="secondary-btn creator-tool ${creatorTool === "select" ? "tool-active" : ""}" data-creator-tool="select" type="button">Select</button><button id="creator-apply-selection" class="secondary-btn" type="button" ${creatorSelected.size === 0 ? "disabled" : ""}>Color selected (${creatorSelected.size})</button><button id="creator-clear-selection" class="secondary-btn" type="button" ${creatorSelected.size === 0 ? "disabled" : ""}>Clear selection</button></div></article>
    <article class="panel"><div class="creator-actions"><button id="creator-wasd-demo" class="secondary-btn" type="button">WASD demo</button><button id="creator-fill" class="secondary-btn" type="button">Fill all</button><button id="creator-black" class="secondary-btn" type="button">All off</button><button id="creator-white" class="secondary-btn" type="button">All white</button><button id="creator-undo" class="secondary-btn" type="button" ${creatorHistory.length === 0 ? "disabled" : ""}>Undo</button><button id="creator-save" class="secondary-btn" type="button">Save scene</button><button id="creator-apply" class="primary-btn" type="button" ${!supported || busy ? "disabled" : ""}>Apply to keyboard</button><button id="creator-disable" class="secondary-btn" type="button" ${!supported || busy ? "disabled" : ""}>Exit Creator</button></div></article>
    <article class="panel"><div class="panel-title-row"><div><p class="eyebrow">Exact recovered layout</p><h2>79 RGB keys</h2></div>${badge("Click or drag to paint")}</div><div class="creator-board">${keys}${controls}</div></article>
    <article class="panel"><div class="panel-title-row"><div><p class="eyebrow">Decorative zones</p><h2>Accent LEDs</h2></div>${badge("LED 76 / 77 / 78")}</div><div class="creator-accent-row">${accents}</div></article>
    <article class="panel"><div class="panel-title-row"><div><p class="eyebrow">Local library</p><h2>Saved scenes</h2></div>${badge(`${savedCreatorScenes.length} saved`)}</div><div class="creator-scene-library">${saved}</div></article>
    <article class="panel"><p class="muted">Creator Scene temporarily takes priority over Snake. Exit Creator to return to Snake/normal RGB. Low-battery red indication remains highest priority.</p></article>
  </section>`;
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
    
      <article class="panel">
        <div class="panel-heading">
          <div>
            <p class="eyebrow">Generic transient feedback</p>
            <h2>Typed LCD feedback</h2>
          </div>
        </div>

        <p class="muted">
          Runtime-only 96×160 RGB565 feedback through al80d.
          No LCD media is stored on the keyboard.
        </p>

        ${
          capabilities?.lcdFeedback === true
            ? `
              <div class="control-grid">
                <label>
                  <span>Kind</span>
                  <select id="lcd-feedback-kind">
                    <option value="PROFILE">Profile</option>
                    <option value="ACTION">Action</option>
                    <option value="RGB_BRIGHTNESS">RGB value</option>
                    <option value="RGB_HUE">RGB hue</option>
                    <option value="RGB_SPEED">RGB speed</option>
                    <option value="SNAKE">Snake</option>
                    <option value="SCENE">Scene</option>
                    <option value="ROUTER">Router</option>
                  </select>
                </label>

                <label>
                  <span>Typed value</span>
                  <input
                    id="lcd-feedback-value"
                    value="ON"
                    autocomplete="off"
                    spellcheck="false"
                  />
                </label>
              </div>

              <div class="button-row">
                <button
                  type="button"
                  id="lcd-feedback-preview"
                  class="primary-btn"
                >
                  Preview feedback
                </button>
              </div>
            `
            : `
              <p class="muted">
                Generic LCD feedback requires al80d 0.4.0+.
              </p>
            `
        }

        <p class="muted">
          ${
            capabilities?.inputEventAutoLcd === true
              ? "Automatic per-knob LCD feedback is active through the physically validated 0x4C Event Bridge. Volume/Mute use the actual Fedora audio state; other allowlisted actions use typed automatic LCD feedback."
              : "Typed host LCD preview is available, but this runtime does not advertise automatic per-knob action feedback."
          }
        </p>
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
    case "creator":
      return renderCreator();
    case "inputs":
      return renderInputDesigner(capabilities, busy);
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
          ${navButton("creator", "Creator")}
          ${navButton("inputs", "Inputs")}
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
    const [nextStatus, nextCaps, nextRegistry, nextCreatorLayout] = await Promise.all([
      invoke<DeviceStatus>("get_device_status"),
      invoke<Capabilities>("get_capabilities"),
      loadRegistry(),
      loadCreatorLayout(),
    ]);

    status = nextStatus;
    capabilities = nextCaps;
    registry = nextRegistry;
    creatorLayout = nextCreatorLayout;
    await refreshInputDesigner(nextCaps);
    profiles = loadProfiles();
    savedCreatorScenes = loadCreatorScenes();

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

  bindInputDesignerEvents({
    capabilities,
    busy,
    runAction: action,
    rerender: render,
    setNotice: (message: string) => {
      notice = message;
      render();
    },
  });

  document
    .querySelector<HTMLInputElement>("#creator-color")
    ?.addEventListener("input", (event) => {
      creatorPaintColor = normalizeHexColor((event.currentTarget as HTMLInputElement).value);
      render();
    });

  document.querySelectorAll<HTMLButtonElement>(".creator-tool").forEach((button) => {
    button.addEventListener("click", () => {
      const tool = button.dataset.creatorTool;
      if (tool === "paint" || tool === "select") {
        creatorTool = tool;
        render();
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-creator-led]").forEach((button) => {
    const interact = (snapshot: boolean) => {
      const raw = button.dataset.creatorLed;
      if (!raw) return;
      const led = Number(raw);
      if (!Number.isInteger(led) || led < 0 || led >= 82) return;
      if (creatorTool === "select") {
        if (creatorSelected.has(led)) creatorSelected.delete(led);
        else creatorSelected.add(led);
        render();
        return;
      }
      if (snapshot) creatorSnapshot();
      paintCreatorLed(led);
      render();
    };

    button.addEventListener("pointerdown", (event) => {
      if (event.button !== 0) return;
      event.preventDefault();
      interact(true);
    });
    button.addEventListener("pointerenter", (event) => {
      if (creatorTool !== "paint" || (event.buttons & 1) !== 1) return;
      interact(false);
    });
  });

  document.querySelector<HTMLButtonElement>("#creator-apply-selection")?.addEventListener("click", () => {
    if (creatorSelected.size === 0) return;
    creatorSnapshot();
    creatorSelected.forEach((led) => paintCreatorLed(led));
    render();
  });

  document.querySelector<HTMLButtonElement>("#creator-clear-selection")?.addEventListener("click", () => {
    creatorSelected.clear();
    render();
  });

  document.querySelector<HTMLButtonElement>("#creator-wasd-demo")?.addEventListener("click", () => {
    creatorWasdDemo();
    notice = "WASD demonstration loaded locally.";
    render();
  });

  document.querySelector<HTMLButtonElement>("#creator-fill")?.addEventListener("click", () => {
    creatorSnapshot();
    creatorColors = Array.from({ length: 82 }, () => creatorPaintColor);
    render();
  });

  document.querySelector<HTMLButtonElement>("#creator-black")?.addEventListener("click", () => {
    creatorSnapshot();
    creatorColors = Array.from({ length: 82 }, () => "#000000");
    render();
  });

  document.querySelector<HTMLButtonElement>("#creator-white")?.addEventListener("click", () => {
    creatorSnapshot();
    creatorColors = Array.from({ length: 82 }, () => "#ffffff");
    render();
  });

  document.querySelector<HTMLButtonElement>("#creator-undo")?.addEventListener("click", creatorUndo);

  document.querySelector<HTMLButtonElement>("#creator-save")?.addEventListener("click", () => {
    const suggested = `Scene ${savedCreatorScenes.length + 1}`;
    const name = window.prompt("Scene name", suggested)?.trim();
    if (!name) return;
    savedCreatorScenes = [...savedCreatorScenes, {
      id: crypto.randomUUID(),
      name,
      colors: [...creatorColors],
      createdAt: new Date().toLocaleString(),
    }];
    saveCreatorScenes();
    notice = `Saved ${name}.`;
    render();
  });

  document.querySelectorAll<HTMLButtonElement>(".creator-scene-load").forEach((button) => {
    button.addEventListener("click", () => {
      const id = button.dataset.sceneId;
      if (!id) return;
      const scene = savedCreatorScenes.find((item) => item.id === id);
      if (!scene) return;
      creatorSnapshot();
      creatorColors = scene.colors.map(normalizeHexColor);
      notice = `Loaded ${scene.name}.`;
      render();
    });
  });

  document.querySelectorAll<HTMLButtonElement>(".creator-scene-delete").forEach((button) => {
    button.addEventListener("click", () => {
      const id = button.dataset.sceneId;
      if (!id) return;
      const scene = savedCreatorScenes.find((item) => item.id === id);
      if (!scene || !window.confirm(`Delete "${scene.name}"?`)) return;
      savedCreatorScenes = savedCreatorScenes.filter((item) => item.id !== id);
      saveCreatorScenes();
      notice = `Deleted ${scene.name}.`;
      render();
    });
  });

  document.querySelector<HTMLButtonElement>("#creator-apply")?.addEventListener("click", () => {
    void action(async () => {
      await invoke<string>("apply_creator_scene", { colors: creatorColors });
    }, "Creator Scene applied to keyboard.");
  });

  document.querySelector<HTMLButtonElement>("#creator-disable")?.addEventListener("click", () => {
    void action(async () => {
      await invoke<string>("disable_creator_scene");
    }, "Creator Scene disabled; normal effects restored.");
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
    .querySelector<HTMLButtonElement>(
      "#lcd-feedback-preview",
    )
    ?.addEventListener("click", async () => {
      const kind =
        document
          .querySelector<HTMLSelectElement>(
            "#lcd-feedback-kind",
          )
          ?.value ?? "";

      const value =
        document
          .querySelector<HTMLInputElement>(
            "#lcd-feedback-value",
          )
          ?.value.trim() ?? "";

      try {
        const result = await invoke<string>(
          "lcd_feedback",
          {
            kind,
            value,
          },
        );

        notice = result;
        render();
      } catch (error) {
        notice = String(error);
        render();
      }
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
