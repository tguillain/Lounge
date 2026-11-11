import { RepeatGate, buttonAction, familyOf, stickDirection, type ControllerFamily, type PadAction } from "./gamepad-mapping";

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

export const watchGamepads = (onAction: (action: PadAction, family: ControllerFamily) => void): (() => void) => {
  const gates = new Map<number, RepeatGate>();
  let frame = 0;

  const poll = (now: number) => {
    frame = 0;
    const pads = connectedPads();
    if (pads.length === 0) {
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
    frame = requestAnimationFrame(poll);
  };

  const start = () => {
    if (frame === 0) {
      frame = requestAnimationFrame(poll);
    }
  };

  window.addEventListener("gamepadconnected", start);
  if (connectedPads().length > 0) {
    start();
  }

  return () => {
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
