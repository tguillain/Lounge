import {
  RepeatGate,
  buttonAction,
  familyOf,
  isDualSense,
  stickDirection,
  type ControllerFamily,
  type PadAction,
} from "./gamepad-mapping";

const connectedPads = (): Gamepad[] => navigator.getGamepads().filter((pad): pad is Gamepad => pad !== null);

const heldActions = (pad: Gamepad): Set<PadAction> => {
  const held = new Set<PadAction>();
  pad.buttons.forEach((button, index) => {
    const action = button.pressed ? buttonAction(index) : null;
    if (action) {
      held.add(action);
    }
  });
  const stick = stickDirection(pad.axes[0] ?? 0, pad.axes[1] ?? 0);
  if (stick) {
    held.add(stick);
  }
  return held;
};

// Lounge's Rust core reads the DualSense over HID. While it does, the
// browser's view of that controller is ignored: over Bluetooth, the browser
// stops seeing it once the controller sends the full reports that carry its
// battery level.
const dualSense = { readByLounge: false, held: new Set<PadAction>() };
let wakeUp: (() => void) | null = null;

export const updateDualSense = (update: { readByLounge?: boolean; held?: PadAction[] }) => {
  if (update.readByLounge !== undefined) {
    dualSense.readByLounge = update.readByLounge;
    if (!update.readByLounge) {
      dualSense.held = new Set();
    }
  }
  if (update.held) {
    dualSense.held = new Set(update.held);
  }
  wakeUp?.();
};

const NOTHING_HELD: ReadonlySet<PadAction> = new Set();

export const watchGamepads = (onAction: (action: PadAction, family: ControllerFamily) => void): (() => void) => {
  const gates = new Map<number, RepeatGate>();
  const dualSenseGate = new RepeatGate();
  let frame = 0;

  const poll = (now: number) => {
    frame = 0;
    const pads = connectedPads().filter((pad) => !(dualSense.readByLounge && isDualSense(pad.id)));
    if (pads.length === 0 && !dualSense.readByLounge) {
      gates.clear();
      return;
    }
    for (const pad of pads) {
      const gate = gates.get(pad.index) ?? new RepeatGate();
      gates.set(pad.index, gate);
      for (const action of gate.update(heldActions(pad), now)) {
        onAction(action, familyOf(pad.id));
      }
    }
    const held = document.hasFocus() ? dualSense.held : NOTHING_HELD;
    for (const action of dualSenseGate.update(held, now)) {
      onAction(action, "playstation");
    }
    frame = requestAnimationFrame(poll);
  };

  const start = () => {
    if (frame === 0) {
      frame = requestAnimationFrame(poll);
    }
  };

  wakeUp = start;
  window.addEventListener("gamepadconnected", start);
  if (connectedPads().length > 0 || dualSense.readByLounge) {
    start();
  }

  return () => {
    wakeUp = null;
    window.removeEventListener("gamepadconnected", start);
    cancelAnimationFrame(frame);
  };
};

export const pulse = () => {
  for (const pad of connectedPads()) {
    pad.vibrationActuator
      ?.playEffect("dual-rumble", { duration: 18, weakMagnitude: 0.25, strongMagnitude: 0 })
      .catch(() => {});
  }
};
