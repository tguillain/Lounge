<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import { isRare, isSecret, progressPercent, rarityLabel, unlockedLabel } from "$lib/achievements";
  import type { Achievement, AchievementsResult, Game } from "$lib/types";

  type Props = {
    game: Game;
    result: AchievementsResult;
    achievements: Achievement[];
    focusedIndex: number;
    revealed: Set<string>;
    onFocus: (index: number) => void;
    onActivate: (achievement: Achievement) => void;
    onRetry: () => void;
  };

  let { game, result, achievements, focusedIndex, revealed, onFocus, onActivate, onRetry }: Props = $props();

  const rows: HTMLButtonElement[] = [];
  const brokenIcons = new SvelteSet<string>();

  $effect(() => {
    rows[focusedIndex]?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  });

  const percent = $derived(result.status === "ready" ? progressPercent(result.unlocked, result.total) : 0);
  const masked = (achievement: Achievement) => isSecret(achievement) && !revealed.has(achievement.id);
</script>

<section class="achievements" aria-label="Achievements">
  <header>
    <div class="title">
      <p class="game">{game.name}</p>
      <h1>Achievements</h1>
    </div>
    {#if result.status === "ready"}
      <div class="progress" aria-label="{result.unlocked} of {result.total} unlocked">
        <p class="count"><strong>{result.unlocked}</strong> of {result.total} · {percent}%</p>
        <div class="bar"><div class="fill" style:width="{percent}%"></div></div>
      </div>
    {/if}
  </header>

  {#if result.status === "loading"}
    <p class="message" aria-live="polite">Loading achievements…</p>
  {:else if result.status === "none"}
    <p class="message">{game.name} has no Steam achievements.</p>
  {:else if result.status === "failed"}
    <div class="message" role="alert">
      <p>{result.message}</p>
      <button type="button" class="retry" onclick={onRetry}>Try again</button>
    </div>
  {:else}
    <div class="scroller">
      <div class="list">
        {#each achievements as achievement, index (achievement.id)}
          {@const hiddenNow = masked(achievement)}
          {@const unlocked = achievement.unlockedAt !== null}
          <button
            type="button"
            class="row"
            class:focused={index === focusedIndex}
            class:locked={!unlocked}
            bind:this={rows[index]}
            onpointerenter={() => onFocus(index)}
            onclick={() => onActivate(achievement)}
          >
            <span class="icon" aria-hidden="true">
              {#if hiddenNow}
                <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="5" y="11" width="14" height="10" rx="2" /><path d="M8 11V8a4 4 0 0 1 8 0v3" /></svg>
              {:else if achievement.icon && !brokenIcons.has(achievement.icon)}
                <img src={achievement.icon} alt="" loading="lazy" onerror={() => achievement.icon && brokenIcons.add(achievement.icon)} />
              {:else}
                <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 21h8M12 17v4M7 4h10v5a5 5 0 0 1-10 0z" /><path d="M17 5h3v2a3 3 0 0 1-3 3M7 5H4v2a3 3 0 0 0 3 3" /></svg>
              {/if}
            </span>
            <span class="text">
              <span class="name">{hiddenNow ? "Hidden achievement" : achievement.name}</span>
              {#if hiddenNow}
                <span class="description">Select to reveal it.</span>
              {:else if achievement.description}
                <span class="description">{achievement.description}</span>
              {/if}
            </span>
            <span class="meta">
              {#if unlocked}
                <span class="when">{unlockedLabel(achievement.unlockedAt ?? 0)}</span>
              {/if}
              {#if isRare(achievement)}
                <span class="rare">Rare · {rarityLabel(achievement.rarity)}</span>
              {:else if rarityLabel(achievement.rarity)}
                <span class="rarity">{rarityLabel(achievement.rarity)}</span>
              {/if}
            </span>
          </button>
        {/each}
      </div>
      {#if !result.complete}
        <p class="note">
          Steam only keeps {achievements.length} of these on this PC. Add your Steam Web API key in Settings to see all of them.
        </p>
      {/if}
    </div>
  {/if}
</section>

<style>
  .achievements {
    flex: 1;
    min-height: 0;
    width: 100%;
    max-width: 900px;
    align-self: center;
    margin-top: 36px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  header {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 32px;
  }

  .title {
    min-width: 0;
  }

  .game {
    margin: 0 0 4px;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  h1 {
    margin: 0;
    font-size: 44px;
    font-weight: 700;
    letter-spacing: -0.03em;
  }

  .progress {
    flex: 0 0 260px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-bottom: 6px;
  }

  .count {
    margin: 0;
    text-align: right;
    font-size: 15px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .count strong {
    font-size: 22px;
    color: var(--text);
  }

  .bar {
    height: 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.12);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    border-radius: inherit;
    background: var(--text);
    transition: width 0.4s var(--ease);
  }

  .message {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    margin: 0;
    font-size: 17px;
    color: var(--text-tertiary);
    text-align: center;
  }

  .message p {
    margin: 0;
    max-width: 520px;
  }

  .retry {
    height: 48px;
    padding: 0 28px;
    border-radius: 999px;
    background: var(--glass);
    border: 1px solid var(--glass-border);
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
  }

  .scroller {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    margin: 0 -16px;
    padding: 4px 16px 48px;
    scrollbar-width: none;
  }

  .scroller::-webkit-scrollbar {
    display: none;
  }

  .list {
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }

  .row {
    width: 100%;
    min-height: 84px;
    padding: 12px 20px 12px 14px;
    border: 0;
    background: transparent;
    display: flex;
    align-items: center;
    gap: 18px;
    text-align: left;
    cursor: pointer;
    scroll-margin: 24px;
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }

  .row + .row {
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }

  .row:focus-visible {
    outline: none;
  }

  .row.focused {
    background: var(--text);
    color: var(--bg);
  }

  .row.focused + .row {
    border-top-color: transparent;
  }

  .icon {
    flex-shrink: 0;
    width: 60px;
    height: 60px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.08);
    display: grid;
    place-items: center;
    overflow: hidden;
    color: var(--text-tertiary);
  }

  .icon img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .row.locked .icon img {
    filter: grayscale(1);
    opacity: 0.45;
  }

  .row.focused .icon {
    background: rgba(10, 10, 12, 0.08);
    color: rgba(10, 10, 12, 0.55);
  }

  .row.focused.locked .icon img {
    opacity: 0.6;
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .name {
    font-size: 17px;
    font-weight: 600;
  }

  .row.locked .name {
    color: var(--text-secondary);
  }

  .row.focused .name {
    color: var(--bg);
  }

  .description {
    font-size: 14px;
    line-height: 1.35;
    color: var(--text-tertiary);
  }

  .row.focused .description {
    color: rgba(10, 10, 12, 0.6);
  }

  .meta {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
    font-size: 13px;
    color: var(--text-tertiary);
    text-align: right;
  }

  .row.focused .meta {
    color: rgba(10, 10, 12, 0.6);
  }

  .when {
    font-weight: 600;
    color: var(--live-text);
  }

  .row.focused .when {
    color: #1f8a3c;
  }

  .rare {
    font-weight: 600;
    color: #ffd60a;
  }

  .row.focused .rare {
    color: #9a6b00;
  }

  .note {
    margin: 20px 4px 0;
    font-size: 14px;
    line-height: 1.4;
    color: var(--text-tertiary);
  }
</style>
