import type { PadAction } from "./input/gamepad-mapping";

export type Sound = "move" | "select" | "back" | "launch";

type Note = {
  frequency: number;
  endFrequency?: number;
  start: number;
  duration: number;
  volume: number;
  wave: OscillatorType;
};

const LAUNCH_CHORD = [261.63, 329.63, 392, 523.25];

export const SOUND_NOTES: Record<Sound, Note[]> = {
  move: [{ frequency: 330, endFrequency: 260, start: 0, duration: 0.06, volume: 0.04, wave: "sine" }],
  select: [
    { frequency: 392, start: 0, duration: 0.11, volume: 0.03, wave: "sine" },
    { frequency: 523.25, start: 0.07, duration: 0.16, volume: 0.028, wave: "sine" },
  ],
  back: [
    { frequency: 523.25, start: 0, duration: 0.11, volume: 0.028, wave: "sine" },
    { frequency: 392, start: 0.07, duration: 0.16, volume: 0.03, wave: "sine" },
  ],
  launch: LAUNCH_CHORD.map((frequency, index) => ({
    frequency,
    start: index * 0.09,
    duration: 0.7 - index * 0.07,
    volume: 0.025,
    wave: "sine" as const,
  })),
};

export type NavigationChange = {
  layersBefore: number;
  layersAfter: number;
  focusMoved: boolean;
  launched: boolean;
};

export const soundFor = (action: PadAction, change: NavigationChange): Sound | null => {
  if (change.launched) {
    return "launch";
  }
  if (change.layersAfter > change.layersBefore) {
    return "select";
  }
  if (change.layersAfter < change.layersBefore) {
    return "back";
  }
  if (action === "confirm") {
    return "select";
  }
  if (!change.focusMoved) {
    return null;
  }
  return action === "back" ? "back" : "move";
};

const ATTACK_SECONDS = 0.014;
const SILENCE = 0.0001;

let context: AudioContext | null = null;

export const playSound = (sound: Sound): void => {
  if (typeof AudioContext === "undefined") {
    return;
  }
  context ??= new AudioContext();
  if (context.state === "suspended") {
    context.resume().catch(() => {});
  }
  const startsAt = context.currentTime + 0.005;
  for (const note of SOUND_NOTES[sound]) {
    const begin = startsAt + note.start;
    const end = begin + note.duration;
    const oscillator = context.createOscillator();
    const gain = context.createGain();
    oscillator.type = note.wave;
    oscillator.frequency.setValueAtTime(note.frequency, begin);
    if (note.endFrequency) {
      oscillator.frequency.exponentialRampToValueAtTime(note.endFrequency, end);
    }
    gain.gain.setValueAtTime(SILENCE, begin);
    gain.gain.linearRampToValueAtTime(note.volume, begin + ATTACK_SECONDS);
    gain.gain.exponentialRampToValueAtTime(SILENCE, end);
    oscillator.connect(gain).connect(context.destination);
    oscillator.start(begin);
    oscillator.stop(end + 0.02);
  }
};
