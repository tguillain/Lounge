export type Store = "steam" | "epic";

export type GameImages = {
  hero: string | null;
  logo: string | null;
  logoIsDark: boolean;
  header: string | null;
  cover: string | null;
};

export type Download = {
  kind: "install" | "update";
  state: "downloading" | "paused" | "queued";
  percent: number | null;
};

export type Game = {
  id: string;
  store: Store;
  name: string;
  lastPlayed: number;
  sizeOnDisk: number;
  playTime: number | null;
  launchUri: string;
  images: GameImages;
  hidden: boolean;
  download: Download | null;
};

export type Session = {
  gameId: string;
  startedAt: number;
  game: Game | null;
};

export type Presence = "in-game" | "online" | "away";

export type Friend = {
  steamId: string;
  name: string;
  avatar: string | null;
  presence: Presence;
  game: string | null;
  joinUri: string | null;
};

export type FriendsResult =
  | { status: "loading" }
  | { status: "not-configured" }
  | { status: "ready"; friends: Friend[] }
  | { status: "failed"; message: string };

export type LibraryState =
  | { status: "loading" }
  | { status: "ready"; games: Game[] }
  | { status: "error"; message: string };

export type Preferences = {
  startWithWindows: boolean;
  startWithWindowsAvailable: boolean;
  startFullscreen: boolean;
  vibration: boolean;
  showFriends: boolean;
  sounds: boolean;
  lightbar: boolean;
  quickMenuShortcut: string | null;
  version: string;
  canUpdate: boolean;
};

export type Battery = { percent: number; charging: boolean };

export type AvailableUpdate = { version: string };

export type UpdateStatus =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "up-to-date" }
  | { state: "available"; version: string }
  | { state: "installing"; version: string; percent: number | null }
  | { state: "failed"; message: string };

export type Toggle = "startFullscreen" | "sounds" | "vibration" | "lightbar" | "showFriends";

export type Achievement = {
  id: string;
  name: string;
  description: string | null;
  icon: string | null;
  unlockedAt: number | null;
  hidden: boolean;
  rarity: number | null;
};

export type AchievementsResult =
  | { status: "loading" }
  | { status: "none" }
  | { status: "ready"; unlocked: number; total: number; achievements: Achievement[]; complete: boolean }
  | { status: "failed"; message: string };

export type AudioOutputKind = "speakers" | "headphones" | "headset" | "display" | "controller" | "other";

export type Volume = { level: number; muted: boolean };

export type Picture = { thumbnail: string; full: string };

export type StoreInfo = {
  description: string | null;
  genres: string[];
  developers: string[];
  releaseDate: string | null;
  screenshots: Picture[];
};

export type GameDetails = {
  store: StoreInfo | null;
  installDir: string | null;
  screenshots: Picture[];
};

export type DetailsResult = { status: "loading" } | ({ status: "ready" } & GameDetails) | { status: "failed"; message: string };

export type AudioOutput = {
  id: string;
  name: string;
  detail: string | null;
  kind: AudioOutputKind;
  current: boolean;
};
