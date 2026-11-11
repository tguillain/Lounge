import { lastPlayedLabel, sizeLabel, storeLabel, totalPlayTimeLabel } from "./format";
import type { Game } from "./types";

export type LibraryFilter = "all" | "steam" | "epic" | "hidden";
export type LibrarySort = "name" | "recent" | "playtime" | "size";
export type FilterOption = { id: LibraryFilter; label: string };
export type GridStep = "left" | "right" | "up" | "down";
export type GridMove = { index: number; leftTop: boolean };

const SORT_ORDER: LibrarySort[] = ["playtime", "recent", "name", "size"];

export const DEFAULT_SORT: LibrarySort = "playtime";

export const SORT_LABELS: Record<LibrarySort, string> = {
  playtime: "Play time",
  recent: "Recently played",
  name: "Name",
  size: "Size",
};

export const nextSort = (sort: LibrarySort): LibrarySort => SORT_ORDER[(SORT_ORDER.indexOf(sort) + 1) % SORT_ORDER.length];

export const filterGames = (games: Game[], filter: LibraryFilter): Game[] => {
  if (filter === "hidden") {
    return games.filter((game) => game.hidden);
  }
  const visible = games.filter((game) => !game.hidden);
  return filter === "all" ? visible : visible.filter((game) => game.store === filter);
};

export const sortGames = (games: Game[], sort: LibrarySort): Game[] => {
  const byName = (a: Game, b: Game) => a.name.localeCompare(b.name, "en", { sensitivity: "base" });
  const sorted = [...games];
  if (sort === "name") {
    return sorted.sort(byName);
  }
  if (sort === "size") {
    return sorted.sort((a, b) => b.sizeOnDisk - a.sizeOnDisk || byName(a, b));
  }
  if (sort === "playtime") {
    return sorted.sort((a, b) => (b.playTime ?? -1) - (a.playTime ?? -1) || byName(a, b));
  }
  return sorted.sort((a, b) => b.lastPlayed - a.lastPlayed || byName(a, b));
};

export const sortDetail = (game: Game, sort: LibrarySort, nowSeconds: number): string => {
  const value =
    sort === "size"
      ? sizeLabel(game.sizeOnDisk)
      : sort === "recent"
        ? (lastPlayedLabel(game.lastPlayed, nowSeconds) ?? "Never played")
        : totalPlayTimeLabel(game.playTime);
  return [storeLabel(game.store), value].filter(Boolean).join(" · ");
};

export const availableFilters = (games: Game[]): FilterOption[] => {
  const visible = games.filter((game) => !game.hidden);
  const hiddenCount = games.length - visible.length;
  const options: FilterOption[] = [{ id: "all", label: "All" }];
  if (visible.some((game) => game.store === "steam")) {
    options.push({ id: "steam", label: "Steam" });
  }
  if (visible.some((game) => game.store === "epic")) {
    options.push({ id: "epic", label: "Epic Games" });
  }
  if (hiddenCount > 0) {
    options.push({ id: "hidden", label: `Hidden (${hiddenCount})` });
  }
  return options;
};

export const moveInGrid = (index: number, step: GridStep, columns: number, length: number): GridMove => {
  const stay = { index, leftTop: false };
  if (length === 0) {
    return stay;
  }
  const column = index % columns;
  if (step === "left") {
    return column === 0 ? stay : { index: index - 1, leftTop: false };
  }
  if (step === "right") {
    return column === columns - 1 || index === length - 1 ? stay : { index: index + 1, leftTop: false };
  }
  if (step === "up") {
    return index < columns ? { index, leftTop: true } : { index: index - columns, leftTop: false };
  }
  const below = index + columns;
  if (below < length) {
    return { index: below, leftTop: false };
  }
  const lastRowStart = length - (length % columns || columns);
  return index < lastRowStart ? { index: length - 1, leftTop: false } : stay;
};
