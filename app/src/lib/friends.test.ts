import { describe, expect, it } from "vitest";
import { friendsProblems, homeFriends, nothingConnected, readyFriends, statusLine, stillLoading } from "./friends";
import type { Friend } from "./types";

const friend = (id: string, presence: Friend["presence"], extra: Partial<Friend> = {}): Friend => ({
  id,
  network: id.startsWith("psn:") ? "psn" : "steam",
  name: id.split(":")[1],
  avatar: null,
  presence,
  game: null,
  platform: null,
  joinUri: null,
  ...extra,
});

describe("homeFriends", () => {
  const friends = [
    friend("steam:Zoe", "online"),
    friend("psn:Lou", "offline"),
    friend("steam:Bastien", "offline"),
    friend("psn:Kaori", "in-game", { game: "Astro Bot" }),
    friend("steam:Mathis", "in-game", { game: "Rocket League", joinUri: "steam://joinlobby/1/2/3" }),
    friend("steam:Nyx", "away"),
  ];

  it("lists everyone online from both networks, playing first and joinable games on top", () => {
    expect(homeFriends(friends, new Set()).map((found) => found.id)).toEqual([
      "steam:Mathis",
      "psn:Kaori",
      "steam:Zoe",
      "steam:Nyx",
    ]);
  });

  it("puts pinned friends first and keeps them even offline", () => {
    expect(homeFriends(friends, new Set(["psn:Lou", "steam:Nyx"])).map((found) => found.id)).toEqual([
      "steam:Nyx",
      "psn:Lou",
      "steam:Mathis",
      "psn:Kaori",
      "steam:Zoe",
    ]);
  });
});

describe("statusLine", () => {
  it("says what and where a friend is playing", () => {
    expect(statusLine(friend("psn:Kaori", "in-game", { game: "Astro Bot", platform: "PS5" }))).toBe("Playing Astro Bot · PS5");
    expect(statusLine(friend("steam:Mathis", "in-game", { game: "Rocket League" }))).toBe("Playing Rocket League");
    expect(statusLine(friend("psn:Jade", "online", { platform: "PS4" }))).toBe("Online on PS4");
    expect(statusLine(friend("steam:Nyx", "away"))).toBe("Away");
    expect(statusLine(friend("psn:Lou", "offline"))).toBe("Offline");
  });
});

describe("connection states", () => {
  it("merges the friends that loaded and reports the networks that failed", () => {
    const steam = { status: "ready" as const, friends: [friend("steam:Zoe", "online")] };
    const psn = { status: "failed" as const, message: "Your PlayStation sign-in has expired." };
    expect(readyFriends(steam, psn)).toHaveLength(1);
    expect(friendsProblems({ steam, psn })).toEqual([{ network: "psn", message: "Your PlayStation sign-in has expired." }]);
  });

  it("knows when nothing is connected and when the panel is still loading", () => {
    const off = { status: "not-configured" as const };
    const loading = { status: "loading" as const };
    expect(nothingConnected({ steam: off, psn: off })).toBe(true);
    expect(stillLoading({ steam: loading, psn: off })).toBe(true);
    expect(stillLoading({ steam: loading, psn: { status: "ready", friends: [] } })).toBe(false);
  });
});
