import type { Friend, GameDetails, Picture, StoreInfo } from "./types";

export type DetailAction = "play" | "achievements" | "folder" | "options";

export type DetailSpot = { row: "actions" | "pictures"; index: number };

export const FIRST_DETAIL_SPOT: DetailSpot = { row: "actions", index: 0 };

const MAX_GENRES = 3;

export const detailActions = (options: { achievements: boolean; folder: boolean }): DetailAction[] => [
  "play",
  ...(options.achievements ? (["achievements"] as const) : []),
  ...(options.folder ? (["folder"] as const) : []),
  "options",
];

export const moveDetail = (
  spot: DetailSpot,
  direction: "up" | "down" | "left" | "right",
  actionCount: number,
  pictureCount: number,
): DetailSpot | null => {
  const length = spot.row === "actions" ? actionCount : pictureCount;
  if (direction === "left" || direction === "right") {
    const index = Math.max(0, Math.min(length - 1, spot.index + (direction === "right" ? 1 : -1)));
    return { row: spot.row, index };
  }
  if (spot.row === "actions") {
    if (direction === "up") {
      return null;
    }
    return pictureCount > 0 ? { row: "pictures", index: 0 } : spot;
  }
  return direction === "up" ? { row: "actions", index: 0 } : spot;
};

export const genresLine = (store: StoreInfo | null): string | null =>
  store && store.genres.length > 0 ? store.genres.slice(0, MAX_GENRES).join(" · ") : null;

export const creditsLine = (store: StoreInfo | null): string | null => {
  const parts = [store?.developers[0], store?.releaseDate].filter((part): part is string => Boolean(part));
  return parts.length > 0 ? parts.join(" · ") : null;
};

export const friendsPlaying = (friends: Friend[], gameName: string): Friend[] => {
  const wanted = gameName.trim().toLowerCase();
  return friends.filter((friend) => friend.game?.trim().toLowerCase() === wanted);
};

export const libraryFolder = (installDir: string | null): string | null => {
  if (!installDir) {
    return null;
  }
  const at = installDir.toLowerCase().indexOf("\\steamapps\\");
  return at > 0 ? installDir.slice(0, at) : installDir;
};

export const picturesOf = (details: GameDetails | null): { title: string; pictures: Picture[] } | null => {
  if (details && details.screenshots.length > 0) {
    return { title: "Your screenshots", pictures: details.screenshots };
  }
  if (details?.store && details.store.screenshots.length > 0) {
    return { title: "Screenshots", pictures: details.store.screenshots };
  }
  return null;
};
