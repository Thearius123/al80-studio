import {
  CREATOR_EFFECTS,
  type CreatorEffectId,
  renderCreatorEffectFrame,
} from "./creator-effects";
import { invoke } from "@tauri-apps/api/core";
import "./style.css";
import {
  bindInputDesignerEvents,
  getCurrentInputDraftForHost,
  getSavedInputProfilesForHost,
  hydrateInputProfilesFromHost,
  refreshInputDesigner,
  renderInputDesigner,
  replaceInputDraftFromHost,
  type HostProfileInputBinding,
} from "./input-designer";


interface LiveRgbTelemetry {
  version: number;
  source: string;
  frameValid: boolean;
  rgbCoreEnabled: boolean;
  overlayEnabled: boolean;
  creatorSceneEnabled: boolean;
  colors: string[];
}

interface LcdLogicalStatus {
  mode: string;
  generation: number;
  percent: number | null;
  muted: boolean;
  kind: string | null;
  value: string | null;
}

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

interface HostProfileCreatorScene {
  name: string;
  colors: string[];
}

interface HostProfileInputSnapshot {
  name: string;
  bindings: HostProfileInputBinding[];
}

interface HostProfile {
  schemaVersion: 1 | 2;
  id: string;
  name: string;
  rgbEnabled: boolean;
  overlayEnabled: boolean;
  creatorScene?: HostProfileCreatorScene | null;
  inputProfile?: HostProfileInputSnapshot | null;
  createdAt: string;
}

const PROFILE_KEY = "al80-studio.host-profiles.v1";
const CREATOR_SCENE_KEY = "al80-studio.creator-scenes.v1";
const HOST_LIBRARY_HOST_PROFILES = "host-profiles-v1";
const HOST_LIBRARY_CREATOR_SCENES = "creator-scenes-v1";

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
type CreatorViewMode = "top" | "studio3d";
type CreatorScaleMode = "fit" | "actual";

let creatorViewMode: CreatorViewMode = "studio3d";

/* AL80 LIVE DIGITAL TWIN V1 */
let liveRgbTelemetry: LiveRgbTelemetry | null = null;
let liveLcdStatus: LcdLogicalStatus | null = null;
let liveTelemetryTimer: number | null = null;
let liveTelemetryBusy = false;
let lastRenderedView: View | null = null;

let creatorOrbitX = 9;
let creatorOrbitY = -1.8;
let creatorOrbitZoom = 1;
let creatorOrbitDragging = false;
let creatorOrbitPointerId: number | null = null;
let creatorOrbitLastX = 0;
let creatorOrbitLastY = 0;

let creatorScaleMode: CreatorScaleMode = "fit";
let creatorMirrorExact = false;
let creatorInputSource = "draft";
let creatorEffectId: CreatorEffectId = "snake";
let creatorEffectPrimary = "#7c83ff";
let creatorEffectSecondary = "#000000";
let creatorEffectSpeed = 3;
let creatorEffectTail = 8;
let creatorEffectPhase = 0;
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
  const json = JSON.stringify(savedCreatorScenes);
  localStorage.setItem(CREATOR_SCENE_KEY, json);

  void invoke<string>("write_host_library", {
    library: HOST_LIBRARY_CREATOR_SCENES,
    json,
  }).catch((error) => {
    console.error("Creator scene host persistence failed", error);
  });
}

function creatorSnapshot(): void {
  creatorMirrorExact = false;
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

function validProfileCreatorScene(
  value: unknown,
): value is HostProfileCreatorScene {
  if (typeof value !== "object" || value === null) return false;

  const scene = value as Partial<HostProfileCreatorScene>;

  return (
    typeof scene.name === "string" &&
    Array.isArray(scene.colors) &&
    scene.colors.length === 82 &&
    scene.colors.every(
      (color) =>
        typeof color === "string" &&
        /^#[0-9a-fA-F]{6}$/.test(color),
    )
  );
}

function validProfileInputSnapshot(
  value: unknown,
): value is HostProfileInputSnapshot {
  if (typeof value !== "object" || value === null) return false;

  const input = value as Partial<HostProfileInputSnapshot>;

  if (
    typeof input.name !== "string" ||
    !Array.isArray(input.bindings) ||
    input.bindings.length === 0 ||
    input.bindings.length > 12
  ) {
    return false;
  }

  return input.bindings.every((binding) => {
    if (typeof binding !== "object" || binding === null) return false;

    const item = binding as Partial<HostProfileInputBinding>;

    return (
      Number.isInteger(item.event) &&
      Number.isInteger(item.trigger) &&
      Number.isInteger(item.triggerA) &&
      Number.isInteger(item.triggerB) &&
      Number.isInteger(item.action)
    );
  });
}

function loadProfiles(): HostProfile[] {
  try {
    const raw = localStorage.getItem(PROFILE_KEY);

    if (!raw) return [];

    const parsed = JSON.parse(raw) as unknown;

    if (!Array.isArray(parsed)) return [];

    return parsed.flatMap((item): HostProfile[] => {
      if (typeof item !== "object" || item === null) return [];

      const profile = item as Partial<HostProfile>;

      if (
        typeof profile.id !== "string" ||
        typeof profile.name !== "string" ||
        typeof profile.rgbEnabled !== "boolean" ||
        typeof profile.overlayEnabled !== "boolean" ||
        typeof profile.createdAt !== "string"
      ) {
        return [];
      }

      if (profile.schemaVersion === 2) {
        const creatorScene =
          profile.creatorScene === null
            ? null
            : validProfileCreatorScene(profile.creatorScene)
              ? {
                  name: profile.creatorScene.name,
                  colors: profile.creatorScene.colors.map(normalizeHexColor),
                }
              : undefined;

        const inputProfile =
          profile.inputProfile === null
            ? null
            : validProfileInputSnapshot(profile.inputProfile)
              ? {
                  name: profile.inputProfile.name,
                  bindings: profile.inputProfile.bindings.map((binding) => ({
                    ...binding,
                  })),
                }
              : undefined;

        if (creatorScene === undefined || inputProfile === undefined) {
          return [];
        }

        return [
          {
            schemaVersion: 2,
            id: profile.id,
            name: profile.name,
            rgbEnabled: profile.rgbEnabled,
            overlayEnabled: profile.overlayEnabled,
            creatorScene,
            inputProfile,
            createdAt: profile.createdAt,
          },
        ];
      }

      // Backward-compatible Host Profiles V1 keep their historical
      // behavior: they change only RGB + overlay and preserve Creator/Input.
      return [
        {
          schemaVersion: 1,
          id: profile.id,
          name: profile.name,
          rgbEnabled: profile.rgbEnabled,
          overlayEnabled: profile.overlayEnabled,
          createdAt: profile.createdAt,
        },
      ];
    });
  } catch {
    return [];
  }
}

function saveProfiles(): void {
  const json = JSON.stringify(profiles);
  localStorage.setItem(PROFILE_KEY, json);

  void invoke<string>("write_host_library", {
    library: HOST_LIBRARY_HOST_PROFILES,
    json,
  }).catch((error) => {
    console.error("Host profile persistence failed", error);
  });
}

async function hydrateMainHostLibraries(): Promise<void> {
  const [creatorHostJson, profileHostJson] = await Promise.all([
    invoke<string | null>("read_host_library", {
      library: HOST_LIBRARY_CREATOR_SCENES,
    }),
    invoke<string | null>("read_host_library", {
      library: HOST_LIBRARY_HOST_PROFILES,
    }),
  ]);

  if (creatorHostJson !== null) {
    localStorage.setItem(CREATOR_SCENE_KEY, creatorHostJson);
    savedCreatorScenes = loadCreatorScenes();

    const normalized = JSON.stringify(savedCreatorScenes);

    if (normalized !== creatorHostJson) {
      localStorage.setItem(CREATOR_SCENE_KEY, normalized);
      await invoke<string>("write_host_library", {
        library: HOST_LIBRARY_CREATOR_SCENES,
        json: normalized,
      });
    }
  } else {
    savedCreatorScenes = loadCreatorScenes();

    const migrated = JSON.stringify(savedCreatorScenes);
    localStorage.setItem(CREATOR_SCENE_KEY, migrated);

    await invoke<string>("write_host_library", {
      library: HOST_LIBRARY_CREATOR_SCENES,
      json: migrated,
    });
  }

  if (profileHostJson !== null) {
    localStorage.setItem(PROFILE_KEY, profileHostJson);
    profiles = loadProfiles();

    const normalized = JSON.stringify(profiles);

    if (normalized !== profileHostJson) {
      localStorage.setItem(PROFILE_KEY, normalized);
      await invoke<string>("write_host_library", {
        library: HOST_LIBRARY_HOST_PROFILES,
        json: normalized,
      });
    }
  } else {
    profiles = loadProfiles();

    const migrated = JSON.stringify(profiles);
    localStorage.setItem(PROFILE_KEY, migrated);

    await invoke<string>("write_host_library", {
      library: HOST_LIBRARY_HOST_PROFILES,
      json: migrated,
    });
  }
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


function creatorOrbitClamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

function applyCreatorOrbitTransform(): void {
  const shell = document.querySelector<HTMLElement>(".creator-keyboard-shell");
  if (!shell) return;

  shell.style.setProperty("--creator-orbit-x", `${creatorOrbitX.toFixed(2)}deg`);
  shell.style.setProperty("--creator-orbit-y", `${creatorOrbitY.toFixed(2)}deg`);
  shell.style.setProperty("--creator-orbit-zoom", creatorOrbitZoom.toFixed(4));
}

function renderLiveKeyboardTwin(): string {
  if (!creatorLayout) {
    return `<div class="live-twin-empty">Recovered AL80 layout is not loaded yet.</div>`;
  }

  const colors = liveRgbTelemetry?.colors ?? [];
  const width = creatorLayout.layoutWidth;
  const height = creatorLayout.layoutHeight;

  const keys = creatorLayout.keys
    .map((key) => {
      const color = colors[key.ledIndex] ?? "#18212c";
      return `<div
        class="live-twin-key"
        data-live-led="${key.ledIndex}"
        title="${esc(`${key.label} · LED ${key.ledIndex}`)}"
        style="left:${(key.x / width) * 100}%;top:${(key.y / height) * 100}%;width:${(key.w / width) * 100}%;height:${(key.h / height) * 100}%;--live-led-color:${esc(color)}"
      ><span>${esc(key.label)}</span></div>`;
    })
    .join("");

  const controls = creatorLayout.controls
    .map(
      (control) => `<div
        class="live-twin-key live-twin-no-rgb"
        style="left:${(control.x / width) * 100}%;top:${(control.y / height) * 100}%;width:${(control.w / width) * 100}%;height:${(control.h / height) * 100}%"
      ><span>${esc(control.label)}</span></div>`,
    )
    .join("");

  const accents = creatorLayout.accents
    .map((accent) => {
      const color = colors[accent.ledIndex] ?? "#18212c";
      return `<span
        class="live-twin-accent"
        data-live-led="${accent.ledIndex}"
        style="--live-led-color:${esc(color)}"
      >${esc(accent.label)} · ${accent.ledIndex}</span>`;
    })
    .join("");

  return `
    <div class="live-twin-board">${keys}${controls}</div>
    <div class="live-twin-accents">${accents}</div>
  `;
}

function renderLiveLcdMirror(): string {
  const lcd = liveLcdStatus;

  if (!lcd) {
    return `
      <div class="live-lcd-screen">
        <span class="live-lcd-kicker">HOST LOGICAL MIRROR</span>
        <strong id="live-lcd-mode">Awaiting status</strong>
        <span id="live-lcd-value">—</span>
        <small id="live-lcd-detail">No LCD semantic state received yet.</small>
      </div>
    `;
  }

  const value =
    lcd.mode === "VOLUME" || lcd.mode === "MUTE"
      ? `${lcd.percent ?? 0}%`
      : lcd.mode === "FEEDBACK"
        ? `${lcd.kind ?? "FEEDBACK"} · ${lcd.value ?? "—"}`
        : "Normal keyboard screen";

  const detail =
    lcd.mode === "HOME"
      ? "HOME semantic state"
      : lcd.mode === "MUTE"
        ? "Host audio is muted"
        : lcd.mode === "VOLUME"
          ? "Host volume OSD"
          : "Typed transient feedback";

  return `
    <div class="live-lcd-screen">
      <span class="live-lcd-kicker">HOST LOGICAL MIRROR</span>
      <strong id="live-lcd-mode">${esc(lcd.mode)}</strong>
      <span id="live-lcd-value">${esc(value)}</span>
      <small id="live-lcd-detail">${esc(detail)} · generation ${lcd.generation}</small>
    </div>
  `;
}

function updateLiveTelemetryDom(): void {
  const colors = liveRgbTelemetry?.colors ?? [];

  document
    .querySelectorAll<HTMLElement>("[data-live-led]")
    .forEach((element) => {
      const raw = element.dataset.liveLed;
      if (!raw) return;
      const index = Number(raw);
      if (!Number.isInteger(index) || index < 0 || index >= colors.length) return;
      element.style.setProperty("--live-led-color", colors[index]);
    });

  const source = document.querySelector<HTMLElement>("#live-rgb-source");
  if (source) {
    source.textContent = liveRgbTelemetry?.source ?? "OFFLINE";
  }

  const validity = document.querySelector<HTMLElement>("#live-rgb-validity");
  if (validity) {
    validity.textContent = liveRgbTelemetry?.frameValid === true
      ? "Firmware frame"
      : liveRgbTelemetry
        ? "Native frame unavailable"
        : "No telemetry";
  }

  const lcdMode = document.querySelector<HTMLElement>("#live-lcd-mode");
  const lcdValue = document.querySelector<HTMLElement>("#live-lcd-value");
  const lcdDetail = document.querySelector<HTMLElement>("#live-lcd-detail");

  if (lcdMode && lcdValue && lcdDetail && liveLcdStatus) {
    const lcd = liveLcdStatus;
    lcdMode.textContent = lcd.mode;

    if (lcd.mode === "VOLUME" || lcd.mode === "MUTE") {
      lcdValue.textContent = `${lcd.percent ?? 0}%`;
    } else if (lcd.mode === "FEEDBACK") {
      lcdValue.textContent = `${lcd.kind ?? "FEEDBACK"} · ${lcd.value ?? "—"}`;
    } else {
      lcdValue.textContent = "Normal keyboard screen";
    }

    lcdDetail.textContent =
      `${lcd.mode === "HOME" ? "HOME semantic state" : "Host-driven semantic state"} · generation ${lcd.generation}`;
  }
}

async function refreshLiveTelemetry(): Promise<void> {
  if (liveTelemetryBusy) return;
  if (view !== "dashboard" && view !== "creator" && view !== "lcd") return;

  liveTelemetryBusy = true;

  try {
    const [rgbResult, lcdResult] = await Promise.allSettled([
      invoke<LiveRgbTelemetry>("get_live_rgb_telemetry"),
      invoke<LcdLogicalStatus>("get_lcd_logical_status"),
    ]);

    liveRgbTelemetry =
      rgbResult.status === "fulfilled" ? rgbResult.value : null;

    if (lcdResult.status === "fulfilled") {
      liveLcdStatus = lcdResult.value;
    }

    updateLiveTelemetryDom();
  } finally {
    liveTelemetryBusy = false;
  }
}

function startLiveTelemetryLoop(): void {
  if (liveTelemetryTimer !== null) return;

  liveTelemetryTimer = window.setInterval(() => {
    void refreshLiveTelemetry();
  }, 300);

  void refreshLiveTelemetry();
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

      <article class="panel live-twin-panel">
        <div class="panel-title-row">
          <div>
            <p class="eyebrow">Physical telemetry</p>
            <h2>Live AL80 RGB twin</h2>
            <p class="muted">
              Firmware-backed colors for Snake, Creator Scene, and the
              low-battery safety frame. Native QMK base effects fail closed
              instead of being simulated.
            </p>
          </div>
          <div class="live-twin-status">
            <span id="live-rgb-source">${esc(liveRgbTelemetry?.source ?? "OFFLINE")}</span>
            <span id="live-rgb-validity">${
              liveRgbTelemetry?.frameValid === true
                ? "Firmware frame"
                : liveRgbTelemetry
                  ? "Native frame unavailable"
                  : "No telemetry"
            }</span>
          </div>
        </div>
        <div class="live-twin-keyboard-wrap">
          ${renderLiveKeyboardTwin()}
        </div>
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

function syncCreatorViewport(recenter = true): void {
  const stage = document.querySelector<HTMLElement>(".creator-board-stage");
  const shell = document.querySelector<HTMLElement>(".creator-keyboard-shell");

  if (!stage || !shell) return;

  applyCreatorOrbitTransform();

  if (creatorScaleMode === "actual") {
    shell.style.removeProperty("--creator-auto-scale");

    if (recenter) {
      requestAnimationFrame(() => {
        stage.scrollLeft = Math.max(
          0,
          (stage.scrollWidth - stage.clientWidth) / 2,
        );
        stage.scrollTop = Math.max(
          0,
          (stage.scrollHeight - stage.clientHeight) / 2,
        );
      });
    }

    return;
  }

  requestAnimationFrame(() => {
    const style = getComputedStyle(stage);
    const padX =
      Number.parseFloat(style.paddingLeft) +
      Number.parseFloat(style.paddingRight);
    const padY =
      Number.parseFloat(style.paddingTop) +
      Number.parseFloat(style.paddingBottom);

    const availableWidth = Math.max(1, stage.clientWidth - padX);
    const availableHeight = Math.max(1, stage.clientHeight - padY);
    const naturalWidth = Math.max(1, shell.offsetWidth);
    const naturalHeight = Math.max(1, shell.offsetHeight);

    const perspectiveReserve =
      creatorViewMode === "studio3d" ? 0.90 : 0.97;

    const scale = Math.min(
      1,
      (availableWidth / naturalWidth) * perspectiveReserve,
      (availableHeight / naturalHeight) * perspectiveReserve,
    );

    shell.style.setProperty(
      "--creator-auto-scale",
      scale.toFixed(4),
    );

    if (recenter) {
      stage.scrollLeft = 0;
      stage.scrollTop = 0;
    }
  });
}

function scheduleCreatorViewportSync(recenter = true): void {
  syncCreatorViewport(recenter);
  requestAnimationFrame(() => syncCreatorViewport(false));
}

function renderCreator(): string {
  const supported = capabilities?.perKeyRgb === true
    && capabilities?.creatorScene === true
    && capabilities?.rgbLeds === 82;

  const unifiedInputProfiles = getSavedInputProfilesForHost();
  const unifiedInputOptions = unifiedInputProfiles
    .map(
      (profile) =>
        `<option value="saved:${esc(profile.id)}" ${
          creatorInputSource === `saved:${profile.id}` ? "selected" : ""
        }>${esc(profile.name)}</option>`,
    )
    .join("");

  const creatorLive =
    capabilities?.creatorSceneState === true;
  const inputLive =
    capabilities?.inputRouterState === true;
  const autoLcdReady =
    capabilities?.inputEventAutoLcd === true;

  const creatorEffectOptions = CREATOR_EFFECTS.map(
    (effect) =>
      `<option value="${effect.id}" ${
        creatorEffectId === effect.id ? "selected" : ""
      }>${esc(effect.name)}</option>`,
  ).join("");

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
      <article class="panel">
        <div class="panel-title-row">
          <div>
            <p class="eyebrow">Creator Effect Engine V1</p>
            <h2>Effect Lab · preview only</h2>
          </div>
          ${badge("Host renderer")}
        </div>

        <div class="control-grid">
          <label>
            <span>Effect</span>
            <select id="creator-effect-kind">
              ${creatorEffectOptions}
            </select>
          </label>

          <label>
            <span>Primary</span>
            <input
              id="creator-effect-primary"
              type="color"
              value="${esc(creatorEffectPrimary)}"
            />
          </label>

          <label>
            <span>Secondary</span>
            <input
              id="creator-effect-secondary"
              type="color"
              value="${esc(creatorEffectSecondary)}"
            />
          </label>

          <label>
            <span>Speed 1–10</span>
            <input
              id="creator-effect-speed"
              type="range"
              min="1"
              max="10"
              step="1"
              value="${creatorEffectSpeed}"
            />
          </label>

          <label>
            <span>Tail 1–32</span>
            <input
              id="creator-effect-tail"
              type="range"
              min="1"
              max="32"
              step="1"
              value="${creatorEffectTail}"
            />
          </label>

          <label>
            <span>Phase</span>
            <input
              id="creator-effect-phase"
              type="range"
              min="0"
              max="100"
              step="1"
              value="${Math.round(creatorEffectPhase * 100)}"
            />
          </label>
        </div>

        <div class="button-row">
          <button
            id="creator-effect-preview"
            class="primary-btn"
            type="button"
          >
            Render preview frame
          </button>
        </div>

        <p class="muted">
          Preview modifies only the local 82-LED painter buffer. It does not
          stream frames to the keyboard and does not call any new device API.
          You can inspect or edit the generated frame, then use the existing
          validated Creator Scene Apply manually if desired.
        </p>
      </article>

      <article class="panel">
        <div class="panel-title-row">
          <div>
            <p class="eyebrow">Unified Creator Session</p>
            <h2>Scene + Input behavior</h2>
          </div>
          ${badge(
            creatorLive && inputLive
              ? "Creator + Input live"
              : creatorLive
                ? "Creator live"
                : inputLive
                  ? "Input live"
                  : "Workspace idle",
            creatorLive || inputLive ? "good" : "neutral",
          )}
        </div>

        <div class="control-grid">
          <label>
            <span>Input behavior</span>
            <select id="creator-input-source">
              <option value="off" ${
                creatorInputSource === "off" ? "selected" : ""
              }>Router OFF</option>
              <option value="draft" ${
                creatorInputSource === "draft" ? "selected" : ""
              }>Current Input Designer draft</option>
              ${unifiedInputOptions}
            </select>
          </label>
        </div>

        <div class="button-row">
          <button
            id="creator-apply-unified"
            class="primary-btn"
            type="button"
            ${!supported || busy ? "disabled" : ""}
          >
            Apply unified workspace
          </button>

          <button
            id="creator-exit-unified"
            class="secondary-btn"
            type="button"
            ${!supported || busy ? "disabled" : ""}
          >
            Exit unified workspace
          </button>
        </div>

        <div class="input-safety-grid">
          <span>
            <strong>${creatorLive ? "ON" : "OFF"}</strong>
            Creator Scene
          </span>
          <span>
            <strong>${inputLive ? "ON" : "OFF"}</strong>
            Input Router
          </span>
          <span>
            <strong>${autoLcdReady ? "YES" : "NO"}</strong>
            Automatic LCD
          </span>
        </div>

        <p class="muted">
          Apply sends the current 82-LED painting and the selected typed
          Input behavior through the existing al80d APIs. Current draft uses
          the bindings already designed in Inputs. Exit disables Creator Scene
          and Input Router together, returning to normal RGB/Snake behavior.
        </p>
      </article>

    <div class="page-heading creator-premium-heading"><div><p class="eyebrow">AL80 Creator Studio</p><h1>Design your keyboard</h1><p>Create lighting visually, refine every key, save your work locally, then apply only when you choose. The validated 82-LED scene remains volatile RAM.</p></div><div class="creator-heading-status">${badge("Local canvas", "neutral")}${badge(supported ? "Keyboard ready" : "Keyboard unavailable", supported ? "good" : "warn")}</div></div>
    <article class="panel creator-live-mirror">
      <div class="creator-live-mirror-copy">
        <div class="creator-live-title-row">
          <div>
            <p class="eyebrow">Digital Twin · Live Known State</p>
            <h2>${creatorMirrorExact && capabilities?.creatorSceneState === true ? "Exact Studio-applied scene" : "Host-known device state"}</h2>
          </div>
          ${badge(creatorMirrorExact && capabilities?.creatorSceneState === true ? "Exact scene mirror" : "No per-key readback", creatorMirrorExact && capabilities?.creatorSceneState === true ? "good" : "neutral")}
        </div>
        <p class="muted">
          The protocol exposes live feature state, but not arbitrary physical per-key RGB readback.
          After this Studio successfully applies the current Creator frame, the canvas is an exact
          session mirror until you edit it again. Otherwise the state rail below is authoritative.
        </p>
      </div>
      <div class="creator-live-state-grid">
        <div class="creator-live-state ${capabilities?.rgbState === true ? "is-live" : ""}"><span>RGB CORE</span><strong>${capabilities?.rgbState === true ? "ON" : capabilities?.rgbState === false ? "OFF" : "UNKNOWN"}</strong></div>
        <div class="creator-live-state ${capabilities?.overlayState === true ? "is-live" : ""}"><span>SNAKE / OVERLAY</span><strong>${capabilities?.overlayState === true ? "ON" : capabilities?.overlayState === false ? "OFF" : "UNKNOWN"}</strong></div>
        <div class="creator-live-state ${capabilities?.creatorSceneState === true ? "is-live" : ""}"><span>CREATOR SCENE</span><strong>${capabilities?.creatorSceneState === true ? "ON" : capabilities?.creatorSceneState === false ? "OFF" : "UNKNOWN"}</strong></div>
        <div class="creator-live-state ${capabilities?.inputRouterState === true ? "is-live" : ""}"><span>INPUT ROUTER</span><strong>${capabilities?.inputRouterState === true ? "ON" : capabilities?.inputRouterState === false ? "OFF" : "UNKNOWN"}</strong></div>
        <div class="creator-live-state ${capabilities?.lcdOsd === true ? "is-ready" : ""}"><span>LCD TRANSPORT</span><strong>${capabilities?.lcdOsd === true ? "READY" : "UNAVAILABLE"}</strong></div>
        <div class="creator-live-state ${capabilities?.lcdFeedback === true && capabilities?.inputEventAutoLcd === true ? "is-ready" : ""}"><span>AUTO LCD</span><strong>${capabilities?.lcdFeedback === true && capabilities?.inputEventAutoLcd === true ? "READY" : "LIMITED"}</strong></div>
      </div>
    </article>

    <div class="creator-flow-strip" aria-label="Creator workflow">
      <div class="creator-flow-step active"><span>1</span><div><strong>Create</strong><small>Paint or generate</small></div></div>
      <div class="creator-flow-line"></div>
      <div class="creator-flow-step"><span>2</span><div><strong>Refine</strong><small>Select and edit</small></div></div>
      <div class="creator-flow-line"></div>
      <div class="creator-flow-step"><span>3</span><div><strong>Save</strong><small>Keep it on this PC</small></div></div>
      <div class="creator-flow-line"></div>
      <div class="creator-flow-step device-step"><span>4</span><div><strong>Apply</strong><small>Explicit keyboard action</small></div></div>
    </div>
    <article class="panel creator-toolbar creator-command-center">
      <div class="creator-command-block color-block">
        <span class="creator-command-label">Paint color</span>
        <label class="creator-color-control creator-premium-color">
          <input id="creator-color" type="color" value="${esc(creatorPaintColor)}"/>
          <span class="creator-color-swatch-copy"><strong>Current color</strong><code>${esc(creatorPaintColor)}</code></span>
        </label>
      </div>
      <div class="creator-command-divider"></div>
      <div class="creator-command-block">
        <span class="creator-command-label">Tool</span>
        <div class="creator-segmented-control">
          <button class="secondary-btn creator-tool ${creatorTool === "paint" ? "tool-active" : ""}" data-creator-tool="paint" type="button"><span class="tool-glyph">✦</span>Paint</button>
          <button class="secondary-btn creator-tool ${creatorTool === "select" ? "tool-active" : ""}" data-creator-tool="select" type="button"><span class="tool-glyph">◇</span>Select</button>
        </div>
      </div>
      <div class="creator-command-divider"></div>
      <div class="creator-command-block creator-selection-block">
        <span class="creator-command-label">Selection</span>
        <div class="creator-tool-group">
          <button id="creator-apply-selection" class="secondary-btn" type="button" ${creatorSelected.size === 0 ? "disabled" : ""}>Color ${creatorSelected.size || "selected"}</button>
          <button id="creator-clear-selection" class="secondary-btn" type="button" ${creatorSelected.size === 0 ? "disabled" : ""}>Clear</button>
        </div>
      </div>
    </article>
    <article class="panel creator-action-dock">
      <div class="creator-action-group">
        <span class="creator-command-label">Quick canvas</span>
        <div class="creator-actions">
          <button id="creator-wasd-demo" class="secondary-btn" type="button">WASD preset</button>
          <button id="creator-fill" class="secondary-btn" type="button">Fill</button>
          <button id="creator-black" class="secondary-btn" type="button">Lights off</button>
          <button id="creator-white" class="secondary-btn" type="button">White</button>
          <button id="creator-undo" class="secondary-btn" type="button" ${creatorHistory.length === 0 ? "disabled" : ""}>Undo</button>
        </div>
      </div>
      <div class="creator-action-separator"></div>
      <div class="creator-action-group save-group">
        <span class="creator-command-label">Local</span>
        <button id="creator-save" class="secondary-btn creator-save-premium" type="button">Save scene</button>
      </div>
      <div class="creator-action-separator"></div>
      <div class="creator-action-group device-actions">
        <span class="creator-command-label">Keyboard</span>
        <div class="creator-actions">
          <button id="creator-disable" class="secondary-btn" type="button" ${!supported || busy ? "disabled" : ""}>Return to normal RGB</button>
          <button id="creator-apply" class="primary-btn creator-apply-premium" type="button" ${!supported || busy ? "disabled" : ""}>Apply to keyboard</button>
        </div>
      </div>
    </article>
    <article class="panel creator-studio-board-panel">
      <div class="panel-title-row creator-board-title">
        <div>
          <p class="eyebrow">AL80 Digital Twin</p>
          <h2>Keyboard canvas</h2>
          <p class="muted">The same exact recovered key map, presented as a tactile editing surface.</p>
        </div>
        <div class="creator-board-header-actions">
          ${badge("79 RGB keys")}
          ${badge("3 accents")}
          <div class="creator-camera-controls">
            <div class="creator-view-switch" aria-label="Keyboard view">
              <button id="creator-view-top" class="secondary-btn ${creatorViewMode === "top" ? "view-active" : ""}" type="button">Top</button>
              <button id="creator-view-3d" class="secondary-btn ${creatorViewMode === "studio3d" ? "view-active" : ""}" type="button">3D</button>
            </div>
            <div class="creator-view-switch creator-scale-switch" aria-label="Keyboard scale">
              <button id="creator-scale-fit" class="secondary-btn ${creatorScaleMode === "fit" ? "view-active" : ""}" type="button">Fit</button>
              <button id="creator-scale-100" class="secondary-btn ${creatorScaleMode === "actual" ? "view-active" : ""}" type="button">100%</button>
            </div>
            <button id="creator-recenter" class="secondary-btn creator-recenter-btn" type="button">Re-center</button>
          </div>
        </div>
      </div>
      <div class="creator-board-stage ${creatorViewMode} ${creatorScaleMode}">
        <div class="creator-board-ambient"></div>
        <div class="creator-keyboard-shell">
          <div class="creator-keyboard-lip"></div>
          <div class="creator-board">${keys}${controls}</div>
        </div>
      </div>
      <div class="creator-board-foot">
        <span>Click or drag to paint</span>
        <span>Fit keeps all 82 LEDs visible</span>
        <span>100% shows native canvas scale</span>
        <span>3D is visual only — LED addressing never changes</span>
      </div>
    </article>
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
          <h1>Display Studio</h1>
          <p>
            Safe volatile previews for the validated display protocol.
          </p>
        </div>
        ${badge(
          supported ? "OSD supported" : "Unavailable",
          supported ? "good" : "warn",
        )}
      </div>

      <article class="panel lcd-studio-hero">
        <div class="lcd-studio-device">
          <div class="lcd-device-crown">
            <span>AL80</span>
            <span>96 × 160</span>
          </div>
          <div class="lcd-device-screen">
            <div class="lcd-screen-glow"></div>
            <div class="lcd-screen-content">
              <span class="lcd-screen-kicker">DISPLAY STUDIO</span>
              <strong>AL80</strong>
              <span>Validated feedback. Visible boundaries.</span>
            </div>
          </div>
          <div class="lcd-device-footer">
            <span>VOLATILE PREVIEW</span>
            <span>RGB565</span>
          </div>
        </div>

        <div class="lcd-studio-copy">
          <p class="eyebrow">Screen Composer</p>
          <h2>Everything shown here says exactly what works today</h2>
          <p class="muted">
            AL80 Studio separates physically validated LCD feedback from future
            framebuffer work. Nothing marked experimental is presented as a working
            device feature.
          </p>

          <div class="lcd-live-strip" aria-label="Display capability status">
            ${badge(capabilities?.lcdOsd ? "Display transport ready" : "Display unavailable", capabilities?.lcdOsd ? "good" : "warn")}
            ${badge(capabilities?.lcdFeedback ? "Typed feedback validated" : "Typed feedback unavailable", capabilities?.lcdFeedback ? "good" : "warn")}
            ${badge(capabilities?.inputEventAutoLcd ? "Auto LCD path ready" : "Auto LCD unavailable", capabilities?.inputEventAutoLcd ? "good" : "warn")}
            ${badge("Volatile only", "neutral")}
          </div>

          <section class="lcd-capability-section ready-now">
            <div class="lcd-capability-heading">
              <div>
                <span class="lcd-capability-kicker">AVAILABLE NOW</span>
                <h3>Validated on the real AL80</h3>
              </div>
              ${badge("Ready now", "good")}
            </div>

            <div class="lcd-template-grid">
              <div class="lcd-template-card validated">
                <span class="lcd-template-icon">◒</span>
                <div><strong>Volume</strong><small>0–100% OSD</small></div>
                <span class="template-state ready">Validated</span>
              </div>
              <div class="lcd-template-card validated">
                <span class="lcd-template-icon">◐</span>
                <div><strong>Mute</strong><small>Actual host audio state</small></div>
                <span class="template-state ready">Validated</span>
              </div>
              <div class="lcd-template-card validated">
                <span class="lcd-template-icon">◎</span>
                <div><strong>Action</strong><small>Typed generic feedback</small></div>
                <span class="template-state ready">Validated</span>
              </div>
              <div class="lcd-template-card validated">
                <span class="lcd-template-icon">◇</span>
                <div><strong>Profile / Scene</strong><small>Typed state feedback</small></div>
                <span class="template-state ready">Validated</span>
              </div>
            </div>
          </section>

          <section class="lcd-capability-section future-track">
            <div class="lcd-capability-heading">
              <div>
                <span class="lcd-capability-kicker">FUTURE / EXPERIMENTAL</span>
                <h3>Requires additional protocol work</h3>
              </div>
              ${badge("Not active", "neutral")}
            </div>

            <div class="lcd-template-grid lcd-future-grid">
              <div class="lcd-template-card future">
                <span class="lcd-template-icon">▧</span>
                <div>
                  <strong>Artwork</strong>
                  <small>Needs a dedicated arbitrary framebuffer API</small>
                </div>
                <span class="template-state future">Protocol work</span>
              </div>
              <div class="lcd-template-card future">
                <span class="lcd-template-icon">▶</span>
                <div>
                  <strong>Animation</strong>
                  <small>Needs validated frame scheduling / streaming</small>
                </div>
                <span class="template-state future">Protocol work</span>
              </div>
            </div>

      <article class="panel live-lcd-panel">
        <div class="panel-title-row">
          <div>
            <p class="eyebrow">Current host-driven state</p>
            <h2>Live LCD semantic mirror</h2>
            <p class="muted">
              Mirrors the last successfully host-driven HOME, Volume/Mute,
              or typed feedback state. This is semantic telemetry, not a
              fabricated pixel screenshot of arbitrary LCD firmware content.
            </p>
          </div>
          ${badge("Logical status")}
        </div>
        <div class="live-lcd-device">
          ${renderLiveLcdMirror()}
        </div>
      </article>

    </section>

          <div class="lcd-truth-rail">
            <div>
              <span class="lcd-truth-index">01</span>
              <div><strong>Works now</strong><small>Volume, mute and typed feedback use the validated volatile LCD path.</small></div>
            </div>
            <div>
              <span class="lcd-truth-index">02</span>
              <div><strong>No fake persistence</strong><small>This Studio does not claim persistent display writes.</small></div>
            </div>
            <div>
              <span class="lcd-truth-index">03</span>
              <div><strong>Future stays visible</strong><small>Artwork and animation remain roadmap capabilities until their protocol is proven.</small></div>
            </div>
          </div>

          <div class="lcd-studio-note">
            <span class="lcd-note-dot"></span>
            The controls below are the actual validated hardware previews.
          </div>
        </div>
      </article>
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
  const inputOptions = getSavedInputProfilesForHost();

  const creatorOptions = savedCreatorScenes
    .map(
      (scene) =>
        `<option value="${esc(scene.id)}">${esc(scene.name)}</option>`,
    )
    .join("");

  const inputProfileOptions = inputOptions
    .map(
      (profile) =>
        `<option value="${esc(profile.id)}">${esc(profile.name)}</option>`,
    )
    .join("");

  const cards =
    profiles.length === 0
      ? `
        <article class="panel empty-state">
          <div class="placeholder-plus">+</div>
          <h2>No host profiles yet</h2>
          <p class="muted">
            Compose RGB/Snake with an optional saved Creator scene and
            saved Input profile.
          </p>
        </article>
      `
      : profiles
          .map(
            (profile) => `
              <article class="profile-card">
                <div>
                  <p class="eyebrow">
                    Host profile ${profile.schemaVersion === 2 ? "V2" : "V1"}
                  </p>
                  <h2>${esc(profile.name)}</h2>
                  <div class="chip-row">
                    ${badge(`RGB ${profile.rgbEnabled ? "ON" : "OFF"}`)}
                    ${badge(`Snake ${profile.overlayEnabled ? "ON" : "OFF"}`)}
                    ${
                      profile.schemaVersion === 2
                        ? badge(
                            profile.creatorScene
                              ? `Creator ${profile.creatorScene.name}`
                              : "Creator OFF",
                          )
                        : badge("Creator preserve")
                    }
                    ${
                      profile.schemaVersion === 2
                        ? badge(
                            profile.inputProfile
                              ? `Input ${profile.inputProfile.name}`
                              : "Input OFF",
                          )
                        : badge("Input preserve")
                    }
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
            Host Profiles V2 compose safe volatile features into one preset:
            RGB/Snake, a saved Creator scene and a saved Input profile.
          </p>
        </div>
      </div>

      <article class="panel">
        <div class="panel-title-row">
          <div>
            <p class="eyebrow">New Host Profile V2</p>
            <h2>Compose saved components</h2>
          </div>
          ${badge("Host-local / volatile hardware", "good")}
        </div>

        <div class="control-grid">
          <label>
            <span>Creator scene</span>
            <select id="profile-creator-source">
              <option value="">OFF</option>
              ${creatorOptions}
            </select>
          </label>

          <label>
            <span>Input profile</span>
            <select id="profile-input-source">
              <option value="">Router OFF</option>
              ${inputProfileOptions}
            </select>
          </label>
        </div>

        <div class="button-row">
          <button
            id="profile-save"
            class="primary-btn"
            type="button"
            ${!status?.connected || busy ? "disabled" : ""}
          >
            Save current RGB/Snake + selections
          </button>
        </div>

        <p class="muted">
          Creator and Input selections are copied into the Host Profile, so
          applying it does not depend on the source item remaining in its
          separate library. Legacy V1 profiles keep their original behavior.
        </p>
      </article>

      <div class="profile-grid">
        ${cards}
      </div>

      <article class="panel">
        <p class="muted">
          al80d still reports <code>profiles=NO</code> because firmware-side
          profiles do not exist. Host Profiles V2 orchestrate only existing,
          validated volatile APIs through al80d.
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
  const previousWorkspace =
    app.querySelector<HTMLElement>(".workspace");
  const previousCreatorStage =
    app.querySelector<HTMLElement>(".creator-board-stage");

  const preserveSameView = lastRenderedView === view;
  const workspaceScrollTop =
    preserveSameView ? previousWorkspace?.scrollTop ?? 0 : 0;
  const creatorStageScroll =
    preserveSameView && view === "creator" && previousCreatorStage
      ? {
          left: previousCreatorStage.scrollLeft,
          top: previousCreatorStage.scrollTop,
        }
      : null;

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
  lastRenderedView = view;
  startLiveTelemetryLoop();

  requestAnimationFrame(() => {
    const workspace = app.querySelector<HTMLElement>(".workspace");
    if (workspace && preserveSameView) {
      workspace.scrollTop = workspaceScrollTop;
    }

    if (creatorStageScroll) {
      requestAnimationFrame(() => {
        const stage =
          app.querySelector<HTMLElement>(".creator-board-stage");
        if (stage) {
          stage.scrollLeft = creatorStageScroll.left;
          stage.scrollTop = creatorStageScroll.top;
        }
      });
    }

    applyCreatorOrbitTransform();
    updateLiveTelemetryDom();
  });
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
    await Promise.all([
      hydrateMainHostLibraries(),
      hydrateInputProfilesFromHost(),
    ]);

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
  void refreshLiveTelemetry();
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
    .querySelector<HTMLButtonElement>("#creator-view-top")
    ?.addEventListener("click", () => {
      creatorViewMode = "top";
      render();
      scheduleCreatorViewportSync();
    });

  document
    .querySelector<HTMLButtonElement>("#creator-view-3d")
    ?.addEventListener("click", () => {
      creatorViewMode = "studio3d";
      render();
      scheduleCreatorViewportSync();
    });

  document
    .querySelector<HTMLButtonElement>("#creator-scale-fit")
    ?.addEventListener("click", () => {
      creatorScaleMode = "fit";
      render();
      scheduleCreatorViewportSync();
    });

  document
    .querySelector<HTMLButtonElement>("#creator-scale-100")
    ?.addEventListener("click", () => {
      creatorScaleMode = "actual";
      render();
      scheduleCreatorViewportSync();
    });

  document
    .querySelector<HTMLButtonElement>("#creator-recenter")
    ?.addEventListener("click", () => {
      creatorOrbitX = 9;
      creatorOrbitY = -1.8;
      creatorOrbitZoom = 1;
      applyCreatorOrbitTransform();
      syncCreatorViewport(true);
    });

  const creatorStage =
    document.querySelector<HTMLElement>(".creator-board-stage");

  creatorStage?.addEventListener("pointerdown", (event) => {
    if (creatorViewMode !== "studio3d" || event.button !== 0) return;

    const target = event.target as Element | null;
    if (
      target?.closest(
        "[data-creator-led],button,input,select,textarea,label,a",
      )
    ) {
      return;
    }

    creatorOrbitDragging = true;
    creatorOrbitPointerId = event.pointerId;
    creatorOrbitLastX = event.clientX;
    creatorOrbitLastY = event.clientY;
    creatorStage.setPointerCapture(event.pointerId);
    creatorStage.classList.add("orbit-dragging");
    event.preventDefault();
  });

  creatorStage?.addEventListener("pointermove", (event) => {
    if (
      !creatorOrbitDragging ||
      creatorOrbitPointerId !== event.pointerId
    ) {
      return;
    }

    const dx = event.clientX - creatorOrbitLastX;
    const dy = event.clientY - creatorOrbitLastY;

    creatorOrbitLastX = event.clientX;
    creatorOrbitLastY = event.clientY;

    creatorOrbitY = creatorOrbitClamp(
      creatorOrbitY + dx * 0.22,
      -55,
      55,
    );
    creatorOrbitX = creatorOrbitClamp(
      creatorOrbitX - dy * 0.18,
      -8,
      55,
    );

    applyCreatorOrbitTransform();
  });

  const finishCreatorOrbit = (event: PointerEvent) => {
    if (creatorOrbitPointerId !== event.pointerId) return;

    creatorOrbitDragging = false;
    creatorOrbitPointerId = null;
    creatorStage?.classList.remove("orbit-dragging");

    if (creatorStage?.hasPointerCapture(event.pointerId)) {
      creatorStage.releasePointerCapture(event.pointerId);
    }
  };

  creatorStage?.addEventListener("pointerup", finishCreatorOrbit);
  creatorStage?.addEventListener("pointercancel", finishCreatorOrbit);

  creatorStage?.addEventListener(
    "wheel",
    (event) => {
      if (creatorViewMode !== "studio3d") return;

      creatorOrbitZoom = creatorOrbitClamp(
        creatorOrbitZoom + (event.deltaY < 0 ? 0.06 : -0.06),
        0.72,
        1.30,
      );

      applyCreatorOrbitTransform();
      event.preventDefault();
    },
    { passive: false },
  );

  /*
   * Safe on every page: the helper returns immediately when the
   * Creator viewport is not mounted.
   */
  scheduleCreatorViewportSync(false);

  document
    .querySelector<HTMLSelectElement>("#creator-effect-kind")
    ?.addEventListener("change", (event) => {
      creatorEffectId =
        (event.currentTarget as HTMLSelectElement).value as CreatorEffectId;
    });

  document
    .querySelector<HTMLInputElement>("#creator-effect-primary")
    ?.addEventListener("input", (event) => {
      creatorEffectPrimary =
        (event.currentTarget as HTMLInputElement).value;
    });

  document
    .querySelector<HTMLInputElement>("#creator-effect-secondary")
    ?.addEventListener("input", (event) => {
      creatorEffectSecondary =
        (event.currentTarget as HTMLInputElement).value;
    });

  document
    .querySelector<HTMLInputElement>("#creator-effect-speed")
    ?.addEventListener("input", (event) => {
      creatorEffectSpeed = Number.parseInt(
        (event.currentTarget as HTMLInputElement).value,
        10,
      );
    });

  document
    .querySelector<HTMLInputElement>("#creator-effect-tail")
    ?.addEventListener("input", (event) => {
      creatorEffectTail = Number.parseInt(
        (event.currentTarget as HTMLInputElement).value,
        10,
      );
    });

  document
    .querySelector<HTMLInputElement>("#creator-effect-phase")
    ?.addEventListener("input", (event) => {
      creatorEffectPhase =
        Number.parseInt(
          (event.currentTarget as HTMLInputElement).value,
          10,
        ) / 100;
    });

  document
    .querySelector<HTMLButtonElement>("#creator-effect-preview")
    ?.addEventListener("click", () => {
      if (!creatorLayout) return;

      creatorSnapshot();

      const order = [
        ...creatorLayout.keys.map((key) => key.ledIndex),
        ...creatorLayout.accents.map((accent) => accent.ledIndex),
      ];

      creatorColors = renderCreatorEffectFrame(
        {
          effect: creatorEffectId,
          primary: creatorEffectPrimary,
          secondary: creatorEffectSecondary,
          speed: creatorEffectSpeed,
          tailLength: creatorEffectTail,
          phase: creatorEffectPhase,
        },
        order,
        82,
      );

      notice = `Rendered ${creatorEffectId} preview locally.`;
      render();
    });

  document
    .querySelector<HTMLSelectElement>("#creator-input-source")
    ?.addEventListener("change", (event) => {
      creatorInputSource =
        (event.currentTarget as HTMLSelectElement).value;
    });

  document
    .querySelector<HTMLButtonElement>("#creator-apply-unified")
    ?.addEventListener("click", () => {
      void action(async () => {
        let inputBindings: HostProfileInputBinding[] | null = null;

        if (creatorInputSource === "draft") {
          inputBindings = getCurrentInputDraftForHost();
        } else if (creatorInputSource.startsWith("saved:")) {
          const profileId = creatorInputSource.slice("saved:".length);
          const profile = getSavedInputProfilesForHost().find(
            (item) => item.id === profileId,
          );

          if (!profile) {
            throw new Error("Selected Input profile no longer exists");
          }

          inputBindings = profile.bindings;
        } else if (creatorInputSource !== "off") {
          throw new Error("Unknown Creator Input source");
        }

        await invoke<boolean>("set_rgb_core_runtime", {
          enabled: true,
        });

        await invoke<string>("apply_creator_scene", {
          colors: creatorColors,
        });

        if (inputBindings === null) {
          await invoke<string>("disable_input_router");
        } else {
          if (
            inputBindings.length === 0 ||
            inputBindings.length > 12
          ) {
            throw new Error(
              "Creator Input behavior must contain 1..12 bindings",
            );
          }

          await invoke<string>("apply_input_profile", {
            bindings: inputBindings,
          });
        }
      }, "Unified Creator workspace applied.");
    });

  document
    .querySelector<HTMLButtonElement>("#creator-exit-unified")
    ?.addEventListener("click", () => {
      void action(async () => {
        await invoke<string>("disable_creator_scene");
        await invoke<string>("disable_input_router");
      }, "Unified Creator workspace exited.");
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
      creatorMirrorExact = true;
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
      const suggested = `Profile ${profiles.length + 1}`;
      const name = window.prompt("Profile name", suggested)?.trim();

      if (!name) return;

      const creatorId =
        document
          .querySelector<HTMLSelectElement>("#profile-creator-source")
          ?.value ?? "";

      const inputId =
        document
          .querySelector<HTMLSelectElement>("#profile-input-source")
          ?.value ?? "";

      const creatorSource =
        creatorId.length > 0
          ? savedCreatorScenes.find((scene) => scene.id === creatorId)
          : undefined;

      const inputSource =
        inputId.length > 0
          ? getSavedInputProfilesForHost().find(
              (profile) => profile.id === inputId,
            )
          : undefined;

      if (creatorId.length > 0 && !creatorSource) {
        notice = "Selected Creator scene no longer exists.";
        render();
        return;
      }

      if (inputId.length > 0 && !inputSource) {
        notice = "Selected Input profile no longer exists.";
        render();
        return;
      }

      const profile: HostProfile = {
        schemaVersion: 2,
        id: crypto.randomUUID(),
        name,
        rgbEnabled: status?.rgbCoreEnabled === true,
        overlayEnabled: status?.overlayEnabled === true,
        creatorScene: creatorSource
          ? {
              name: creatorSource.name,
              colors: [...creatorSource.colors],
            }
          : null,
        inputProfile: inputSource
          ? {
              name: inputSource.name,
              bindings: inputSource.bindings.map((binding) => ({
                ...binding,
              })),
            }
          : null,
        createdAt: new Date().toLocaleString(),
      };

      profiles = [...profiles, profile];
      saveProfiles();
      notice = `Saved Host Profile V2 ${name}.`;
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

          if (profile.schemaVersion === 2) {
            if (profile.creatorScene) {
              creatorColors = [...profile.creatorScene.colors];

              await invoke<string>("apply_creator_scene", {
                colors: profile.creatorScene.colors,
              });
            } else {
              await invoke<string>("disable_creator_scene");
            }

            if (profile.inputProfile) {
              await invoke<string>("apply_input_profile", {
                bindings: profile.inputProfile.bindings,
              });

              replaceInputDraftFromHost(
                profile.inputProfile.bindings,
              );
            } else {
              await invoke<string>("disable_input_router");
            }
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
