import type { Friend, FriendNetwork, FriendsResult } from "./types";

const PRESENCE_RANK: Record<Friend["presence"], number> = { "in-game": 0, online: 1, away: 2, offline: 3 };

export const NETWORK_NAMES: Record<FriendNetwork, string> = { steam: "Steam", psn: "PlayStation" };

export const isOnline = (friend: Friend): boolean => friend.presence !== "offline";

export const readyFriends = (...results: FriendsResult[]): Friend[] =>
  results.flatMap((result) => (result.status === "ready" ? result.friends : []));

/** Pinned friends first, online or not, then everyone online: playing, online, away. */
export const homeFriends = (friends: Friend[], pinned: ReadonlySet<string>): Friend[] =>
  friends
    .filter((friend) => isOnline(friend) || pinned.has(friend.id))
    .sort(
      (a, b) =>
        Number(pinned.has(b.id)) - Number(pinned.has(a.id)) ||
        PRESENCE_RANK[a.presence] - PRESENCE_RANK[b.presence] ||
        Number(b.joinUri !== null) - Number(a.joinUri !== null) ||
        a.name.localeCompare(b.name, undefined, { sensitivity: "base" }),
    );

export const statusLine = (friend: Friend): string => {
  const platform = friend.platform ? ` on ${friend.platform}` : "";
  switch (friend.presence) {
    case "in-game":
      return friend.game ? `Playing ${friend.game}${friend.platform ? ` · ${friend.platform}` : ""}` : `In a game${platform}`;
    case "online":
      return `Online${platform}`;
    case "away":
      return "Away";
    case "offline":
      return "Offline";
  }
};

export type FriendsProblem = { network: FriendNetwork; message: string };

export const friendsProblems = (results: Record<FriendNetwork, FriendsResult>): FriendsProblem[] =>
  (Object.entries(results) as [FriendNetwork, FriendsResult][]).flatMap(([network, result]) =>
    result.status === "failed" ? [{ network, message: result.message }] : [],
  );

export const nothingConnected = (results: Record<FriendNetwork, FriendsResult>): boolean =>
  Object.values(results).every((result) => result.status === "not-configured");

export const stillLoading = (results: Record<FriendNetwork, FriendsResult>): boolean =>
  Object.values(results).some((result) => result.status === "loading") &&
  Object.values(results).every((result) => result.status === "loading" || result.status === "not-configured");
