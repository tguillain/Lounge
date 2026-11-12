import { describe, expect, it } from "vitest";
import { captureShortcut, shortcutLabel, type KeyPress } from "./shortcut";

const press = (key: string, code: string, modifiers: Partial<KeyPress> = {}): KeyPress => ({
  key,
  code,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  metaKey: false,
  ...modifiers,
});

describe("captureShortcut", () => {
  it("keeps listening while only modifiers are held", () => {
    expect(captureShortcut(press("Control", "ControlLeft", { ctrlKey: true }))).toEqual({ kind: "waiting" });
    expect(captureShortcut(press("Shift", "ShiftLeft", { ctrlKey: true, shiftKey: true }))).toEqual({ kind: "waiting" });
  });

  it("records modifiers in a fixed order before the key", () => {
    expect(captureShortcut(press("Q", "KeyQ", { shiftKey: true, ctrlKey: true }))).toEqual({ kind: "shortcut", shortcut: "Ctrl+Shift+KeyQ" });
    expect(captureShortcut(press("&", "Digit1", { altKey: true }))).toEqual({ kind: "shortcut", shortcut: "Alt+Digit1" });
  });

  it("follows the letter printed on the key, so AZERTY keyboards get what they pressed", () => {
    expect(captureShortcut(press("a", "KeyQ", { ctrlKey: true }))).toEqual({ kind: "shortcut", shortcut: "Ctrl+KeyA" });
  });

  it("falls back to the key position when the key doesn't print a letter", () => {
    expect(captureShortcut(press("€", "KeyE", { ctrlKey: true, altKey: true }))).toEqual({ kind: "shortcut", shortcut: "Ctrl+Alt+KeyE" });
  });

  it("accepts function and navigation keys on their own", () => {
    expect(captureShortcut(press("F10", "F10"))).toEqual({ kind: "shortcut", shortcut: "F10" });
    expect(captureShortcut(press("Pause", "Pause"))).toEqual({ kind: "shortcut", shortcut: "Pause" });
    expect(captureShortcut(press("PageUp", "PageUp", { shiftKey: true }))).toEqual({ kind: "shortcut", shortcut: "Shift+PageUp" });
  });

  it("asks for Ctrl or Alt before taking a key games need", () => {
    expect(captureShortcut(press("q", "KeyQ")).kind).toBe("invalid");
    expect(captureShortcut(press("Tab", "Tab", { shiftKey: true })).kind).toBe("invalid");
  });

  it("refuses keys Windows keeps for itself", () => {
    expect(captureShortcut(press("F12", "F12")).kind).toBe("invalid");
    expect(captureShortcut(press("g", "KeyG", { metaKey: true })).kind).toBe("invalid");
    expect(captureShortcut(press("Tab", "Tab", { altKey: true })).kind).toBe("invalid");
    expect(captureShortcut(press("F4", "F4", { altKey: true })).kind).toBe("invalid");
    expect(captureShortcut(press("Enter", "Enter", { altKey: true })).kind).toBe("invalid");
  });

  it("refuses punctuation, whose place changes from one keyboard to another", () => {
    expect(captureShortcut(press(";", "Semicolon", { ctrlKey: true })).kind).toBe("invalid");
  });

  it("cancels on Esc and turns the shortcut off on Backspace or Delete", () => {
    expect(captureShortcut(press("Escape", "Escape"))).toEqual({ kind: "cancel" });
    expect(captureShortcut(press("Backspace", "Backspace"))).toEqual({ kind: "clear" });
    expect(captureShortcut(press("Delete", "Delete"))).toEqual({ kind: "clear" });
  });
});

describe("shortcutLabel", () => {
  it("names keys the way they're printed", () => {
    expect(shortcutLabel("Ctrl+Shift+KeyQ")).toBe("Ctrl + Shift + Q");
    expect(shortcutLabel("Alt+Digit1")).toBe("Alt + 1");
    expect(shortcutLabel("Ctrl+Numpad5")).toBe("Ctrl + Num 5");
    expect(shortcutLabel("Shift+PageUp")).toBe("Shift + Page Up");
    expect(shortcutLabel("F10")).toBe("F10");
  });
});
