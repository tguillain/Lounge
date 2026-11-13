import { describe, expect, it } from "vitest";
import { lastPlayedLabel, playTimeLabel, playTimeValue, sizeLabel, totalPlayTimeLabel } from "./format";

const MINUTE_MS = 60_000;

describe("playTimeValue", () => {
  it("shows minutes, then hours with one decimal under 10, then whole hours", () => {
    expect(playTimeValue(45)).toBe("45 min");
    expect(playTimeValue(210)).toBe("3.5 h");
    expect(playTimeValue(12_720)).toBe("212 h");
    expect(playTimeValue(90_000)).toBe("1,500 h");
  });

  it("has nothing to show for a game never played or not tracked", () => {
    expect(playTimeValue(0)).toBeNull();
    expect(playTimeValue(null)).toBeNull();
  });

  it("keeps the longer label for the Library and the Quick Menu", () => {
    expect(totalPlayTimeLabel(12_720)).toBe("212 h played");
    expect(totalPlayTimeLabel(0)).toBe("Never played");
    expect(totalPlayTimeLabel(null)).toBeNull();
  });
});

describe("playTimeLabel", () => {
  it("says the game just started during the first minute", () => {
    expect(playTimeLabel(0, 30_000)).toBe("Just started");
  });

  it("shows minutes under an hour", () => {
    expect(playTimeLabel(0, 12 * MINUTE_MS)).toBe("Playing for 12 min");
  });

  it("shows hours and padded minutes", () => {
    expect(playTimeLabel(0, 72 * MINUTE_MS)).toBe("Playing for 1 h 12");
    expect(playTimeLabel(0, 125 * MINUTE_MS)).toBe("Playing for 2 h 05");
  });
});

describe("lastPlayedLabel", () => {
  const now = 1_000_000;

  it("returns nothing for a game never played", () => {
    expect(lastPlayedLabel(0, now)).toBeNull();
  });

  it("uses singular and plural units", () => {
    expect(lastPlayedLabel(now - 3600, now)).toBe("1 hour ago");
    expect(lastPlayedLabel(now - 3 * 86_400, now)).toBe("3 days ago");
  });
});

describe("sizeLabel", () => {
  it("keeps one decimal under 10 GB", () => {
    expect(sizeLabel(2.5 * 1024 ** 3)).toBe("2.5 GB");
    expect(sizeLabel(78 * 1024 ** 3)).toBe("78 GB");
  });
});
