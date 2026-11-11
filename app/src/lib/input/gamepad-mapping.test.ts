import { describe, expect, it } from "vitest";
import {
  INITIAL_REPEAT_DELAY_MS,
  REPEAT_INTERVAL_MS,
  RepeatGate,
  buttonAction,
  familyOf,
  stickDirection,
  type PadAction,
} from "./gamepad-mapping";

describe("familyOf", () => {
  it("recognises a PS5 controller from its vendor id", () => {
    expect(familyOf("DualSense Wireless Controller (STANDARD GAMEPAD Vendor: 054c Product: 0ce6)")).toBe("playstation");
  });

  it("treats an Xbox controller as xbox", () => {
    expect(familyOf("Xbox 360 Controller (XInput STANDARD GAMEPAD)")).toBe("xbox");
  });
});

describe("buttonAction", () => {
  it("maps cross, circle and the PS button", () => {
    expect(buttonAction(0)).toBe("confirm");
    expect(buttonAction(1)).toBe("back");
    expect(buttonAction(16)).toBe("menu");
  });

  it("ignores triggers", () => {
    expect(buttonAction(6)).toBeNull();
  });
});

describe("stickDirection", () => {
  it("ignores small movements", () => {
    expect(stickDirection(0.2, -0.3)).toBeNull();
  });

  it("keeps the dominant axis", () => {
    expect(stickDirection(0.9, 0.6)).toBe("right");
    expect(stickDirection(-0.2, -0.8)).toBe("up");
  });
});

describe("RepeatGate", () => {
  const held = (...actions: PadAction[]) => new Set(actions);

  it("fires once on press and not again while a button stays held", () => {
    const gate = new RepeatGate();
    expect(gate.update(held("confirm"), 0)).toEqual(["confirm"]);
    expect(gate.update(held("confirm"), 5000)).toEqual([]);
  });

  it("repeats a held direction after the initial delay", () => {
    const gate = new RepeatGate();
    expect(gate.update(held("right"), 0)).toEqual(["right"]);
    expect(gate.update(held("right"), INITIAL_REPEAT_DELAY_MS - 1)).toEqual([]);
    expect(gate.update(held("right"), INITIAL_REPEAT_DELAY_MS)).toEqual(["right"]);
    expect(gate.update(held("right"), INITIAL_REPEAT_DELAY_MS + REPEAT_INTERVAL_MS)).toEqual(["right"]);
  });

  it("fires again after a release", () => {
    const gate = new RepeatGate();
    gate.update(held("confirm"), 0);
    gate.update(held(), 10);
    expect(gate.update(held("confirm"), 20)).toEqual(["confirm"]);
  });
});
