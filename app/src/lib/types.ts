export type Store = "steam" | "epic";

export type GameImages = {
  hero: string | null;
  logo: string | null;
  logoIsDark: boolean;
  header: string | null;
  cover: string | null;
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
  version: string;
};

export type Toggle = "startFullscreen" | "vibration" | "showFriends";
