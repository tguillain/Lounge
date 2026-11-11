import type { Preferences, Toggle } from "./types";

export type SteamKeyState = "connected" | "missing" | "unknown";

export type SettingsSection = "General" | "Controller" | "Steam friends" | "About";

type RowBase = { id: string; section: SettingsSection; label: string; description: string };

export type SettingsRow =
  | (RowBase & { kind: "toggle"; toggle: Toggle | "startWithWindows"; on: boolean; available: boolean })
  | (RowBase & { kind: "action"; action: "steam-key" | "remove-key" | "releases"; value?: string; danger?: boolean });

const steamKeyValue: Record<SteamKeyState, string> = {
  connected: "Connected",
  missing: "Not connected",
  unknown: "…",
};

export const settingsRows = (preferences: Preferences, steamKey: SteamKeyState): SettingsRow[] => [
  {
    id: "start-with-windows",
    section: "General",
    kind: "toggle",
    toggle: "startWithWindows",
    label: "Start with Windows",
    description: preferences.startWithWindowsAvailable
      ? "Open Lounge when you sign in to Windows."
      : "Only available in the installed app.",
    on: preferences.startWithWindows,
    available: preferences.startWithWindowsAvailable,
  },
  {
    id: "start-fullscreen",
    section: "General",
    kind: "toggle",
    toggle: "startFullscreen",
    label: "Open in full screen",
    description: "Applies the next time Lounge opens.",
    on: preferences.startFullscreen,
    available: true,
  },
  {
    id: "vibration",
    section: "Controller",
    kind: "toggle",
    toggle: "vibration",
    label: "Vibration",
    description: "A light tap as you move between games.",
    on: preferences.vibration,
    available: true,
  },
  {
    id: "show-friends",
    section: "Steam friends",
    kind: "toggle",
    toggle: "showFriends",
    label: "Show friends on Home",
    description: "See which Steam friends are online and join their games.",
    on: preferences.showFriends,
    available: true,
  },
  {
    id: "steam-key",
    section: "Steam friends",
    kind: "action",
    action: "steam-key",
    label: steamKey === "connected" ? "Change Steam Web API key" : "Connect Steam",
    description: "Lounge needs a free Steam Web API key to see your friends.",
    value: steamKeyValue[steamKey],
  },
  ...(steamKey === "connected"
    ? [
        {
          id: "remove-key",
          section: "Steam friends",
          kind: "action",
          action: "remove-key",
          label: "Remove key",
          description: "Forget the key stored in Windows Credential Manager.",
          danger: true,
        } satisfies SettingsRow,
      ]
    : []),
  {
    id: "releases",
    section: "About",
    kind: "action",
    action: "releases",
    label: `Lounge ${preferences.version}`,
    description: "See what's new and download updates on GitHub.",
    value: "Open",
  },
];

export const sectionsOf = (rows: SettingsRow[]): { section: SettingsSection; rows: { row: SettingsRow; index: number }[] }[] => {
  const groups: { section: SettingsSection; rows: { row: SettingsRow; index: number }[] }[] = [];
  rows.forEach((row, index) => {
    const last = groups.at(-1);
    if (last?.section === row.section) {
      last.rows.push({ row, index });
    } else {
      groups.push({ section: row.section, rows: [{ row, index }] });
    }
  });
  return groups;
};
