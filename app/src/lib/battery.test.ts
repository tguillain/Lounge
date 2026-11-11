import { describe, expect, it } from "vitest";
import { batteryDescription, batteryTone } from "./battery";

describe("batteryTone", () => {
  it("warns when the battery is low", () => {
    expect(batteryTone({ percent: 15, charging: false })).toBe("low");
    expect(batteryTone({ percent: 20, charging: false })).toBe("low");
    expect(batteryTone({ percent: 25, charging: false })).toBe("normal");
  });

  it("doesn't warn while the controller charges", () => {
    expect(batteryTone({ percent: 5, charging: true })).toBe("charging");
  });
});

describe("batteryDescription", () => {
  it("reads the level and the charging state", () => {
    expect(batteryDescription({ percent: 65, charging: false })).toBe("Controller battery 65%");
    expect(batteryDescription({ percent: 100, charging: true })).toBe("Controller battery 100%, charging");
  });
});
