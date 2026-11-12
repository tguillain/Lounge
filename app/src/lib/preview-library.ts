import type { Achievement, AchievementsResult, Game, Store } from "./types";

const HOUR = 3600;

const previewGame = (name: string, store: Store, hoursAgo: number, sizeGb: number, playedHours: number): Game => ({
  id: `preview:${name}`,
  store,
  name,
  lastPlayed: hoursAgo === 0 ? 0 : Math.floor(Date.now() / 1000) - hoursAgo * HOUR,
  sizeOnDisk: sizeGb * 1024 ** 3,
  playTime: store === "epic" ? null : Math.round(playedHours * 60),
  launchUri: "",
  images: { hero: null, logo: null, logoIsDark: false, header: null, cover: null },
  hidden: false,
  download: name === "The Depths" ? { kind: "update", state: "downloading", percent: 45 } : null,
});

export const previewLibrary = (): Game[] => [
  previewGame("Ashen Horizon", "steam", 2, 78, 212),
  previewGame("Neon Drift", "epic", 26, 24, 0),
  previewGame("The Depths", "steam", 72, 41, 38.5),
  previewGame("Northern Keep", "steam", 170, 12, 4.2),
  previewGame("Verdant", "steam", 340, 6, 0.6),
  previewGame("Wake", "steam", 700, 9, 1340),
  previewGame("Orbit", "steam", 2000, 15, 17),
  previewGame("Workshop 9", "steam", 0, 2, 0),
];

const PREVIEW_ACHIEVEMENTS: [string, string, number, number | null, boolean][] = [
  ["First Light", "Finish the prologue.", 94.1, 2, false],
  ["Cartographer", "Uncover every region of the map.", 18.4, 30, false],
  ["Untouchable", "Beat a boss without taking damage.", 4.2, 75, false],
  ["Collector", "Find 50 relics.", 27.5, null, false],
  ["Night Owl", "Play for an hour after midnight.", 35.2, null, false],
  ["Perfectionist", "Unlock every other achievement.", 0.04, null, false],
  ["The Truth", "Learn what happened at the lighthouse.", 12.0, null, true],
  ["Old Friends", "Reunite the crew.", 61.7, 160, true],
];

export const previewAchievements = (game: Game): AchievementsResult => {
  if (game.store !== "steam" || game.name === "Orbit") {
    return { status: "none" };
  }
  const now = Math.floor(Date.now() / 1000);
  const achievements: Achievement[] = PREVIEW_ACHIEVEMENTS.map(([name, description, rarity, hoursAgo, hidden]) => ({
    id: name,
    name,
    description,
    icon: null,
    unlockedAt: hoursAgo === null ? null : now - hoursAgo * HOUR,
    hidden,
    rarity,
  }));
  const unlocked = achievements.filter((achievement) => achievement.unlockedAt !== null).length;
  return { status: "ready", unlocked, total: achievements.length, achievements, complete: true };
};
