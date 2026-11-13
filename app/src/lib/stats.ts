import type { Game, PlaySession } from "./types";

export type StatsPeriod = "recent" | "all";

export type DayBar = { label: string; minutes: number; today: boolean; future: boolean };
export type WeekBar = { start: Date; minutes: number; current: boolean };
export type RankedGame = { game: Game; minutes: number };
export type Streak = { current: number; best: number };

const DAY_LETTERS = ["M", "T", "W", "T", "F", "S", "S"];
const RECENT_DAYS = 14;

export const startOfDay = (date: Date): Date => new Date(date.getFullYear(), date.getMonth(), date.getDate());

export const addDays = (date: Date, days: number): Date => new Date(date.getFullYear(), date.getMonth(), date.getDate() + days);

/** Weeks start on Monday. */
export const startOfWeek = (date: Date): Date => addDays(startOfDay(date), -((date.getDay() + 6) % 7));

const dayKey = (date: Date): string => `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`;

/** Minutes of play inside [from, to), cutting sessions that cross either edge. */
export const minutesBetween = (sessions: PlaySession[], from: Date, to: Date, gameId?: string): number => {
  const total = sessions.reduce((sum, session) => {
    if (gameId && session.gameId !== gameId) {
      return sum;
    }
    const start = Math.max(session.start * 1000, from.getTime());
    const end = Math.min(session.end * 1000, to.getTime());
    return end > start ? sum + (end - start) / 60_000 : sum;
  }, 0);
  return Math.round(total);
};

export const thisWeek = (sessions: PlaySession[], now: Date): DayBar[] => {
  const monday = startOfWeek(now);
  const today = startOfDay(now);
  return DAY_LETTERS.map((label, index) => {
    const day = addDays(monday, index);
    return {
      label,
      minutes: minutesBetween(sessions, day, addDays(day, 1)),
      today: day.getTime() === today.getTime(),
      future: day.getTime() > today.getTime(),
    };
  });
};

export const weekTotals = (sessions: PlaySession[], now: Date, count = 12, gameId?: string): WeekBar[] => {
  const current = startOfWeek(now);
  return Array.from({ length: count }, (_, index) => {
    const start = addDays(current, -7 * (count - 1 - index));
    return { start, minutes: minutesBetween(sessions, start, addDays(start, 7), gameId), current: index === count - 1 };
  });
};

/** Days in a row with some play. A streak that reaches yesterday is still alive today. */
export const streaks = (sessions: PlaySession[], now: Date): Streak => {
  const played = new Set<string>();
  for (const session of sessions) {
    const last = startOfDay(new Date(session.end * 1000));
    for (let day = startOfDay(new Date(session.start * 1000)); day <= last; day = addDays(day, 1)) {
      played.add(dayKey(day));
    }
  }
  if (played.size === 0) {
    return { current: 0, best: 0 };
  }
  const runFrom = (day: Date): number => {
    let length = 0;
    while (played.has(dayKey(addDays(day, -length)))) {
      length += 1;
    }
    return length;
  };
  const today = startOfDay(now);
  const current = played.has(dayKey(today)) ? runFrom(today) : runFrom(addDays(today, -1));
  const days = sessions.flatMap((session) => [session.start, session.end]).map((seconds) => startOfDay(new Date(seconds * 1000)));
  const best = Math.max(current, ...days.map(runFrom));
  return { current, best };
};

/** The last two weeks: Steam's own count when it's there, or what Lounge saw, whichever is more. */
export const recentMinutes = (game: Game, sessions: PlaySession[], now: Date): number => {
  const counted = minutesBetween(sessions, addDays(startOfDay(now), -(RECENT_DAYS - 1)), now, game.id);
  return Math.max(game.recentPlayTime ?? 0, counted);
};

export const mostPlayed = (games: Game[], sessions: PlaySession[], now: Date, period: StatsPeriod, limit = 5): RankedGame[] =>
  games
    .map((game) => ({ game, minutes: period === "recent" ? recentMinutes(game, sessions, now) : (game.playTime ?? 0) }))
    .filter((ranked) => ranked.minutes > 0)
    .sort((a, b) => b.minutes - a.minutes || a.game.name.localeCompare(b.game.name))
    .slice(0, limit);

export const totalMinutes = (games: Game[]): number => games.reduce((sum, game) => sum + (game.playTime ?? 0), 0);

export const sessionsOf = (gameId: string, sessions: PlaySession[]): PlaySession[] =>
  sessions.filter((session) => session.gameId === gameId).sort((a, b) => b.start - a.start);

export const sessionMinutes = (session: PlaySession): number => Math.round((session.end - session.start) / 60);

export const durationLabel = (minutes: number): string => {
  if (minutes < 60) {
    return `${Math.max(0, Math.round(minutes))} min`;
  }
  const hours = Math.floor(minutes / 60);
  const rest = Math.round(minutes % 60);
  if (hours >= 100) {
    return `${Math.round(minutes / 60).toLocaleString("en-US")} h`;
  }
  return rest === 0 ? `${hours} h` : `${hours} h ${String(rest).padStart(2, "0")}`;
};

/** A short figure for a bar: "2 h", "1.5 h", "45 min". */
export const barLabel = (minutes: number): string => {
  if (minutes === 0) {
    return "";
  }
  if (minutes < 60) {
    return `${minutes} min`;
  }
  const hours = minutes / 60;
  return `${Number.isInteger(Math.round(hours * 10) / 10) ? Math.round(hours) : hours.toFixed(1)} h`;
};

export const sessionWhen = (session: PlaySession, now: Date): string => {
  const started = new Date(session.start * 1000);
  const time = started.toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" });
  const daysAgo = Math.round((startOfDay(now).getTime() - startOfDay(started).getTime()) / 86_400_000);
  if (daysAgo === 0) {
    return `Today, ${time}`;
  }
  if (daysAgo === 1) {
    return `Yesterday, ${time}`;
  }
  if (daysAgo < 7) {
    return `${started.toLocaleDateString("en-US", { weekday: "long" })}, ${time}`;
  }
  return `${started.toLocaleDateString("en-US", { month: "short", day: "numeric" })}, ${time}`;
};

export const weekLabel = (week: WeekBar): string =>
  week.current ? "This week" : week.start.toLocaleDateString("en-US", { month: "short", day: "numeric" });

export const differenceLabel = (minutes: number, previous: number): string | null => {
  const difference = minutes - previous;
  if (previous === 0 && minutes === 0) {
    return null;
  }
  if (difference === 0) {
    return "Same as last week";
  }
  return `${difference > 0 ? "+" : "−"}${durationLabel(Math.abs(difference))} on last week`;
};
