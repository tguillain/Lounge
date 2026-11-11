export type PadAction = "left" | "right" | "up" | "down" | "confirm" | "back" | "options" | "menu";

export type ControllerFamily = "playstation" | "xbox";

export type InputMode = "keyboard" | ControllerFamily;

const STANDARD_BUTTON_ACTIONS: Record<number, PadAction> = {
  0: "confirm",
  1: "back",
  3: "options",
  9: "options",
  12: "up",
  13: "down",
  14: "left",
  15: "right",
  16: "menu",
};

const DIRECTIONS: ReadonlySet<PadAction> = new Set(["left", "right", "up", "down"]);
const SONY_PATTERN = /054c|dualsense|dualshock|playstation|wireless controller/i;
const STICK_DEADZONE = 0.5;

export const INITIAL_REPEAT_DELAY_MS = 380;
export const REPEAT_INTERVAL_MS = 110;

export const familyOf = (gamepadId: string): ControllerFamily => (SONY_PATTERN.test(gamepadId) ? "playstation" : "xbox");

export const buttonAction = (index: number): PadAction | null => STANDARD_BUTTON_ACTIONS[index] ?? null;

export const stickDirection = (x: number, y: number): PadAction | null => {
  if (Math.abs(x) < STICK_DEADZONE && Math.abs(y) < STICK_DEADZONE) {
    return null;
  }
  if (Math.abs(x) >= Math.abs(y)) {
    return x > 0 ? "right" : "left";
  }
  return y > 0 ? "down" : "up";
};

export class RepeatGate {
  private nextFireAt = new Map<PadAction, number>();

  update(held: ReadonlySet<PadAction>, now: number): PadAction[] {
    const fired: PadAction[] = [];
    for (const action of held) {
      const scheduled = this.nextFireAt.get(action);
      if (scheduled === undefined) {
        fired.push(action);
        this.nextFireAt.set(action, DIRECTIONS.has(action) ? now + INITIAL_REPEAT_DELAY_MS : Number.POSITIVE_INFINITY);
      } else if (now >= scheduled) {
        fired.push(action);
        this.nextFireAt.set(action, now + REPEAT_INTERVAL_MS);
      }
    }
    for (const action of [...this.nextFireAt.keys()]) {
      if (!held.has(action)) {
        this.nextFireAt.delete(action);
      }
    }
    return fired;
  }
}
