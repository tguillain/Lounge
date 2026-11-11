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
  previewGame("Horizon de Cendres", "steam", 2, 78),
  previewGame("Neon Drift", "epic", 26, 24),
  previewGame("Les Abysses", "steam", 72, 41),
  previewGame("Forteresse Nord", "steam", 170, 12),
  previewGame("Verdure", "steam", 340, 6),
  previewGame("Sillage", "steam", 700, 9),
  previewGame("Orbite", "steam", 2000, 15),
  previewGame("Atelier 9", "steam", 0, 2),
];
