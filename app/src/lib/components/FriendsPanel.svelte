<script lang="ts">
  import { placeholderArt } from "$lib/format";
  import { NETWORK_NAMES, isOnline, statusLine, type FriendsProblem } from "$lib/friends";
  import type { Friend, FriendNetwork } from "$lib/types";

  type Props = {
    friends: Friend[];
    pinned: ReadonlySet<string>;
    loading: boolean;
    needsSetup: boolean;
    problems: FriendsProblem[];
    focusedIndex: number | null;
    panelFocused: boolean;
    onConnect: (network: FriendNetwork) => void;
    onRetry: () => void;
    onJoin: (friend: Friend) => void;
    onTogglePin: (friend: Friend) => void;
  };

  let { friends, pinned, loading, needsSetup, problems, focusedIndex, panelFocused, onConnect, onRetry, onJoin, onTogglePin }: Props =
    $props();

  const onlineCount = $derived(friends.filter(isOnline).length);
</script>

<aside class="panel" class:focused={panelFocused} aria-label="Friends">
  <div class="heading">
    <h2>Friends</h2>
    {#if !loading && !needsSetup}
      <span class="count">{onlineCount === 0 ? "Nobody online" : `${onlineCount} online`}</span>
    {/if}
  </div>

  {#if loading}
    <div class="skeleton" aria-busy="true" aria-label="Loading friends">
      {#each [0, 1, 2] as row (row)}
        <span class="skeleton-row"></span>
      {/each}
    </div>
  {:else if needsSetup}
    <p class="message">See which Steam and PlayStation friends are online, and jump into their Steam games.</p>
    <div class="buttons">
      <button type="button" class="action" onclick={() => onConnect("steam")}>Connect Steam</button>
      <button type="button" class="action ghost" onclick={() => onConnect("psn")}>Connect PlayStation</button>
    </div>
  {:else}
    {#if friends.length === 0 && problems.length === 0}
      <p class="message">None of your friends are online right now.</p>
    {/if}
    {#if friends.length > 0}
      <ul>
        {#each friends as friend, index (friend.id)}
          <li
            class:focused={index === focusedIndex}
            class:offline={!isOnline(friend)}
            oncontextmenu={(event) => {
              event.preventDefault();
              onTogglePin(friend);
            }}
          >
            <span class="avatar" style:background={friend.avatar ? undefined : placeholderArt(friend.name)}>
              {#if friend.avatar}
                <img src={friend.avatar} alt="" />
              {:else}
                {friend.name.charAt(0)}
              {/if}
              <span class="presence {friend.presence}" aria-hidden="true"></span>
            </span>
            <span class="who">
              <span class="name">
                <span class="label">{friend.name}</span>
                {#if friend.network === "psn"}
                  <span class="network" title="PlayStation Network">PS</span>
                {/if}
                {#if pinned.has(friend.id)}
                  <svg class="pin" role="img" aria-label="Pinned" width="13" height="13" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M16 3l5 5-2.5 1-3.5 3.5.5 5-1.5 1.5-4-4-5 5-1.5-1.5 5-5-4-4L6 8l5 .5L14.5 5z" />
                  </svg>
                {/if}
              </span>
              <span class="status {friend.presence}">{statusLine(friend)}</span>
            </span>
            {#if friend.joinUri}
              <button type="button" class="join" aria-label={`Join ${friend.name}`} onclick={() => onJoin(friend)}>Join</button>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
    {#each problems as problem (problem.network)}
      <div class="problem" role="alert">
        <p><strong>{NETWORK_NAMES[problem.network]}</strong> · {problem.message}</p>
        <div class="buttons">
          <button type="button" class="small" onclick={onRetry}>Try again</button>
          <button type="button" class="small ghost" onclick={() => onConnect(problem.network)}>
            {problem.network === "steam" ? "Change key" : "Sign in again"}
          </button>
        </div>
      </div>
    {/each}
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

  .message {
    margin: 0;
    font-size: 14px;
    line-height: 1.45;
    color: var(--text-secondary);
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
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
  }

  .action.ghost,
  .small.ghost {
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

  li.offline .avatar {
    opacity: 0.5;
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

  .presence.offline {
    display: none;
  }

  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .name {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    font-size: 15px;
    font-weight: 600;
  }

  .label {
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .network {
    flex-shrink: 0;
    padding: 1px 5px;
    border-radius: 5px;
    background: rgba(0, 112, 209, 0.35);
    color: #9cc8ff;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.04em;
  }

  .pin {
    flex-shrink: 0;
    color: #ffd60a;
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

  .problem {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border-radius: 14px;
    background: rgba(255, 69, 58, 0.1);
    border: 1px solid rgba(255, 69, 58, 0.25);
  }

  .problem p {
    margin: 0;
    font-size: 13px;
    line-height: 1.4;
    color: var(--text-secondary);
  }

  .problem strong {
    color: var(--text);
  }

  .small {
    height: 32px;
    padding: 0 14px;
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
