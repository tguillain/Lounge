import { describe, expect, it } from "vitest";
import { soundFor, type NavigationChange } from "./sounds";

const nothing: NavigationChange = { layersBefore: 0, layersAfter: 0, focusMoved: false, launched: false };

describe("soundFor", () => {
  it("ticks when the focus moves", () => {
    expect(soundFor("right", { ...nothing, focusMoved: true })).toBe("move");
    expect(soundFor("nextTab", { ...nothing, focusMoved: true })).toBe("move");
  });

  it("stays quiet when nothing moves", () => {
    expect(soundFor("left", nothing)).toBeNull();
    expect(soundFor("back", nothing)).toBeNull();
  });

  it("plays select when a menu or Settings opens, and back when it closes", () => {
    expect(soundFor("options", { ...nothing, layersAfter: 1, focusMoved: true })).toBe("select");
    expect(soundFor("menu", { ...nothing, layersBefore: 1, focusMoved: true })).toBe("back");
    expect(soundFor("back", { ...nothing, layersBefore: 1, focusMoved: true })).toBe("back");
  });

  it("plays back when leaving the Library for Home", () => {
    expect(soundFor("back", { ...nothing, focusMoved: true })).toBe("back");
  });

  it("plays select when confirming", () => {
    expect(soundFor("confirm", nothing)).toBe("select");
  });

  it("plays the launch chime when a game starts, whatever else changed", () => {
    expect(soundFor("confirm", { ...nothing, launched: true, focusMoved: true })).toBe("launch");
  });
});
