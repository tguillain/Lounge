import { describe, expect, it } from "vitest";
import { sectionsOf, settingsRows, type SettingsRow } from "./settings";
import type { Preferences, UpdateStatus } from "./types";

const preferences: Preferences = {
  startWithWindows: true,
  startWithWindowsAvailable: true,
  startFullscreen: false,
  vibration: true,
  showFriends: true,
  sounds: true,
  version: "0.1.1",
  canUpdate: true,
};

const upToDate: UpdateStatus = { state: "up-to-date" };

const updateRow = (update: UpdateStatus, overrides: Partial<Preferences> = {}): SettingsRow => {
  const row = settingsRows({ ...preferences, ...overrides }, "missing", update).find((candidate) => candidate.id === "update");
  if (!row) {
    throw new Error("No update row");
  }
  return row;
};

describe("settingsRows", () => {
  it("offers to remove the Steam key only once one is saved", () => {
    expect(settingsRows(preferences, "connected", upToDate).map((row) => row.id)).toContain("remove-key");
    expect(settingsRows(preferences, "missing", upToDate).map((row) => row.id)).not.toContain("remove-key");
  });

  it("asks to connect Steam when no key is saved", () => {
    const row = settingsRows(preferences, "missing", upToDate).find((candidate) => candidate.id === "steam-key");
    expect(row?.label).toBe("Connect Steam");
  });

  it("marks Start with Windows unavailable outside the installed app", () => {
    const row = settingsRows({ ...preferences, startWithWindowsAvailable: false }, "missing", upToDate)[0];
    expect(row.kind === "toggle" && row.available).toBe(false);
    expect(row.description).toBe("Only available in the installed app.");
  });
});

describe("the update row", () => {
  it("shows the installed version when Lounge is up to date", () => {
    const row = updateRow(upToDate);
    expect(row.label).toBe("Lounge 0.1.1");
    expect(row.kind === "action" && row.value).toBe("Check again");
  });

  it("offers to install a newer version", () => {
    const row = updateRow({ state: "available", version: "0.1.3" });
    expect(row.label).toBe("Lounge 0.1.3 is available");
    expect(row.kind === "action" && row.value).toBe("Install");
  });

  it("shows the download progress and can't be pressed while installing", () => {
    const row = updateRow({ state: "installing", version: "0.1.3", percent: 42 });
    expect(row.kind === "action" && row.value).toBe("42%");
    expect(row.kind === "action" && row.inert).toBe(true);
  });

  it("can't be pressed outside the installed app", () => {
    const row = updateRow({ state: "idle" }, { canUpdate: false });
    expect(row.kind === "action" && row.inert).toBe(true);
    expect(row.description).toBe("Updates are checked in the installed app.");
  });

  it("offers to try again after a failure", () => {
    const row = updateRow({ state: "failed", message: "Couldn't check for updates." });
    expect(row.description).toBe("Couldn't check for updates.");
    expect(row.kind === "action" && row.value).toBe("Try again");
  });
});

describe("sectionsOf", () => {
  it("groups rows by section and keeps their position in the list", () => {
    const groups = sectionsOf(settingsRows(preferences, "connected", upToDate));
    expect(groups.map((group) => group.section)).toEqual(["General", "Controller", "Steam friends", "About"]);
    expect(groups[2].rows.map(({ index }) => index)).toEqual([4, 5, 6]);
    expect(groups[3].rows.map(({ row }) => row.id)).toEqual(["update", "releases"]);
  });
});
