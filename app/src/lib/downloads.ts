import type { Download, Game } from "./types";

export type Downloads = Record<string, Download>;

export const downloadLabel = (download: Download): string => {
  if (download.state === "paused") {
    return "Download paused";
  }
  if (download.state === "queued") {
    return download.kind === "install" ? "Waiting to install" : "Update queued";
  }
  const action = download.kind === "install" ? "Installing" : "Updating";
  return download.percent === null ? action : `${action} ${download.percent}%`;
};

const sameDownload = (a: Download | null, b: Download | null): boolean =>
  a === b || (a !== null && b !== null && a.kind === b.kind && a.state === b.state && a.percent === b.percent);

export const applyDownloads = (games: Game[], downloads: Downloads): { games: Game[]; needsRescan: boolean } => {
  const known = new Set(games.map((game) => game.id));
  const finished = games.some((game) => game.download !== null && !downloads[game.id]);
  const started = Object.keys(downloads).some((id) => !known.has(id));
  return {
    games: games.map((game) => {
      const download = downloads[game.id] ?? null;
      return sameDownload(game.download, download) ? game : { ...game, download };
    }),
    needsRescan: finished || started,
  };
};
