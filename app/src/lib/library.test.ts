import { describe, expect, it } from "vitest";
import { availableFilters, filterGames, moveInGrid, nextSort, sortDetail, sortGames } from "./library";
import type { Game, Store } from "./types";

const game = (name: string, store: Store, lastPlayed: number, sizeGb: number, playTime: number | null, hidden = false): Game => ({
  id: `${store}:${name}`,
  store,
  name,
  lastPlayed,
  sizeOnDisk: sizeGb * 1024 ** 3,
  playTime,
  recentPlayTime: null,
  launchUri: "",
  images: { hero: null, logo: null, logoIsDark: false, header: null, cover: null },
  hidden,
  download: null,
});

const games = [
  game("Counter-Strike 2", "steam", 300, 69, 1122),
  game("Rocket League", "epic", 200, 25, null),
  game("beamNG.drive", "steam", 100, 52, 8677),
  game("DSX", "steam", 400, 1, 30, true),
];

describe("filterGames", () => {
  it("leaves hidden games out of every store filter", () => {
    expect(filterGames(games, "all").map((g) => g.name)).toEqual(["Counter-Strike 2", "Rocket League", "beamNG.drive"]);
    expect(filterGames(games, "steam").map((g) => g.name)).toEqual(["Counter-Strike 2", "beamNG.drive"]);
  });

  it("shows only hidden games in the hidden filter", () => {
    expect(filterGames(games, "hidden").map((g) => g.name)).toEqual(["DSX"]);
  });
});

describe("sortGames", () => {
  it("sorts by name without caring about case", () => {
    expect(sortGames(games.slice(0, 3), "name").map((g) => g.name)).toEqual(["beamNG.drive", "Counter-Strike 2", "Rocket League"]);
  });

  it("puts the biggest games first when sorting by size", () => {
    expect(sortGames(games.slice(0, 3), "size")[0].name).toBe("Counter-Strike 2");
  });

  it("puts the most played games first, and games without play time last", () => {
    expect(sortGames(games.slice(0, 3), "playtime").map((g) => g.name)).toEqual(["beamNG.drive", "Counter-Strike 2", "Rocket League"]);
  });

  it("cycles through the sort orders", () => {
    expect(nextSort("playtime")).toBe("recent");
    expect(nextSort("size")).toBe("playtime");
  });
});

describe("sortDetail", () => {
  it("shows the value the library is sorted by", () => {
    expect(sortDetail(games[2], "name", 3700)).toBe("Steam · 145 h played");
    expect(sortDetail(games[2], "playtime", 3700)).toBe("Steam · 145 h played");
    expect(sortDetail(games[2], "size", 3700)).toBe("Steam · 52 GB");
    expect(sortDetail(games[2], "recent", 3700)).toBe("Steam · 1 hour ago");
  });

  it("shows only the store when it has no play time to show", () => {
    expect(sortDetail(games[1], "name", 3700)).toBe("Epic Games");
  });

  it("keeps hours readable for short and long play times", () => {
    expect(sortDetail(game("A", "steam", 0, 1, 0), "name", 0)).toBe("Steam · Never played");
    expect(sortDetail(game("A", "steam", 0, 1, 45), "name", 0)).toBe("Steam · 45 min played");
    expect(sortDetail(game("A", "steam", 0, 1, 252), "name", 0)).toBe("Steam · 4.2 h played");
    expect(sortDetail(game("A", "steam", 0, 1, 90000), "name", 0)).toBe("Steam · 1,500 h played");
  });
});

describe("availableFilters", () => {
  it("only offers stores you have games from, plus the hidden count", () => {
    expect(availableFilters(games).map((f) => f.label)).toEqual(["All", "Steam", "Epic Games", "Hidden (1)"]);
  });
});

describe("moveInGrid", () => {
  it("moves between rows and leaves the grid from the top row", () => {
    expect(moveInGrid(6, "up", 5, 12)).toEqual({ index: 1, leftTop: false });
    expect(moveInGrid(2, "up", 5, 12)).toEqual({ index: 2, leftTop: true });
  });

  it("stays on the row edges", () => {
    expect(moveInGrid(5, "left", 5, 12)).toEqual({ index: 5, leftTop: false });
    expect(moveInGrid(4, "right", 5, 12)).toEqual({ index: 4, leftTop: false });
  });

  it("drops to the last game when the row below is shorter", () => {
    expect(moveInGrid(8, "down", 5, 12)).toEqual({ index: 11, leftTop: false });
    expect(moveInGrid(11, "down", 5, 12)).toEqual({ index: 11, leftTop: false });
  });
});
