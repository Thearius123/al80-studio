export type CreatorEffectId = "solid" | "pulse" | "comet" | "snake";

export interface CreatorEffectDefinition {
  id: CreatorEffectId;
  name: string;
  description: string;
  usesSecondary: boolean;
  usesTail: boolean;
}

export interface CreatorEffectSpec {
  effect: CreatorEffectId;
  primary: string;
  secondary: string;
  speed: number;
  tailLength: number;
  phase: number;
}

export const CREATOR_EFFECTS: readonly CreatorEffectDefinition[] = [
  {
    id: "solid",
    name: "Solid",
    description: "One color across the full recovered LED order.",
    usesSecondary: false,
    usesTail: false,
  },
  {
    id: "pulse",
    name: "Pulse",
    description: "Interpolates between two colors using a deterministic phase.",
    usesSecondary: true,
    usesTail: false,
  },
  {
    id: "comet",
    name: "Comet",
    description: "A bright head with a fading tail over the LED order.",
    usesSecondary: true,
    usesTail: true,
  },
  {
    id: "snake",
    name: "Snake",
    description: "A moving typed segment with head and tail colors.",
    usesSecondary: true,
    usesTail: true,
  },
] as const;

const HEX = /^#[0-9a-fA-F]{6}$/;

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function normalizeHex(value: string): string {
  if (!HEX.test(value)) {
    throw new Error(`Invalid effect color: ${value}`);
  }

  return value.toLowerCase();
}

function hexToRgb(value: string): [number, number, number] {
  const color = normalizeHex(value);
  return [
    Number.parseInt(color.slice(1, 3), 16),
    Number.parseInt(color.slice(3, 5), 16),
    Number.parseInt(color.slice(5, 7), 16),
  ];
}

function rgbToHex(r: number, g: number, b: number): string {
  return `#${[r, g, b]
    .map((channel) =>
      clamp(Math.round(channel), 0, 255).toString(16).padStart(2, "0"),
    )
    .join("")}`;
}

function mixHex(a: string, b: string, amount: number): string {
  const left = hexToRgb(a);
  const right = hexToRgb(b);
  const t = clamp(amount, 0, 1);

  return rgbToHex(
    left[0] + (right[0] - left[0]) * t,
    left[1] + (right[1] - left[1]) * t,
    left[2] + (right[2] - left[2]) * t,
  );
}

export function normalizeCreatorEffectSpec(
  value: CreatorEffectSpec,
): CreatorEffectSpec {
  if (!CREATOR_EFFECTS.some((effect) => effect.id === value.effect)) {
    throw new Error(`Unknown Creator effect: ${value.effect}`);
  }

  return {
    effect: value.effect,
    primary: normalizeHex(value.primary),
    secondary: normalizeHex(value.secondary),
    speed: clamp(Math.round(value.speed), 1, 10),
    tailLength: clamp(Math.round(value.tailLength), 1, 32),
    phase: clamp(value.phase, 0, 1),
  };
}

function blankFrame(ledCount: number, color: string): string[] {
  return Array.from({ length: ledCount }, () => color);
}

function validLedOrder(order: readonly number[], ledCount: number): number[] {
  const seen = new Set<number>();

  return order.filter((led) => {
    if (!Number.isInteger(led) || led < 0 || led >= ledCount || seen.has(led)) {
      return false;
    }

    seen.add(led);
    return true;
  });
}

export function renderCreatorEffectFrame(
  rawSpec: CreatorEffectSpec,
  rawOrder: readonly number[],
  ledCount = 82,
): string[] {
  if (!Number.isInteger(ledCount) || ledCount <= 0 || ledCount > 512) {
    throw new Error(`Invalid Creator LED count: ${ledCount}`);
  }

  const spec = normalizeCreatorEffectSpec(rawSpec);
  const order = validLedOrder(rawOrder, ledCount);

  if (order.length === 0) {
    throw new Error("Creator effect LED order is empty");
  }

  if (spec.effect === "solid") {
    return blankFrame(ledCount, spec.primary);
  }

  if (spec.effect === "pulse") {
    const wave = (Math.sin(spec.phase * Math.PI * 2) + 1) / 2;
    return blankFrame(
      ledCount,
      mixHex(spec.secondary, spec.primary, wave),
    );
  }

  const frame = blankFrame(ledCount, spec.secondary);
  const head =
    Math.floor(spec.phase * order.length * spec.speed) % order.length;
  const length = clamp(spec.tailLength, 1, order.length);

  for (let offset = 0; offset < length; offset += 1) {
    const index = (head - offset + order.length) % order.length;
    const led = order[index];

    if (spec.effect === "snake") {
      frame[led] = offset === 0 ? spec.primary : mixHex(
        spec.primary,
        spec.secondary,
        offset / Math.max(1, length - 1),
      );
      continue;
    }

    const fade = 1 - offset / Math.max(1, length);
    frame[led] = mixHex(spec.secondary, spec.primary, fade);
  }

  return frame;
}
