import { describe, expect, it } from "vitest";
import { sectionsOf, settingsRows } from "./settings";
import type { Preferences } from "./types";

const preferences: Preferences = {
  startWithWindows: true,
  startWithWindowsAvailable: true,
  startFullscreen: false,
  vibration: true,
  showFriends: true,
  version: "0.1.1",
};

describe("settingsRows", () => {
  it("offers to remove the Steam key only once one is saved", () => {
    expect(settingsRows(preferences, "connected").map((row) => row.id)).toContain("remove-key");
    expect(settingsRows(preferences, "missing").map((row) => row.id)).not.toContain("remove-key");
  });

  it("asks to connect Steam when no key is saved", () => {
    const row = settingsRows(preferences, "missing").find((candidate) => candidate.id === "steam-key");
    expect(row?.label).toBe("Connect Steam");
  });

  it("marks Start with Windows unavailable outside the installed app", () => {
    const row = settingsRows({ ...preferences, startWithWindowsAvailable: false }, "missing")[0];
    expect(row.kind === "toggle" && row.available).toBe(false);
    expect(row.description).toBe("Only available in the installed app.");
  });

  it("shows the installed version in About", () => {
    expect(settingsRows(preferences, "missing").at(-1)?.label).toBe("Lounge 0.1.1");
  });
});

describe("sectionsOf", () => {
  it("groups rows by section and keeps their position in the list", () => {
    const groups = sectionsOf(settingsRows(preferences, "connected"));
    expect(groups.map((group) => group.section)).toEqual(["General", "Controller", "Steam friends", "About"]);
    expect(groups[2].rows.map(({ index }) => index)).toEqual([3, 4, 5]);
  });
});
