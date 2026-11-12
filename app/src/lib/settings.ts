import { shortcutLabel } from "./shortcut";
import type { Preferences, Toggle, UpdateStatus } from "./types";

export type SteamKeyState = "connected" | "missing" | "unknown";

export type SettingsSection = "General" | "Controller" | "In game" | "Steam friends" | "About";

type RowBase = { id: string; section: SettingsSection; label: string; description: string };

export type SettingsRow =
  | (RowBase & { kind: "toggle"; toggle: Toggle | "startWithWindows"; on: boolean; available: boolean })
  | (RowBase & {
      kind: "action";
      action: "steam-key" | "remove-key" | "quick-menu-shortcut" | "update" | "releases";
      value?: string;
      danger?: boolean;
      highlight?: boolean;
      inert?: boolean;
    });

const steamKeyValue: Record<SteamKeyState, string> = {
  connected: "Connected",
  missing: "Not connected",
  unknown: "…",
};

const updateRow = (preferences: Preferences, update: UpdateStatus): SettingsRow => {
  const row = { id: "update", section: "About", kind: "action", action: "update", label: `Lounge ${preferences.version}` } as const;
  if (!preferences.canUpdate) {
    return { ...row, description: "Updates are checked in the installed app.", inert: true };
  }
  switch (update.state) {
    case "idle":
    case "checking":
      return { ...row, description: "Checking for updates…", inert: true };
    case "up-to-date":
      return { ...row, description: "You're up to date.", value: "Check again" };
    case "available":
      return {
        ...row,
        label: `Lounge ${update.version} is available`,
        description: `You have ${preferences.version}. Lounge restarts to finish updating.`,
        value: "Install",
        highlight: true,
      };
    case "installing":
      return {
        ...row,
        label: `Updating to Lounge ${update.version}`,
        description: "Lounge restarts on its own when it's done.",
        value: update.percent === null ? "Downloading…" : `${update.percent}%`,
        inert: true,
      };
    case "failed":
      return { ...row, description: update.message, value: "Try again" };
  }
};

const shortcutRow = (preferences: Preferences, capturing: boolean): SettingsRow => ({
  id: "quick-menu-shortcut",
  section: "In game",
  kind: "action",
  action: "quick-menu-shortcut",
  label: "Quick Menu shortcut",
  description: capturing
    ? "Press the keys you want. Esc cancels, Backspace turns the shortcut off."
    : "Open the Quick Menu from the keyboard while you play.",
  value: capturing ? "Press keys…" : preferences.quickMenuShortcut ? shortcutLabel(preferences.quickMenuShortcut) : "Off",
});

export const settingsRows = (
  preferences: Preferences,
  steamKey: SteamKeyState,
  update: UpdateStatus,
  capturingShortcut = false,
): SettingsRow[] => [
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
    id: "sounds",
    section: "General",
    kind: "toggle",
    toggle: "sounds",
    label: "Navigation sounds",
    description: "Soft clicks as you move around Lounge.",
    on: preferences.sounds,
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
    id: "lightbar",
    section: "Controller",
    kind: "toggle",
    toggle: "lightbar",
    label: "Light bar matches the game",
    description: "Your DualSense glows in the colors of the selected game.",
    on: preferences.lightbar,
    available: true,
  },
  shortcutRow(preferences, capturingShortcut),
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
  updateRow(preferences, update),
  {
    id: "releases",
    section: "About",
    kind: "action",
    action: "releases",
    label: "What's new",
    description: "Release notes for every version, on GitHub.",
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
