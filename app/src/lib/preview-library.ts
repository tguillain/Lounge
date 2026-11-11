import type { Game, Store } from "./types";

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
