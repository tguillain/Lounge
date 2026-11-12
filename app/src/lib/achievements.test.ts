import { describe, expect, it } from "vitest";
import { countLabel, isRare, isSecret, orderAchievements, progressPercent, rarityLabel, unlockedLabel } from "./achievements";
import type { Achievement } from "./types";

const achievement = (id: string, unlockedAt: number | null, rarity: number | null, hidden = false): Achievement => ({
  id,
  name: id,
  description: null,
  icon: null,
  unlockedAt,
  hidden,
  rarity,
});

describe("orderAchievements", () => {
  it("lists the latest unlocks first, then the easiest locked ones, secrets last", () => {
    const ordered = orderAchievements([
      achievement("old", 100, 50),
      achievement("rare-locked", null, 2),
      achievement("secret", null, 60, true),
      achievement("new", 300, 10),
      achievement("common-locked", null, 70),
      achievement("unknown-locked", null, null),
    ]);
    expect(ordered.map((a) => a.id)).toEqual(["new", "old", "common-locked", "rare-locked", "unknown-locked", "secret"]);
  });
});

describe("achievement labels", () => {
  it("counts progress", () => {
    expect(countLabel({ status: "ready", unlocked: 18, total: 35, achievements: [], complete: true })).toBe("18 of 35");
    expect(countLabel({ status: "none" })).toBeNull();
    expect(progressPercent(18, 35)).toBe(51);
    expect(progressPercent(34, 35)).toBe(97);
    expect(progressPercent(0, 0)).toBe(0);
  });

  it("describes rarity", () => {
    expect(rarityLabel(63.34)).toBe("63% of players");
    expect(rarityLabel(4.21)).toBe("4.2% of players");
    expect(rarityLabel(0.04)).toBe("Under 0.1% of players");
    expect(rarityLabel(null)).toBeNull();
    expect(isRare(achievement("a", null, 4.2))).toBe(true);
    expect(isRare(achievement("a", null, null))).toBe(false);
  });

  it("keeps locked hidden achievements secret", () => {
    expect(isSecret(achievement("a", null, 5, true))).toBe(true);
    expect(isSecret(achievement("a", 100, 5, true))).toBe(false);
  });

  it("dates unlocks", () => {
    expect(unlockedLabel(Date.UTC(2025, 2, 4, 12) / 1000)).toBe("Unlocked March 4, 2025");
    expect(unlockedLabel(0)).toBe("Unlocked");
  });
});
