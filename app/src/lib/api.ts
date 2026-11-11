import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { PadAction } from "./input/gamepad-mapping";
import { previewLibrary } from "./preview-library";
import type { Friend, FriendsResult, Game, Preferences, Session } from "./types";

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

export const getSession = async (): Promise<Session | null> => {
  if (!runsInTauri) {
    const [game] = previewLibrary();
    return { appId: 0, startedAt: Date.now() - 72 * 60 * 1000, game };
  }
  return invoke<Session | null>("get_session");
};

const previewFriends: Friend[] = [
  { steamId: "1", name: "Kaori", avatar: null, presence: "in-game", game: "Rocket League", joinUri: "steam://joinlobby/0/0/1" },
  { steamId: "2", name: "Mathis_R", avatar: null, presence: "in-game", game: "Counter-Strike 2", joinUri: null },
  { steamId: "3", name: "Nyx", avatar: null, presence: "online", game: null, joinUri: null },
  { steamId: "4", name: "Bastien", avatar: null, presence: "away", game: null, joinUri: null },
];

export const loadFriends = async (): Promise<FriendsResult> => {
  if (!runsInTauri) {
    return { status: "ready", friends: previewFriends };
  }
  return invoke<FriendsResult>("get_friends");
};

export const saveSteamApiKey = async (key: string): Promise<void> => {
  if (runsInTauri) {
    await invoke("save_steam_api_key", { key });
  }
};

export const forgetSteamApiKey = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("forget_steam_api_key");
  }
};

export const openSteamApiKeyPage = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("open_steam_api_key_page");
  }
};

export const joinFriend = async (friend: Friend): Promise<void> => {
  if (runsInTauri && friend.joinUri) {
    await invoke("launch_game", { uri: friend.joinUri });
  }
};

export const backToGame = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("back_to_game");
  }
};

export const resumeGame = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("quick_menu_resume");
  }
};

export const openLounge = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("quick_menu_open_lounge");
  }
};

export const quitGame = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("quick_menu_quit_game");
  }
};

export const onControllerAction = async (handle: (action: PadAction) => void): Promise<() => void> => {
  if (!runsInTauri) {
    return () => {};
  }
  return getCurrentWebviewWindow().listen<PadAction>("pad-action", (event) => handle(event.payload));
};

export const toggleFullscreen = async (): Promise<void> => {
  if (!runsInTauri) {
    return;
  }
  const appWindow = getCurrentWindow();
  await appWindow.setFullscreen(!(await appWindow.isFullscreen()));
};

const previewPreferences: Preferences = { startWithWindows: false, startWithWindowsAvailable: true, startFullscreen: false };

export const getPreferences = async (): Promise<Preferences> => {
  if (!runsInTauri) {
    return { ...previewPreferences };
  }
  return invoke<Preferences>("get_preferences");
};

export const setStartWithWindows = async (enabled: boolean): Promise<void> => {
  if (!runsInTauri) {
    previewPreferences.startWithWindows = enabled;
    return;
  }
  await invoke("set_start_with_windows", { enabled });
};

export const setStartFullscreen = async (enabled: boolean): Promise<void> => {
  if (!runsInTauri) {
    previewPreferences.startFullscreen = enabled;
    return;
  }
  await invoke("set_start_fullscreen", { enabled });
};
