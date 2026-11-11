import type { Battery } from "./types";

export type BatteryTone = "charging" | "low" | "normal";

export const LOW_BATTERY_PERCENT = 20;

export const batteryTone = (battery: Battery): BatteryTone => {
  if (battery.charging) {
    return "charging";
  }
  return battery.percent <= LOW_BATTERY_PERCENT ? "low" : "normal";
};

export const batteryDescription = (battery: Battery): string =>
  `Controller battery ${battery.percent}%${battery.charging ? ", charging" : ""}`;
