import type { AudioOutput } from "./types";

export const outputTitle = (output: AudioOutput): string =>
  output.kind === "display" ? output.name : (output.detail ?? output.name);

export const currentOutputLabel = (outputs: AudioOutput[] | null): string | undefined => {
  if (!outputs) {
    return undefined;
  }
  const current = outputs.find((output) => output.current);
  return current ? outputTitle(current) : "None";
};

export const markCurrent = (outputs: AudioOutput[], id: string): AudioOutput[] =>
  outputs.map((output) => ({ ...output, current: output.id === id }));
