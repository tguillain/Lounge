import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { PadAction } from "./input/gamepad-mapping";
import { hueOf } from "./format";
import { previewAchievements, previewLibrary } from "./preview-library";
import type { Downloads } from "./downloads";
import type { AchievementsResult, AvailableUpdate, Battery, Friend, FriendsResult, Game, Preferences, Session, Toggle } from "./types";

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

export const loadAchievements = async (game: Game): Promise<AchievementsResult> => {
  if (!runsInTauri) {
    return previewAchievements(game);
  }
  const appId = Number(game.id.replace(/^steam:/, ""));
  return invoke<AchievementsResult>("get_achievements", { appId });
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

const previewPreferences: Preferences = {
  startWithWindows: false,
  startWithWindowsAvailable: true,
  startFullscreen: false,
  vibration: true,
  showFriends: true,
  sounds: true,
  lightbar: true,
  version: "0.1.8",
  canUpdate: true,
};

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

export const setToggle = async (toggle: Toggle, enabled: boolean): Promise<void> => {
  if (!runsInTauri) {
    previewPreferences[toggle] = enabled;
    return;
  }
  await invoke("set_toggle", { toggle, enabled });
};

export const openReleasesPage = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("open_releases_page");
  }
};

export const checkForUpdate = async (): Promise<AvailableUpdate | null> => {
  if (!runsInTauri) {
    return { version: "0.1.3" };
  }
  return invoke<AvailableUpdate | null>("check_for_update");
};

export const installUpdate = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("install_update");
  }
};

export const onUpdateProgress = async (handle: (percent: number | null) => void): Promise<() => void> => {
  if (!runsInTauri) {
    return () => {};
  }
  return listen<{ percent: number | null }>("update-progress", (event) => handle(event.payload.percent));
};

export const watchControllerBattery = async (handle: (battery: Battery | null) => void): Promise<() => void> => {
  if (!runsInTauri) {
    handle({ percent: 65, charging: false });
    return () => {};
  }
  const stop = await listen<Battery | null>("controller-battery", (event) => handle(event.payload));
  handle(await invoke<Battery | null>("get_controller_battery"));
  return stop;
};

export const onSteamDownloads = async (handle: (downloads: Downloads) => void): Promise<() => void> => {
  if (!runsInTauri) {
    return () => {};
  }
  return listen<Downloads>("steam-downloads", (event) => handle(event.payload));
};

export const setLightbar = async (game: Game): Promise<void> => {
  if (runsInTauri) {
    await invoke("set_lightbar", { image: game.images.header ?? game.images.hero, fallbackHue: hueOf(game.name) });
  }
};

export const releaseLightbar = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("release_lightbar");
  }
};

export const pulseDualSense = (): void => {
  if (runsInTauri) {
    invoke("pulse_controller").catch(() => {});
  }
};
