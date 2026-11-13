import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { PadAction } from "./input/gamepad-mapping";
import { hueOf } from "./format";
import { previewAchievements, previewLibrary, previewSessions } from "./preview-library";
import type { Downloads } from "./downloads";
import type { AchievementsResult, AudioOutput, AvailableUpdate, Battery, Friend, FriendsResult, Game, PlaySession, Preferences, Session, Toggle, Volume, GameDetails, Picture } from "./types";

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
    return { gameId: game?.id ?? "preview", startedAt: Date.now() - 72 * 60 * 1000, game };
  }
  return invoke<Session | null>("get_session");
};

const previewFriend = (id: string, name: string, presence: Friend["presence"], game: string | null = null, joinUri: string | null = null): Friend => ({
  id: `steam:${id}`,
  network: "steam",
  name,
  avatar: null,
  presence,
  game,
  platform: null,
  joinUri,
});

const previewFriends: Friend[] = [
  previewFriend("1", "Kaori", "in-game", "Rocket League", "steam://joinlobby/0/0/1"),
  previewFriend("2", "Mathis_R", "in-game", "Counter-Strike 2"),
  previewFriend("3", "Nyx", "online"),
  previewFriend("4", "Bastien", "away"),
  previewFriend("5", "Lou", "offline"),
];

const previewPsnFriends: Friend[] = [
  { id: "psn:1", network: "psn", name: "Hakoom_PS", avatar: null, presence: "in-game", game: "Astro Bot", platform: "PS5", joinUri: null },
  { id: "psn:2", network: "psn", name: "Jade-77", avatar: null, presence: "offline", game: null, platform: null, joinUri: null },
];

export const loadFriends = async (): Promise<FriendsResult> => {
  if (!runsInTauri) {
    return { status: "ready", friends: previewFriends };
  }
  return invoke<FriendsResult>("get_friends");
};

export const loadPsnFriends = async (): Promise<FriendsResult> => {
  if (!runsInTauri) {
    return { status: "ready", friends: previewPsnFriends };
  }
  return invoke<FriendsResult>("get_psn_friends");
};

export const connectPsn = async (token: string): Promise<void> => {
  if (runsInTauri) {
    await invoke("connect_psn", { token });
  }
};

export const disconnectPsn = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("disconnect_psn");
  }
};

export const openPsnPage = async (page: "sign-in" | "token"): Promise<void> => {
  if (runsInTauri) {
    await invoke("open_psn_page", { page });
  }
};

export const setFriendPinned = async (friendId: string, pinned: boolean): Promise<void> => {
  if (!runsInTauri) {
    previewPreferences.pinnedFriends = pinned
      ? [...previewPreferences.pinnedFriends, friendId]
      : previewPreferences.pinnedFriends.filter((id) => id !== friendId);
    return;
  }
  await invoke("set_friend_pinned", { friendId, pinned });
};

export const loadPlayHistory = async (): Promise<PlaySession[]> => {
  if (!runsInTauri) {
    return previewSessions();
  }
  return invoke<PlaySession[]>("get_play_history");
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

export type DualSenseUpdate = { readByLounge?: boolean; held?: PadAction[] };

export const watchDualSense = async (handle: (update: DualSenseUpdate) => void): Promise<() => void> => {
  if (!runsInTauri) {
    return () => {};
  }
  const stopConnected = await listen<boolean>("dualsense-connected", (event) => handle({ readByLounge: event.payload }));
  const stopHeld = await getCurrentWebviewWindow().listen<PadAction[]>("dualsense-held", (event) => handle({ held: event.payload }));
  handle({ readByLounge: await invoke<boolean>("is_dualsense_read_by_lounge") });
  return () => {
    stopConnected();
    stopHeld();
  };
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
  quickMenuShortcut: null,
  pinnedFriends: ["psn:2"],
  version: "0.1.14",
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

export const setQuickMenuShortcut = async (shortcut: string | null): Promise<void> => {
  if (!runsInTauri) {
    previewPreferences.quickMenuShortcut = shortcut;
    return;
  }
  await invoke("set_quick_menu_shortcut", { shortcut });
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

export const onPowerButton = async (handle: () => void): Promise<() => void> => {
  if (!runsInTauri) {
    return () => {};
  }
  return getCurrentWebviewWindow().listen("power-button", () => handle());
};

let previewOutputs: AudioOutput[] = [
  { id: "tv", name: "LG TV", detail: "NVIDIA High Definition Audio", kind: "display", current: false },
  { id: "speakers", name: "Speakers", detail: "Scarlett 2i2 USB", kind: "speakers", current: true },
  { id: "pad", name: "Headset", detail: "DualSense Wireless Controller", kind: "controller", current: false },
];

export const getAudioOutputs = async (): Promise<AudioOutput[]> => {
  if (!runsInTauri) {
    return previewOutputs.map((output) => ({ ...output }));
  }
  return invoke<AudioOutput[]>("get_audio_outputs");
};

export const setAudioOutput = async (output: AudioOutput): Promise<void> => {
  if (!runsInTauri) {
    previewOutputs = previewOutputs.map((entry) => ({ ...entry, current: entry.id === output.id }));
    return;
  }
  await invoke("set_audio_output", { id: output.id });
};

const previewShot = (hue: number): Picture => {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="900"><defs><radialGradient id="g" cx="30%" cy="30%" r="80%"><stop offset="0" stop-color="hsl(${hue} 80% 55%)"/><stop offset="1" stop-color="hsl(${hue + 40} 60% 8%)"/></radialGradient></defs><rect width="1600" height="900" fill="url(#g)"/></svg>`;
  const url = `data:image/svg+xml,${encodeURIComponent(svg)}`;
  return { thumbnail: url, full: url };
};

export const loadGameDetails = async (game: Game): Promise<GameDetails> => {
  if (!runsInTauri) {
    const hue = hueOf(game.name);
    return {
      store:
        game.store === "steam"
          ? {
              description: `${game.name} drops you into a world that keeps changing around you. Explore, fight and build your way through a story shaped by your choices.`,
              genres: ["Action", "Adventure", "RPG"],
              developers: ["Lantern Works"],
              releaseDate: "12 Mar 2024",
              screenshots: [0, 50, 110, 200, 280].map((shift) => previewShot((hue + shift) % 360)),
            }
          : null,
      installDir: game.store === "steam" ? `D:\\SteamLibrary\\steamapps\\common\\${game.name}` : null,
      screenshots: [],
    };
  }
  return invoke<GameDetails>("get_game_details", { gameId: game.id });
};

export const openGameFolder = async (game: Game): Promise<void> => {
  if (runsInTauri) {
    await invoke("open_game_folder", { gameId: game.id });
  }
};

let previewVolume: Volume = { level: 70, muted: false };

export const getVolume = async (): Promise<Volume> => {
  if (!runsInTauri) {
    return { ...previewVolume };
  }
  return invoke<Volume>("get_volume");
};

export const setVolume = async (level: number): Promise<Volume> => {
  if (!runsInTauri) {
    previewVolume = { level, muted: level > 0 ? false : previewVolume.muted };
    return { ...previewVolume };
  }
  return invoke<Volume>("set_volume", { level });
};

export const setMuted = async (muted: boolean): Promise<Volume> => {
  if (!runsInTauri) {
    previewVolume = { ...previewVolume, muted };
    return { ...previewVolume };
  }
  return invoke<Volume>("set_muted", { muted });
};

export const openLoungeSettings = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("quick_menu_open_settings");
  }
};

export const takeSettingsRequest = async (): Promise<boolean> => {
  if (!runsInTauri) {
    return false;
  }
  return invoke<boolean>("take_settings_request");
};

export const onOpenSettings = async (handle: () => void): Promise<() => void> => {
  if (!runsInTauri) {
    return () => {};
  }
  return getCurrentWebviewWindow().listen("open-settings", () => handle());
};

export const runPower = async (action: "sleep" | "restart" | "shut-down"): Promise<void> => {
  if (runsInTauri) {
    await invoke("power", { action });
  }
};

export const quitLounge = async (): Promise<void> => {
  if (runsInTauri) {
    await invoke("quit_lounge");
  }
};

export const pulseDualSense = (): void => {
  if (runsInTauri) {
    invoke("pulse_controller").catch(() => {});
  }
};
