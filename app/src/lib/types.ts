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
  launchUri: string;
  images: GameImages;
  hidden: boolean;
};

export type Session = {
  appId: number;
  startedAt: number;
  game: Game | null;
};

export type LibraryState =
  | { status: "loading" }
  | { status: "ready"; games: Game[] }
  | { status: "error"; message: string };
