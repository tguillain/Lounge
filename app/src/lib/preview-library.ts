import type { Game, Store } from "./types";

const HOUR = 3600;

const previewGame = (name: string, store: Store, hoursAgo: number, sizeGb: number): Game => ({
  id: `preview:${name}`,
  store,
  name,
  lastPlayed: hoursAgo === 0 ? 0 : Math.floor(Date.now() / 1000) - hoursAgo * HOUR,
  sizeOnDisk: sizeGb * 1024 ** 3,
  launchUri: "",
  images: { hero: null, logo: null, logoIsDark: false, header: null, cover: null },
  hidden: false,
});

export const previewLibrary = (): Game[] => [
  previewGame("Ashen Horizon", "steam", 2, 78),
  previewGame("Neon Drift", "epic", 26, 24),
  previewGame("The Depths", "steam", 72, 41),
  previewGame("Northern Keep", "steam", 170, 12),
  previewGame("Verdant", "steam", 340, 6),
  previewGame("Wake", "steam", 700, 9),
  previewGame("Orbit", "steam", 2000, 15),
  previewGame("Workshop 9", "steam", 0, 2),
];
