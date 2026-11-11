import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { previewLibrary } from "./preview-library";
import type { Game } from "./types";

const runsInTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const loadLibrary = async (): Promise<Game[]> => {
  if (!runsInTauri) {
    return previewLibrary();
  }
  return invoke<Game[]>("get_library");
};

export const launchGame = async (game: Game): Promise<void> => {
  if (!runsInTauri) {
    return;
  }
  await invoke("launch_game", { uri: game.launchUri });
};

export const setGameHidden = async (game: Game, hidden: boolean): Promise<void> => {
  if (!runsInTauri) {
    return;
  }
  await invoke("set_game_hidden", { gameId: game.id, hidden });
};

export const imageUrl =(source: string | null): string | null => {
  if (!source) {
    return null;
  }
  if (/^https?:\/\//.test(source) || !runsInTauri) {
    return source;
  }
  return convertFileSrc(source);
};

export const onGameSession = async (
  handleStarted: (gameId: string) => void,
  handleEnded: (gameId: string) => void,
): Promise<() => void> => {
  if (!runsInTauri) {
    return () => {};
  }
  const stopStarted = await listen<string>("game-started", (event) => handleStarted(event.payload));
  const stopEnded = await listen<string>("game-ended", (event) => handleEnded(event.payload));
  return () => {
    stopStarted();
    stopEnded();
  };
};

export const toggleFullscreen = async (): Promise<void> => {
  if (!runsInTauri) {
    return;
  }
  const appWindow = getCurrentWindow();
  await appWindow.setFullscreen(!(await appWindow.isFullscreen()));
};
