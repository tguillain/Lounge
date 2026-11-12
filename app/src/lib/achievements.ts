import type { Achievement, AchievementsResult } from "./types";

export const RARE_BELOW_PERCENT = 10;

export const orderAchievements = (achievements: Achievement[]): Achievement[] => {
  const unlocked = achievements
    .filter((achievement) => achievement.unlockedAt !== null)
    .sort((a, b) => (b.unlockedAt ?? 0) - (a.unlockedAt ?? 0));
  const locked = achievements
    .filter((achievement) => achievement.unlockedAt === null)
    .sort((a, b) => Number(a.hidden) - Number(b.hidden) || (b.rarity ?? -1) - (a.rarity ?? -1));
  return [...unlocked, ...locked];
};

export const isSecret = (achievement: Achievement): boolean => achievement.hidden && achievement.unlockedAt === null;

export const isRare = (achievement: Achievement): boolean =>
  achievement.rarity !== null && achievement.rarity < RARE_BELOW_PERCENT;

export const progressPercent = (unlocked: number, total: number): number =>
  total === 0 ? 0 : Math.floor((unlocked / total) * 100);

export const countLabel = (result: AchievementsResult): string | null => {
  if (result.status === "loading") {
    return "Loading…";
  }
  return result.status === "ready" ? `${result.unlocked} of ${result.total}` : null;
};

export const rarityPercent = (rarity: number): string =>
  rarity < 0.1 ? "<0.1%" : `${rarity < 10 ? rarity.toFixed(1) : Math.round(rarity)}%`;

export const rarityLabel = (rarity: number | null): string | null => {
  if (rarity === null) {
    return null;
  }
  if (rarity < 0.1) {
    return "Under 0.1% of players";
  }
  return `${rarity < 10 ? rarity.toFixed(1) : Math.round(rarity)}% of players`;
};

export const unlockedLabel = (unlockedAt: number): string =>
  unlockedAt === 0
    ? "Unlocked"
    : `Unlocked ${new Date(unlockedAt * 1000).toLocaleDateString("en-US", { month: "long", day: "numeric", year: "numeric" })}`;

export type SessionHighlights = { unlockedThisSession: Achievement[]; upNext: Achievement[] };

export const sessionHighlights = (achievements: Achievement[], startedAtMs: number, limit = 3): SessionHighlights => {
  const startedAt = Math.floor(startedAtMs / 1000);
  const ordered = orderAchievements(achievements);
  return {
    unlockedThisSession: ordered.filter((a) => a.unlockedAt !== null && a.unlockedAt >= startedAt).slice(0, limit),
    upNext: ordered.filter((a) => a.unlockedAt === null && !a.hidden).slice(0, limit),
  };
};
