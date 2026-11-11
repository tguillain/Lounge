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
  appId: number;
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
