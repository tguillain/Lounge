import { describe, expect, it } from "vitest";
import { FIRST_SPOT, itemAt, moveSpot, spotOf, stepVolume } from "./quick-menu";

const walk = (from: Parameters<typeof spotOf>[0], ...moves: ("up" | "down" | "left" | "right")[]) =>
  itemAt(moves.reduce(moveSpot, spotOf(from)));

describe("moveSpot", () => {
  it("starts on Back to Lounge", () => {
    expect(itemAt(FIRST_SPOT)).toBe("lounge");
  });

  it("keeps the column when the rows are the same width", () => {
    expect(walk("quit", "down")).toBe("output");
    expect(walk("output", "up")).toBe("quit");
  });

  it("lands on the matching side of a wider or narrower row", () => {
    expect(walk("volume", "down")).toBe("sleep");
    expect(walk("shut-down", "up")).toBe("volume");
    expect(walk("output", "down", "down")).toBe("sleep");
    expect(walk("quit", "up")).toBe("settings");
    expect(walk("settings", "down")).toBe("lounge");
  });

  it("stops at the edges instead of wrapping around", () => {
    expect(walk("settings", "up")).toBe("settings");
    expect(walk("sleep", "down")).toBe("sleep");
    expect(walk("lounge", "left")).toBe("lounge");
    expect(walk("shut-down", "right")).toBe("shut-down");
  });

  it("moves sideways inside a row", () => {
    expect(walk("sleep", "right", "right")).toBe("shut-down");
    expect(walk("mute", "right")).toBe("output");
  });
});

describe("stepVolume", () => {
  it("moves by 5 and snaps an odd level to the next step", () => {
    expect(stepVolume(70, "right")).toBe(75);
    expect(stepVolume(70, "left")).toBe(65);
    expect(stepVolume(42, "right")).toBe(45);
    expect(stepVolume(42, "left")).toBe(40);
  });

  it("stays between 0 and 100", () => {
    expect(stepVolume(100, "right")).toBe(100);
    expect(stepVolume(0, "left")).toBe(0);
    expect(stepVolume(98, "right")).toBe(100);
  });
});
