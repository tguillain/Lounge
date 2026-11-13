import { describe, expect, it } from "vitest";
import {
  FIRST_DETAIL_SPOT,
  creditsLine,
  detailActions,
  friendsPlaying,
  genresLine,
  libraryFolder,
  moveDetail,
  picturesOf,
} from "./game-detail";
import type { Friend, GameDetails, StoreInfo } from "./types";

const store: StoreInfo = {
  description: "Drive anything.",
  genres: ["Racing", "Simulation", "Early Access", "Indie"],
  developers: ["BeamNG", "Other"],
  releaseDate: "29 May 2015",
  screenshots: [{ thumbnail: "store-1-small", full: "store-1" }],
};

const friend = (name: string, game: string | null): Friend => ({ steamId: name, name, avatar: null, presence: "online", game, joinUri: null });

describe("detailActions", () => {
  it("always offers Play and Options, and the rest when they apply", () => {
    expect(detailActions({ achievements: true, folder: true })).toEqual(["play", "achievements", "folder", "options"]);
    expect(detailActions({ achievements: false, folder: false })).toEqual(["play", "options"]);
  });
});

describe("moveDetail", () => {
  it("moves along the buttons and stops at both ends", () => {
    expect(moveDetail(FIRST_DETAIL_SPOT, "right", 4, 5)).toEqual({ row: "actions", index: 1 });
    expect(moveDetail({ row: "actions", index: 3 }, "right", 4, 5)).toEqual({ row: "actions", index: 3 });
    expect(moveDetail(FIRST_DETAIL_SPOT, "left", 4, 5)).toEqual(FIRST_DETAIL_SPOT);
  });

  it("goes down to the screenshots and back up to Play", () => {
    expect(moveDetail({ row: "actions", index: 2 }, "down", 4, 5)).toEqual({ row: "pictures", index: 0 });
    expect(moveDetail({ row: "pictures", index: 3 }, "up", 4, 5)).toEqual(FIRST_DETAIL_SPOT);
  });

  it("stays on the buttons when there are no screenshots", () => {
    expect(moveDetail({ row: "actions", index: 1 }, "down", 4, 0)).toEqual({ row: "actions", index: 1 });
  });

  it("asks to leave when going up from the buttons", () => {
    expect(moveDetail(FIRST_DETAIL_SPOT, "up", 4, 5)).toBeNull();
  });
});

describe("store lines", () => {
  it("keeps the first three genres", () => {
    expect(genresLine(store)).toBe("Racing · Simulation · Early Access");
    expect(genresLine({ ...store, genres: [] })).toBeNull();
    expect(genresLine(null)).toBeNull();
  });

  it("names the main developer and the release date", () => {
    expect(creditsLine(store)).toBe("BeamNG · 29 May 2015");
    expect(creditsLine({ ...store, developers: [] })).toBe("29 May 2015");
    expect(creditsLine(null)).toBeNull();
  });
});

describe("friendsPlaying", () => {
  it("finds the friends in this game, whatever the case", () => {
    const friends = [friend("Kaori", "BeamNG.drive"), friend("Nyx", null), friend("Mathis", "beamng.drive "), friend("Bastien", "Rocket League")];
    expect(friendsPlaying(friends, "BeamNG.drive").map((found) => found.name)).toEqual(["Kaori", "Mathis"]);
  });
});

describe("libraryFolder", () => {
  it("shows the Steam library the game lives in", () => {
    expect(libraryFolder("D:\\SteamLibrary\\steamapps\\common\\BeamNG.drive")).toBe("D:\\SteamLibrary");
    expect(libraryFolder("C:\\Program Files (x86)\\Steam\\SteamApps\\common\\Game")).toBe("C:\\Program Files (x86)\\Steam");
    expect(libraryFolder("E:\\Games\\Thing")).toBe("E:\\Games\\Thing");
    expect(libraryFolder(null)).toBeNull();
  });
});

describe("picturesOf", () => {
  const details = (screenshots: GameDetails["screenshots"], withStore = true): GameDetails => ({
    store: withStore ? store : null,
    installDir: null,
    screenshots,
  });

  it("prefers your own screenshots", () => {
    expect(picturesOf(details([{ thumbnail: "mine-small", full: "mine" }]))?.title).toBe("Your screenshots");
  });

  it("falls back to the store's", () => {
    expect(picturesOf(details([]))).toEqual({ title: "Screenshots", pictures: store.screenshots });
  });

  it("has nothing to show without either", () => {
    expect(picturesOf(details([], false))).toBeNull();
    expect(picturesOf(null)).toBeNull();
  });
});
