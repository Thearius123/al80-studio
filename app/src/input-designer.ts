import { invoke } from "@tauri-apps/api/core";

export interface InputDesignerCapabilities {
  inputRouter?: boolean;
  inputBindings?: number;
  inputActions?: number;
  inputRouterState?: boolean | null;
}

interface LayoutItem {
  matrix: [number, number];
  code: string;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
}

interface CreatorLayout {
  schemaVersion: 1;
  layoutWidth: number;
  layoutHeight: number;
  keys: LayoutItem[];
  controls: LayoutItem[];
}

interface ActionItem {
  id: number;
  key: string;
  label: string;
  category: string;
  description: string;
}

interface ActionRegistry {
  schemaVersion: 1;
  protocol: "0x4B";
  maxBindings: 12;
  actions: ActionItem[];
}

interface InputBindingDraft {
  id: string;
  event: number;
  trigger: number;
  triggerA: number;
  triggerB: number;
  action: number;
}

interface SavedInputProfile {
  id: string;
  name: string;
  bindings: InputBindingDraft[];
  createdAt: string;
}

export interface InputDesignerEventContext {
  capabilities: InputDesignerCapabilities | null;
  busy: boolean;
  runAction: (
    operation: () => Promise<void>,
    successMessage?: string,
  ) => Promise<void>;
  rerender: () => void;
  setNotice: (message: string) => void;
}

const DRAFT_KEY = "al80-studio.input-draft.v1";
const PROFILE_KEY = "al80-studio.input-profiles.v1";

const EVENTS = [
  { id: 1, label: "Knob Left / CCW" },
  { id: 2, label: "Knob Right / CW" },
  { id: 3, label: "Knob Press" },
];

const TRIGGERS = [
  { id: 0, label: "Always" },
  { id: 1, label: "Layer / Fn" },
  { id: 2, label: "Hold a key" },
  { id: 3, label: "Hold modifiers" },
];

const MODIFIERS = [
  { bit: 1, label: "L Ctrl" },
  { bit: 2, label: "L Shift" },
  { bit: 4, label: "L Alt" },
  { bit: 8, label: "L GUI" },
  { bit: 16, label: "R Ctrl" },
  { bit: 32, label: "R Shift" },
  { bit: 64, label: "R Alt" },
  { bit: 128, label: "R GUI" },
];

let layout: CreatorLayout | null = null;
let registry: ActionRegistry | null = null;
let bindings: InputBindingDraft[] = loadDraft();
let profiles: SavedInputProfile[] = loadProfiles();
let keyPickerBindingId: string | null = null;

function makeId(): string {
  if (typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }

  return `input-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function safe(value: unknown): string {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function defaultBindings(): InputBindingDraft[] {
  return [
    { id: makeId(), event: 1, trigger: 0, triggerA: 0, triggerB: 0, action: 1 },
    { id: makeId(), event: 2, trigger: 0, triggerA: 0, triggerB: 0, action: 2 },
    { id: makeId(), event: 3, trigger: 0, triggerA: 0, triggerB: 0, action: 3 },
  ];
}

function cloneBindings(items: InputBindingDraft[]): InputBindingDraft[] {
  return items.map((item) => ({ ...item, id: makeId() }));
}

function validDraft(value: unknown): value is InputBindingDraft[] {
  if (!Array.isArray(value) || value.length === 0 || value.length > 12) {
    return false;
  }

  return value.every((raw) => {
    if (typeof raw !== "object" || raw === null) return false;
    const item = raw as Partial<InputBindingDraft>;
    return (
      typeof item.id === "string" &&
      Number.isInteger(item.event) &&
      Number.isInteger(item.trigger) &&
      Number.isInteger(item.triggerA) &&
      Number.isInteger(item.triggerB) &&
      Number.isInteger(item.action)
    );
  });
}

function loadDraft(): InputBindingDraft[] {
  try {
    const raw = localStorage.getItem(DRAFT_KEY);
    if (!raw) return defaultBindings();
    const value = JSON.parse(raw) as unknown;
    return validDraft(value) ? value : defaultBindings();
  } catch {
    return defaultBindings();
  }
}

function saveDraft(): void {
  localStorage.setItem(DRAFT_KEY, JSON.stringify(bindings));
}

function loadProfiles(): SavedInputProfile[] {
  try {
    const raw = localStorage.getItem(PROFILE_KEY);
    if (!raw) return [];
    const value = JSON.parse(raw) as unknown;
    if (!Array.isArray(value)) return [];

    return value.filter((raw): raw is SavedInputProfile => {
      if (typeof raw !== "object" || raw === null) return false;
      const item = raw as Partial<SavedInputProfile>;
      return (
        typeof item.id === "string" &&
        typeof item.name === "string" &&
        typeof item.createdAt === "string" &&
        validDraft(item.bindings)
      );
    });
  } catch {
    return [];
  }
}

function saveProfiles(): void {
  localStorage.setItem(PROFILE_KEY, JSON.stringify(profiles));
}

function persistAndRender(context: InputDesignerEventContext): void {
  saveDraft();
  context.rerender();
}

function actionById(id: number): ActionItem | undefined {
  return registry?.actions.find((item) => item.id === id);
}

function matrixLabel(row: number, col: number): string {
  const all = [...(layout?.keys ?? []), ...(layout?.controls ?? [])];
  const item = all.find(
    (entry) => entry.matrix[0] === row && entry.matrix[1] === col,
  );

  return item ? item.label : `Matrix ${row},${col}`;
}

function eventLabel(id: number): string {
  return EVENTS.find((item) => item.id === id)?.label ?? `Event ${id}`;
}

function triggerLabel(binding: InputBindingDraft): string {
  switch (binding.trigger) {
    case 0:
      return "Always";
    case 1:
      return binding.triggerA === 1
        ? "Fn / Layer 1"
        : `Layer ${binding.triggerA}`;
    case 2:
      return `Hold ${matrixLabel(binding.triggerA, binding.triggerB)}`;
    case 3: {
      const names = MODIFIERS
        .filter((item) => (binding.triggerA & item.bit) !== 0)
        .map((item) => item.label);
      return names.length ? names.join(" + ") : "Modifiers";
    }
    default:
      return "Unknown trigger";
  }
}

function optionList(
  values: Array<{ id: number; label: string }>,
  selected: number,
): string {
  return values
    .map(
      (item) =>
        `<option value="${item.id}" ${item.id === selected ? "selected" : ""}>${safe(item.label)}</option>`,
    )
    .join("");
}

function actionOptions(selected: number): string {
  if (!registry) return "";

  const categories = new Map<string, ActionItem[]>();

  for (const action of registry.actions) {
    const bucket = categories.get(action.category) ?? [];
    bucket.push(action);
    categories.set(action.category, bucket);
  }

  return [...categories.entries()]
    .map(
      ([category, actions]) => `
        <optgroup label="${safe(category)}">
          ${actions
            .map(
              (action) =>
                `<option value="${action.id}" ${action.id === selected ? "selected" : ""}>${safe(action.label)}</option>`,
            )
            .join("")}
        </optgroup>`,
    )
    .join("");
}

function triggerEditor(binding: InputBindingDraft): string {
  if (binding.trigger === 0) {
    return `<div class="input-trigger-summary">No extra key required.</div>`;
  }

  if (binding.trigger === 1) {
    return `
      <label class="input-field compact">
        <span>Layer</span>
        <input
          type="number"
          min="0"
          max="31"
          value="${binding.triggerA}"
          data-input-layer="${safe(binding.id)}"
        />
      </label>
      <div class="input-trigger-summary">
        Layer 1 is the recovered Fn layer on this AL80 keymap.
      </div>
    `;
  }

  if (binding.trigger === 2) {
    return `
      <div class="input-key-trigger">
        <button
          class="secondary-btn"
          type="button"
          data-input-pick-key="${safe(binding.id)}"
        >Pick key</button>
        <strong>${safe(matrixLabel(binding.triggerA, binding.triggerB))}</strong>
        <small>[${binding.triggerA}, ${binding.triggerB}]</small>
      </div>
    `;
  }

  return `
    <div class="input-mod-grid">
      ${MODIFIERS.map(
        (modifier) => `
          <label>
            <input
              type="checkbox"
              data-input-modifier="${safe(binding.id)}"
              data-modifier-bit="${modifier.bit}"
              ${(binding.triggerA & modifier.bit) !== 0 ? "checked" : ""}
            />
            <span>${safe(modifier.label)}</span>
          </label>`,
      ).join("")}
    </div>
  `;
}

function bindingCard(binding: InputBindingDraft, index: number): string {
  const action = actionById(binding.action);

  return `
    <article class="input-rule-card">
      <div class="input-rule-priority">
        <span>Priority</span>
        <strong>${index + 1}</strong>
      </div>

      <div class="input-rule-editor">
        <div class="input-rule-row">
          <label class="input-field">
            <span>Knob event</span>
            <select data-input-event="${safe(binding.id)}">
              ${optionList(EVENTS, binding.event)}
            </select>
          </label>

          <label class="input-field">
            <span>Trigger</span>
            <select data-input-trigger="${safe(binding.id)}">
              ${optionList(TRIGGERS, binding.trigger)}
            </select>
          </label>

          <label class="input-field action-field">
            <span>Action</span>
            <select data-input-action="${safe(binding.id)}">
              ${actionOptions(binding.action)}
            </select>
          </label>
        </div>

        <div class="input-trigger-editor">
          ${triggerEditor(binding)}
        </div>

        <div class="input-rule-description">
          <strong>${safe(eventLabel(binding.event))}</strong>
          <span>when ${safe(triggerLabel(binding))}</span>
          <span>→</span>
          <strong>${safe(action?.label ?? `Action ${binding.action}`)}</strong>
          ${action?.description ? `<small>${safe(action.description)}</small>` : ""}
        </div>
      </div>

      <div class="input-rule-actions">
        <button class="icon-btn" type="button" data-input-up="${safe(binding.id)}" ${index === 0 ? "disabled" : ""} title="Move up">↑</button>
        <button class="icon-btn" type="button" data-input-down="${safe(binding.id)}" ${index === bindings.length - 1 ? "disabled" : ""} title="Move down">↓</button>
        <button class="icon-btn" type="button" data-input-duplicate="${safe(binding.id)}" ${bindings.length >= 12 ? "disabled" : ""} title="Duplicate">⧉</button>
        <button class="icon-btn danger" type="button" data-input-delete="${safe(binding.id)}" ${bindings.length <= 1 ? "disabled" : ""} title="Delete">×</button>
      </div>
    </article>
  `;
}

function keyPicker(): string {
  if (!keyPickerBindingId || !layout) return "";

  const target = bindings.find((item) => item.id === keyPickerBindingId);
  if (!target) return "";

  const width = layout.layoutWidth || 1;
  const height = layout.layoutHeight || 1;
  const items = [...layout.keys, ...layout.controls];

  const keys = items
    .map((item) => {
      const left = (item.x / width) * 100;
      const top = (item.y / height) * 100;
      const w = (item.w / width) * 100;
      const h = (item.h / height) * 100;
      const selected =
        target.triggerA === item.matrix[0] &&
        target.triggerB === item.matrix[1];

      return `
        <button
          type="button"
          class="input-picker-key ${selected ? "selected" : ""}"
          style="left:${left}%;top:${top}%;width:${w}%;height:${h}%"
          data-input-matrix-row="${item.matrix[0]}"
          data-input-matrix-col="${item.matrix[1]}"
          title="${safe(item.code)} [${item.matrix.join(",")}]"
        >${safe(item.label)}</button>
      `;
    })
    .join("");

  return `
    <article class="panel input-key-picker-panel">
      <div class="panel-title-row">
        <div>
          <p class="eyebrow">Physical trigger picker</p>
          <h2>Choose the held key</h2>
          <p class="muted">Studio stores the recovered matrix coordinate internally. Normal users never need to type row/column values.</p>
        </div>
        <button class="secondary-btn" type="button" id="input-picker-cancel">Cancel</button>
      </div>
      <div class="input-keyboard-picker">${keys}</div>
    </article>
  `;
}

function profilesHtml(): string {
  if (profiles.length === 0) {
    return `<div class="input-empty">No local input profiles yet.</div>`;
  }

  return profiles
    .map(
      (profile) => `
        <article class="input-profile-card">
          <div>
            <strong>${safe(profile.name)}</strong>
            <small>${profile.bindings.length} rule(s) · ${safe(profile.createdAt)}</small>
          </div>
          <div class="input-profile-actions">
            <button class="secondary-btn" type="button" data-input-profile-load="${safe(profile.id)}">Load</button>
            <button class="secondary-btn" type="button" data-input-profile-duplicate="${safe(profile.id)}">Duplicate</button>
            <button class="secondary-btn danger" type="button" data-input-profile-delete="${safe(profile.id)}">Delete</button>
          </div>
        </article>
      `,
    )
    .join("");
}

export async function refreshInputDesigner(): Promise<void> {
  const [layoutResponse, actionResponse] = await Promise.all([
    fetch("./devices/al80/layout.json", { cache: "no-store" }),
    fetch("./devices/al80/input-actions.json", { cache: "no-store" }),
  ]);

  if (!layoutResponse.ok) {
    throw new Error(`Input Designer layout failed: HTTP ${layoutResponse.status}`);
  }

  if (!actionResponse.ok) {
    throw new Error(`Input Designer action registry failed: HTTP ${actionResponse.status}`);
  }

  const nextLayout = (await layoutResponse.json()) as CreatorLayout;
  const nextRegistry = (await actionResponse.json()) as ActionRegistry;

  if (
    nextLayout.schemaVersion !== 1 ||
    !Array.isArray(nextLayout.keys) ||
    !Array.isArray(nextLayout.controls)
  ) {
    throw new Error("Invalid AL80 layout for Input Designer");
  }

  if (
    nextRegistry.schemaVersion !== 1 ||
    nextRegistry.protocol !== "0x4B" ||
    nextRegistry.maxBindings !== 12 ||
    !Array.isArray(nextRegistry.actions) ||
    nextRegistry.actions.length !== 25 ||
    nextRegistry.actions.some((item, index) => item.id !== index)
  ) {
    throw new Error("Invalid Input Router action registry");
  }

  layout = nextLayout;
  registry = nextRegistry;
  bindings = loadDraft();
  profiles = loadProfiles();
}

export function renderInputDesigner(
  capabilities: InputDesignerCapabilities | null,
  busy: boolean,
): string {
  const supported = capabilities?.inputRouter === true;
  const routerOn = capabilities?.inputRouterState === true;
  const slotLimit = capabilities?.inputBindings ?? 12;
  const actionLimit = capabilities?.inputActions ?? 24;

  return `
    <section class="page input-designer-page">
      <div class="page-heading">
        <div>
          <p class="eyebrow">Creator / Inputs</p>
          <h1>Input Designer</h1>
          <p>Build typed knob rules visually. Specific triggers override Always rules; within the same trigger class, the first matching rule wins.</p>
        </div>
        <span class="badge ${supported ? "good" : "neutral"}">${supported ? `Router ${routerOn ? "ON" : "OFF"}` : "Not installed"}</span>
      </div>

      <article class="panel input-toolbar-panel">
        <div class="input-toolbar-main">
          <div>
            <p class="eyebrow">Volatile hardware profile</p>
            <h2>${bindings.length} / ${slotLimit} rules</h2>
            <p class="muted">Firmware allowlist: action IDs 0–${actionLimit}. Keyboard reboot starts with Router OFF and safe Volume/Mute fallback.</p>
          </div>
          <div class="input-toolbar-actions">
            <button id="input-read" class="secondary-btn" type="button" ${!supported || busy ? "disabled" : ""}>Read keyboard</button>
            <button id="input-disable" class="secondary-btn" type="button" ${!supported || busy ? "disabled" : ""}>Disable Router</button>
            <button id="input-defaults" class="secondary-btn" type="button" ${!supported || busy ? "disabled" : ""}>Safe defaults</button>
            <button id="input-add" class="secondary-btn" type="button" ${bindings.length >= 12 || busy ? "disabled" : ""}>+ Rule</button>
            <button id="input-apply" class="primary-btn" type="button" ${!supported || busy || bindings.length === 0 ? "disabled" : ""}>Apply profile</button>
          </div>
        </div>
      </article>

      <article class="panel">
        <div class="panel-title-row">
          <div>
            <p class="eyebrow">One-click starting points</p>
            <h2>Presets</h2>
          </div>
          <span class="badge neutral">Local draft only until Apply</span>
        </div>
        <div class="input-presets">
          <button type="button" class="preset-chip" data-input-preset="volume">Default Volume</button>
          <button type="button" class="preset-chip" data-input-preset="fn-snake">Fn + Snake</button>
          <button type="button" class="preset-chip" data-input-preset="ctrl-snake">Ctrl + Snake</button>
          <button type="button" class="preset-chip" data-input-preset="button-wheel">Button + Wheel</button>
          <button type="button" class="preset-chip" data-input-preset="rgb">RGB Brightness</button>
          <button type="button" class="preset-chip" data-input-preset="media">Media Scrub</button>
          <button type="button" class="preset-chip" data-input-preset="pages">Page Navigation</button>
        </div>
      </article>

      <div class="input-rules">
        ${bindings.map(bindingCard).join("")}
      </div>

      ${keyPicker()}

      <article class="panel">
        <div class="panel-title-row">
          <div>
            <p class="eyebrow">Host library</p>
            <h2>Saved input profiles</h2>
            <p class="muted">Profiles live on this computer. Applying copies only the validated 0x4B binding table into volatile keyboard RAM.</p>
          </div>
          <button id="input-profile-save" class="primary-btn" type="button">Save current</button>
        </div>
        <div class="input-profile-grid">${profilesHtml()}</div>
      </article>

      <article class="panel input-safety-panel">
        <div>
          <p class="eyebrow">Safety contract</p>
          <h2>Typed actions only</h2>
        </div>
        <div class="input-safety-grid">
          <span><strong>12</strong> binding slots</span>
          <span><strong>0–24</strong> allowlisted actions</span>
          <span><strong>RAM</strong> keyboard configuration</span>
          <span><strong>NO</strong> EEPROM / arbitrary keycode / shell</span>
        </div>
        <p class="muted">Apply is transactional in al80d/core: disable → clear → set every binding → enable. On failure the router remains disabled and safe defaults are restored.</p>
      </article>
    </section>
  `;
}

function findBinding(id: string): InputBindingDraft | undefined {
  return bindings.find((item) => item.id === id);
}

function replacePreset(name: string): void {
  const volume = defaultBindings();

  switch (name) {
    case "volume":
      bindings = volume;
      break;

    case "fn-snake":
      bindings = [
        ...volume,
        { id: makeId(), event: 1, trigger: 1, triggerA: 1, triggerB: 0, action: 21 },
        { id: makeId(), event: 2, trigger: 1, triggerA: 1, triggerB: 0, action: 22 },
        { id: makeId(), event: 3, trigger: 1, triggerA: 1, triggerB: 0, action: 23 },
      ];
      break;

    case "ctrl-snake":
      bindings = [
        ...volume,
        { id: makeId(), event: 1, trigger: 3, triggerA: 1, triggerB: 0, action: 21 },
        { id: makeId(), event: 2, trigger: 3, triggerA: 1, triggerB: 0, action: 22 },
        { id: makeId(), event: 3, trigger: 3, triggerA: 1, triggerB: 0, action: 23 },
      ];
      break;

    case "button-wheel":
      bindings = [
        ...volume,
        { id: makeId(), event: 3, trigger: 2, triggerA: 0, triggerB: 14, action: 0 },
        { id: makeId(), event: 1, trigger: 2, triggerA: 0, triggerB: 14, action: 21 },
        { id: makeId(), event: 2, trigger: 2, triggerA: 0, triggerB: 14, action: 22 },
      ];
      break;

    case "rgb":
      bindings = [
        { id: makeId(), event: 1, trigger: 0, triggerA: 0, triggerB: 0, action: 15 },
        { id: makeId(), event: 2, trigger: 0, triggerA: 0, triggerB: 0, action: 16 },
        { id: makeId(), event: 3, trigger: 0, triggerA: 0, triggerB: 0, action: 3 },
      ];
      break;

    case "media":
      bindings = [
        { id: makeId(), event: 1, trigger: 0, triggerA: 0, triggerB: 0, action: 4 },
        { id: makeId(), event: 2, trigger: 0, triggerA: 0, triggerB: 0, action: 5 },
        { id: makeId(), event: 3, trigger: 0, triggerA: 0, triggerB: 0, action: 6 },
      ];
      break;

    case "pages":
      bindings = [
        { id: makeId(), event: 1, trigger: 0, triggerA: 0, triggerB: 0, action: 14 },
        { id: makeId(), event: 2, trigger: 0, triggerA: 0, triggerB: 0, action: 13 },
        { id: makeId(), event: 3, trigger: 0, triggerA: 0, triggerB: 0, action: 3 },
      ];
      break;
  }

  keyPickerBindingId = null;
  saveDraft();
}

function encodedBindings(): Array<{
  event: number;
  trigger: number;
  triggerA: number;
  triggerB: number;
  action: number;
}> {
  return bindings.map(({ event, trigger, triggerA, triggerB, action }) => ({
    event,
    trigger,
    triggerA,
    triggerB,
    action,
  }));
}

function parseDump(response: string): InputBindingDraft[] {
  const marker = " bindings=";
  const index = response.indexOf(marker);

  if (index < 0) {
    throw new Error(`Unexpected INPUT DUMP response: ${response}`);
  }

  const raw = response.slice(index + marker.length).trim();
  if (!raw) return defaultBindings();

  const parsed = raw
    .split(";")
    .filter(Boolean)
    .map((segment) => {
      const values = segment.split(",").map(Number);
      if (values.length !== 6 || values.some((value) => !Number.isInteger(value))) {
        throw new Error(`Invalid binding returned by keyboard: ${segment}`);
      }

      const [slot, event, trigger, triggerA, triggerB, action] = values;
      return { slot, id: makeId(), event, trigger, triggerA, triggerB, action };
    })
    .sort((left, right) => left.slot - right.slot)
    .map(({ slot: _slot, ...binding }) => binding);

  return parsed.length ? parsed : defaultBindings();
}

export function bindInputDesignerEvents(
  context: InputDesignerEventContext,
): void {
  document.querySelector<HTMLButtonElement>("#input-add")?.addEventListener("click", () => {
    if (bindings.length >= 12) return;
    bindings = [
      ...bindings,
      { id: makeId(), event: 1, trigger: 0, triggerA: 0, triggerB: 0, action: 0 },
    ];
    persistAndRender(context);
  });

  document.querySelectorAll<HTMLSelectElement>("[data-input-event]").forEach((element) => {
    element.addEventListener("change", () => {
      const binding = findBinding(element.dataset.inputEvent ?? "");
      if (!binding) return;
      binding.event = Number(element.value);
      persistAndRender(context);
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-input-trigger]").forEach((element) => {
    element.addEventListener("change", () => {
      const binding = findBinding(element.dataset.inputTrigger ?? "");
      if (!binding) return;
      binding.trigger = Number(element.value);
      binding.triggerA = binding.trigger === 1 ? 1 : 0;
      binding.triggerB = 0;
      keyPickerBindingId = binding.trigger === 2 ? binding.id : null;
      persistAndRender(context);
    });
  });

  document.querySelectorAll<HTMLSelectElement>("[data-input-action]").forEach((element) => {
    element.addEventListener("change", () => {
      const binding = findBinding(element.dataset.inputAction ?? "");
      if (!binding) return;
      binding.action = Number(element.value);
      persistAndRender(context);
    });
  });

  document.querySelectorAll<HTMLInputElement>("[data-input-layer]").forEach((element) => {
    element.addEventListener("change", () => {
      const binding = findBinding(element.dataset.inputLayer ?? "");
      if (!binding) return;
      const value = Math.max(0, Math.min(31, Number(element.value) || 0));
      binding.triggerA = value;
      binding.triggerB = 0;
      persistAndRender(context);
    });
  });

  document.querySelectorAll<HTMLInputElement>("[data-input-modifier]").forEach((element) => {
    element.addEventListener("change", () => {
      const binding = findBinding(element.dataset.inputModifier ?? "");
      if (!binding) return;
      const bit = Number(element.dataset.modifierBit ?? "0");
      if (element.checked) binding.triggerA |= bit;
      else binding.triggerA &= ~bit;
      binding.triggerB = 0;
      persistAndRender(context);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-input-pick-key]").forEach((button) => {
    button.addEventListener("click", () => {
      keyPickerBindingId = button.dataset.inputPickKey ?? null;
      context.rerender();
    });
  });

  document.querySelector<HTMLButtonElement>("#input-picker-cancel")?.addEventListener("click", () => {
    keyPickerBindingId = null;
    context.rerender();
  });

  document.querySelectorAll<HTMLButtonElement>("[data-input-matrix-row]").forEach((button) => {
    button.addEventListener("click", () => {
      if (!keyPickerBindingId) return;
      const binding = findBinding(keyPickerBindingId);
      if (!binding) return;
      binding.trigger = 2;
      binding.triggerA = Number(button.dataset.inputMatrixRow ?? "0");
      binding.triggerB = Number(button.dataset.inputMatrixCol ?? "0");
      keyPickerBindingId = null;
      persistAndRender(context);
    });
  });

  const move = (id: string, delta: number) => {
    const index = bindings.findIndex((item) => item.id === id);
    const target = index + delta;
    if (index < 0 || target < 0 || target >= bindings.length) return;
    const next = [...bindings];
    [next[index], next[target]] = [next[target], next[index]];
    bindings = next;
    persistAndRender(context);
  };

  document.querySelectorAll<HTMLButtonElement>("[data-input-up]").forEach((button) => {
    button.addEventListener("click", () => move(button.dataset.inputUp ?? "", -1));
  });

  document.querySelectorAll<HTMLButtonElement>("[data-input-down]").forEach((button) => {
    button.addEventListener("click", () => move(button.dataset.inputDown ?? "", 1));
  });

  document.querySelectorAll<HTMLButtonElement>("[data-input-duplicate]").forEach((button) => {
    button.addEventListener("click", () => {
      if (bindings.length >= 12) return;
      const index = bindings.findIndex((item) => item.id === button.dataset.inputDuplicate);
      if (index < 0) return;
      const next = [...bindings];
      next.splice(index + 1, 0, { ...bindings[index], id: makeId() });
      bindings = next;
      persistAndRender(context);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-input-delete]").forEach((button) => {
    button.addEventListener("click", () => {
      if (bindings.length <= 1) return;
      bindings = bindings.filter((item) => item.id !== button.dataset.inputDelete);
      persistAndRender(context);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-input-preset]").forEach((button) => {
    button.addEventListener("click", () => {
      const preset = button.dataset.inputPreset;
      if (!preset) return;
      replacePreset(preset);
      context.setNotice("Input preset loaded locally. Press Apply profile to send it to the keyboard.");
    });
  });

  document.querySelector<HTMLButtonElement>("#input-apply")?.addEventListener("click", () => {
    void context.runAction(async () => {
      await invoke<string>("apply_input_profile", { bindings: encodedBindings() });
    }, "Input profile applied to volatile keyboard RAM.");
  });

  document.querySelector<HTMLButtonElement>("#input-disable")?.addEventListener("click", () => {
    void context.runAction(async () => {
      await invoke<string>("disable_input_router");
    }, "Input Router disabled; safe Volume/Mute fallback is active.");
  });

  document.querySelector<HTMLButtonElement>("#input-defaults")?.addEventListener("click", () => {
    void context.runAction(async () => {
      await invoke<string>("restore_input_defaults");
      bindings = defaultBindings();
      saveDraft();
    }, "Safe Volume/Mute defaults restored and enabled.");
  });

  document.querySelector<HTMLButtonElement>("#input-read")?.addEventListener("click", () => {
    void context.runAction(async () => {
      const response = await invoke<string>("get_input_router_dump");
      bindings = parseDump(response);
      saveDraft();
    }, "Input bindings read from keyboard.");
  });

  document.querySelector<HTMLButtonElement>("#input-profile-save")?.addEventListener("click", () => {
    const name = window.prompt("Input profile name", `Input Profile ${profiles.length + 1}`)?.trim();
    if (!name) return;

    profiles = [
      ...profiles,
      {
        id: makeId(),
        name,
        bindings: cloneBindings(bindings),
        createdAt: new Date().toLocaleString(),
      },
    ];
    saveProfiles();
    context.setNotice(`Saved input profile ${name}.`);
  });

  document.querySelectorAll<HTMLButtonElement>("[data-input-profile-load]").forEach((button) => {
    button.addEventListener("click", () => {
      const profile = profiles.find((item) => item.id === button.dataset.inputProfileLoad);
      if (!profile) return;
      bindings = cloneBindings(profile.bindings);
      saveDraft();
      context.setNotice(`Loaded ${profile.name} locally.`);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-input-profile-duplicate]").forEach((button) => {
    button.addEventListener("click", () => {
      const profile = profiles.find((item) => item.id === button.dataset.inputProfileDuplicate);
      if (!profile) return;
      profiles = [
        ...profiles,
        {
          id: makeId(),
          name: `${profile.name} Copy`,
          bindings: cloneBindings(profile.bindings),
          createdAt: new Date().toLocaleString(),
        },
      ];
      saveProfiles();
      context.setNotice(`Duplicated ${profile.name}.`);
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-input-profile-delete]").forEach((button) => {
    button.addEventListener("click", () => {
      const profile = profiles.find((item) => item.id === button.dataset.inputProfileDelete);
      if (!profile) return;
      if (!window.confirm(`Delete ${profile.name}?`)) return;
      profiles = profiles.filter((item) => item.id !== profile.id);
      saveProfiles();
      context.setNotice(`Deleted ${profile.name}.`);
    });
  });
}
