<script lang="ts">
  import { placeholderArt } from "$lib/format";
  import type { Friend, FriendsResult } from "$lib/types";

  const MAX_VISIBLE = 5;

  type Props = {
    result: FriendsResult;
    focusedIndex: number | null;
    panelFocused: boolean;
    onConnect: () => void;
    onRetry: () => void;
    onJoin: (friend: Friend) => void;
  };

  let { result, focusedIndex, panelFocused, onConnect, onRetry, onJoin }: Props = $props();

  const friends = $derived(result.status === "ready" ? result.friends.slice(0, MAX_VISIBLE) : []);
  const onlineCount = $derived(result.status === "ready" ? result.friends.length : 0);

  const statusLine = (friend: Friend): string => {
    if (friend.presence === "in-game") {
      return friend.game ? `Playing ${friend.game}` : "In a game";
    }
    return friend.presence === "online" ? "Online" : "Away";
  };
</script>

<aside class="panel" class:focused={panelFocused} aria-label="Friends">
  <div class="heading">
    <h2>Friends</h2>
    {#if result.status === "ready"}
      <span class="count">{onlineCount === 0 ? "Nobody online" : `${onlineCount} online`}</span>
      <button type="button" class="manage" aria-label="Manage Steam connection" onclick={onConnect}>
        <svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="5" cy="12" r="1.6" /><circle cx="12" cy="12" r="1.6" /><circle cx="19" cy="12" r="1.6" /></svg>
      </button>
    {/if}
  </div>

  {#if result.status === "loading"}
    <div class="skeleton" aria-busy="true" aria-label="Loading friends">
      {#each [0, 1, 2] as row (row)}
        <span class="skeleton-row"></span>
      {/each}
    </div>
  {:else if result.status === "not-configured"}
    <p class="message">See which friends are online and jump into their games.</p>
    <button type="button" class="action" onclick={onConnect}>Connect Steam</button>
  {:else if result.status === "failed"}
    <p class="message" role="alert">{result.message}</p>
    <div class="buttons">
      <button type="button" class="action" onclick={onRetry}>Try again</button>
      <button type="button" class="action ghost" onclick={onConnect}>Change key</button>
    </div>
  {:else if friends.length === 0}
    <p class="message">None of your friends are online right now.</p>
  {:else}
    <ul>
      {#each friends as friend, index (friend.steamId)}
        <li class:focused={index === focusedIndex}>
          <span class="avatar" style:background={friend.avatar ? undefined : placeholderArt(friend.name)}>
            {#if friend.avatar}
              <img src={friend.avatar} alt="" />
            {:else}
              {friend.name.charAt(0)}
            {/if}
            <span class="presence {friend.presence}" aria-hidden="true"></span>
          </span>
          <span class="who">
            <span class="name">{friend.name}</span>
            <span class="status {friend.presence}">{statusLine(friend)}</span>
          </span>
          {#if friend.joinUri}
            <button type="button" class="join" aria-label={`Join ${friend.name}`} onclick={() => onJoin(friend)}>Join</button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</aside>

<style>
  .panel {
    flex: 0 0 340px;
    padding: 20px;
    border-radius: 24px;
    background: rgba(20, 20, 24, 0.72);
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
    gap: 14px;
    transition: box-shadow 0.2s ease;
  }

  .panel.focused {
    box-shadow:
      0 0 0 3px var(--bg),
      0 0 0 6px var(--text),
      0 24px 60px rgba(0, 0, 0, 0.4);
  }

  @media (max-width: 1180px) {
    .panel {
      display: none;
    }
  }

  .heading {
    display: flex;
    align-items: center;
  }

  h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 600;
  }

  .count {
    margin-left: auto;
    font-size: 14px;
    color: var(--text-tertiary);
  }

  .manage {
    width: 32px;
    height: 32px;
    margin: -6px -6px -6px 6px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--text-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .manage:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }

  .message {
    margin: 0;
    font-size: 14px;
    line-height: 1.45;
    color: var(--text-secondary);
  }

  .buttons {
    display: flex;
    gap: 8px;
  }

  .action {
    height: 44px;
    padding: 0 18px;
    border: 0;
    border-radius: 999px;
    background: var(--text);
    color: var(--bg);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    align-self: flex-start;
  }

  .action.ghost {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  li {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 56px;
    padding: 6px 8px;
    margin: 0 -8px;
    border-radius: 14px;
    transition: background 0.15s ease;
  }

  li.focused {
    background: rgba(255, 255, 255, 0.12);
    box-shadow: inset 0 0 0 2px var(--text);
  }

  .avatar {
    position: relative;
    flex: 0 0 auto;
    width: 40px;
    height: 40px;
    border-radius: 999px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 15px;
    font-weight: 700;
  }

  .avatar img {
    width: 100%;
    height: 100%;
    border-radius: 999px;
    object-fit: cover;
  }

  .presence {
    position: absolute;
    right: -2px;
    bottom: -2px;
    width: 13px;
    height: 13px;
    border-radius: 999px;
    border: 2.5px solid #1c1c20;
    background: var(--line);
  }

  .presence.in-game {
    background: var(--live);
  }

  .presence.online {
    background: #64d2ff;
  }

  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .name {
    font-size: 15px;
    font-weight: 600;
  }

  .status {
    font-size: 13px;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status.in-game {
    color: var(--live-text);
  }

  .join {
    height: 40px;
    padding: 0 16px;
    border: 0;
    border-radius: 999px;
    background: var(--text);
    color: var(--bg);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .skeleton {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .skeleton-row {
    height: 44px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.06);
    animation: pulse 1.4s ease-in-out infinite;
  }

  @keyframes pulse {
    50% {
      opacity: 0.5;
    }
  }
</style>
