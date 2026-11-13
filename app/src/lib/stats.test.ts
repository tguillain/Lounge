import { describe, expect, it } from "vitest";
import {
  barLabel,
  differenceLabel,
  durationLabel,
  minutesBetween,
  mostPlayed,
  recentMinutes,
  sessionWhen,
  startOfWeek,
  streaks,
  thisWeek,
  weekTotals,
} from "./stats";
import type { Game, PlaySession } from "./types";

const at = (day: number, hour: number, minute = 0): number => new Date(2026, 10, day, hour, minute).getTime() / 1000;

const session = (gameId: string, day: number, hour: number, minutes: number): PlaySession => ({
  gameId,
  start: at(day, hour),
  end: at(day, hour) + minutes * 60,
});

const game = (id: string, playTime: number | null, recentPlayTime: number | null = null): Game => ({
  id,
  store: id.startsWith("epic:") ? "epic" : "steam",
  name: id,
  lastPlayed: 0,
  sizeOnDisk: 0,
  playTime,
  recentPlayTime,
  launchUri: "",
  images: { hero: null, logo: null, logoIsDark: false, header: null, cover: null },
  hidden: false,
  download: null,
});

// Friday 13 November 2026, 21:00
const now = new Date(2026, 10, 13, 21, 0);

describe("weeks", () => {
  it("start on Monday", () => {
    expect(startOfWeek(now)).toEqual(new Date(2026, 10, 9));
    expect(startOfWeek(new Date(2026, 10, 15, 23, 0))).toEqual(new Date(2026, 10, 9));
  });

  it("split a session that crosses midnight between both days", () => {
    const late = session("steam:1", 11, 23, 90);
    const days = thisWeek([late], now);
    expect(days.map((day) => day.minutes)).toEqual([0, 0, 60, 30, 0, 0, 0]);
    expect(days[4]).toMatchObject({ today: true, future: false });
    expect(days[5].future).toBe(true);
  });

  it("add up the last weeks, oldest first", () => {
    const weeks = weekTotals([session("steam:1", 2, 20, 120), session("steam:1", 12, 20, 45)], now, 3);
    expect(weeks.map((week) => week.minutes)).toEqual([0, 120, 45]);
    expect(weeks[2].current).toBe(true);
  });

  it("can count one game only", () => {
    const sessions = [session("steam:1", 12, 20, 45), session("epic:2", 12, 22, 30)];
    expect(minutesBetween(sessions, new Date(2026, 10, 12), new Date(2026, 10, 13), "epic:2")).toBe(30);
  });
});

describe("streaks", () => {
  it("counts the days in a row up to today, and the best run", () => {
    const sessions = [1, 2, 3, 4, 10, 11, 12, 13].map((day) => session("steam:1", day, 20, 30));
    expect(streaks(sessions, now)).toEqual({ current: 4, best: 4 });
  });

  it("keeps a streak alive until the end of today", () => {
    const sessions = [10, 11, 12].map((day) => session("steam:1", day, 20, 30));
    expect(streaks(sessions, now).current).toBe(3);
  });

  it("breaks after a day without play", () => {
    const sessions = [3, 4, 5, 6, 7, 11].map((day) => session("steam:1", day, 20, 30));
    expect(streaks(sessions, now)).toEqual({ current: 0, best: 5 });
    expect(streaks([], now)).toEqual({ current: 0, best: 0 });
  });
});

describe("most played", () => {
  const games = [game("steam:1", 4000, 300), game("steam:2", 9000, 0), game("epic:3", 500, null), game("steam:4", null)];
  const threeWeeksAgo = -10;
  const sessions = [session("epic:3", 12, 20, 400), session("steam:1", threeWeeksAgo, 20, 999)];

  it("ranks the last two weeks with Steam's count or Lounge's, whichever is more", () => {
    expect(recentMinutes(games[2], sessions, now)).toBe(400);
    expect(mostPlayed(games, sessions, now, "recent").map((ranked) => [ranked.game.id, ranked.minutes])).toEqual([
      ["epic:3", 400],
      ["steam:1", 300],
    ]);
  });

  it("ranks all time by total play time", () => {
    expect(mostPlayed(games, sessions, now, "all").map((ranked) => ranked.game.id)).toEqual(["steam:2", "steam:1", "epic:3"]);
  });
});

describe("labels", () => {
  it("write durations the way the stats show them", () => {
    expect(durationLabel(580)).toBe("9 h 40");
    expect(durationLabel(125)).toBe("2 h 05");
    expect(durationLabel(120)).toBe("2 h");
    expect(durationLabel(45)).toBe("45 min");
    expect(durationLabel(96_738)).toBe("1,612 h");
    expect(barLabel(0)).toBe("");
    expect(barLabel(90)).toBe("1.5 h");
    expect(barLabel(120)).toBe("2 h");
  });

  it("compare with last week", () => {
    expect(differenceLabel(580, 435)).toBe("+2 h 25 on last week");
    expect(differenceLabel(30, 90)).toBe("−1 h on last week");
    expect(differenceLabel(0, 0)).toBeNull();
  });

  it("say when a session started", () => {
    expect(sessionWhen(session("steam:1", 13, 20, 30), now)).toMatch(/^Today, 8:00\sPM$/);
    expect(sessionWhen(session("steam:1", 12, 21, 30), now)).toMatch(/^Yesterday, 9:00\sPM$/);
    expect(sessionWhen(session("steam:1", 9, 19, 30), now)).toMatch(/^Monday, 7:00\sPM$/);
    expect(sessionWhen(session("steam:1", 2, 19, 30), now)).toMatch(/^Nov 2, 7:00\sPM$/);
  });
});
