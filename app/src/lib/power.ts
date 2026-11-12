export type PowerChoice = "sleep" | "restart" | "shut-down" | "quit";

export const POWER_CHOICES: { id: PowerChoice; label: string }[] = [
  { id: "sleep", label: "Sleep" },
  { id: "restart", label: "Restart" },
  { id: "shut-down", label: "Shut down" },
  { id: "quit", label: "Quit Lounge" },
];

export type PowerConfirmation = { title: string; action: string; note: string };

const CONFIRMATIONS: Record<Exclude<PowerChoice, "sleep">, PowerConfirmation> = {
  restart: { title: "Restart the PC?", action: "Restart", note: "Save your work in other apps first." },
  "shut-down": { title: "Shut down the PC?", action: "Shut down", note: "Save your work in other apps first." },
  quit: { title: "Quit Lounge?", action: "Quit Lounge", note: "Open it again from the Start menu or the desktop." },
};

export const confirmationOf = (choice: PowerChoice): PowerConfirmation | null =>
  choice === "sleep" ? null : CONFIRMATIONS[choice];
