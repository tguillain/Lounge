import type { Store } from "./types";

const MINUTE = 60;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;

const STORE_LABELS: Record<Store, string> = {
  steam: "Steam",
  epic: "Epic Games",
};

export const storeLabel = (store: Store): string => STORE_LABELS[store];

const plural = (count: number, unit: string): string => `${count} ${unit}${count === 1 ? "" : "s"} ago`;

export const lastPlayedLabel = (lastPlayed: number, nowSeconds: number): string | null => {
  if (lastPlayed === 0) {
    return null;
  }
  const elapsed = Math.max(0, nowSeconds - lastPlayed);
  if (elapsed < HOUR) {
    return `${Math.max(1, Math.round(elapsed / MINUTE))} min ago`;
  }
  if (elapsed < DAY) {
    return plural(Math.round(elapsed / HOUR), "hour");
  }
  if (elapsed < 2 * DAY) {
    return "yesterday";
  }
  if (elapsed < WEEK) {
    return plural(Math.floor(elapsed / DAY), "day");
  }
  return `on ${new Date(lastPlayed * 1000).toLocaleDateString("en-US", { month: "long", day: "numeric" })}`;
};

export const sizeLabel = (bytes: number): string | null => {
  if (bytes <= 0) {
    return null;
  }
  const gigabytes = bytes / 1024 ** 3;
  return `${gigabytes < 10 ? gigabytes.toFixed(1) : Math.round(gigabytes)} GB`;
};

export const playTimeValue = (minutes: number | null): string | null => {
  if (minutes === null || minutes === 0) {
    return null;
  }
  if (minutes < 60) {
    return `${minutes} min`;
  }
  const hours = minutes / 60;
  return `${hours < 10 ? hours.toFixed(1) : Math.round(hours).toLocaleString("en-US")} h`;
};

export const totalPlayTimeLabel = (minutes: number | null): string | null => {
  if (minutes === 0) {
    return "Never played";
  }
  const value = playTimeValue(minutes);
  return value ? `${value} played` : null;
};

export const greeting =(date: Date): string => {
  const hour = date.getHours();
  if (hour >= 5 && hour < 12) {
    return "Good morning";
  }
  if (hour >= 12 && hour < 18) {
    return "Good afternoon";
  }
  return "Good evening";
};

export const playTimeLabel = (startedAtMs: number, nowMs: number): string => {
  const minutes = Math.max(0, Math.floor((nowMs - startedAtMs) / 60_000));
  if (minutes < 1) {
    return "Just started";
  }
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  return hours === 0 ? `Playing for ${rest} min` : `Playing for ${hours} h ${String(rest).padStart(2, "0")}`;
};

export const dateLabel = (date: Date): string =>
  date.toLocaleDateString("en-US", { weekday: "long", month: "long", day: "numeric" });

export const clockLabel =(date: Date): string => date.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });

export const hueOf = (text: string): number => {
  let hash = 0;
  for (const character of text) {
    hash = (hash * 31 + character.charCodeAt(0)) % 360;
  }
  return hash;
};

export const placeholderArt = (name: string): string => {
  const hue = hueOf(name);
  return [
    `radial-gradient(60% 80% at 75% 30%, hsl(${hue} 80% 55% / 0.85) 0%, transparent 70%)`,
    `radial-gradient(70% 90% at 10% 100%, hsl(${(hue + 40) % 360} 70% 40% / 0.8) 0%, transparent 70%)`,
    `linear-gradient(160deg, hsl(${hue} 45% 12%) 0%, hsl(${hue} 40% 5%) 100%)`,
  ].join(", ");
};
