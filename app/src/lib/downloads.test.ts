import { describe, expect, it } from "vitest";
import { applyDownloads, downloadLabel } from "./downloads";
import { previewLibrary } from "./preview-library";
import type { Download } from "./types";

const updating: Download = { kind: "update", state: "downloading", percent: 45 };

describe("downloadLabel", () => {
  it("says whether Steam installs or updates the game, with its progress", () => {
    expect(downloadLabel(updating)).toBe("Updating 45%");
    expect(downloadLabel({ kind: "install", state: "downloading", percent: 3 })).toBe("Installing 3%");
    expect(downloadLabel({ kind: "install", state: "downloading", percent: null })).toBe("Installing");
  });

  it("names paused and queued downloads", () => {
    expect(downloadLabel({ ...updating, state: "paused" })).toBe("Download paused");
    expect(downloadLabel({ ...updating, state: "queued" })).toBe("Update queued");
    expect(downloadLabel({ kind: "install", state: "queued", percent: 0 })).toBe("Waiting to install");
  });
});

describe("applyDownloads", () => {
  const games = previewLibrary().map((game) => ({ ...game, download: null }));
  const id = games[0].id;

  it("puts the latest progress on the matching game", () => {
    const { games: next, needsRescan } = applyDownloads(games, { [id]: updating });
    expect(next[0].download).toEqual(updating);
    expect(next[1]).toBe(games[1]);
    expect(needsRescan).toBe(false);
  });

  it("asks for a rescan when a download finishes, so the size and install state refresh", () => {
    const downloading = applyDownloads(games, { [id]: updating }).games;
    const { games: next, needsRescan } = applyDownloads(downloading, {});
    expect(next[0].download).toBeNull();
    expect(needsRescan).toBe(true);
  });

  it("asks for a rescan when Steam starts installing a game Lounge doesn't list yet", () => {
    expect(applyDownloads(games, { "steam:999": updating }).needsRescan).toBe(true);
  });
});
