<script lang="ts">
  import { SvelteSet } from "svelte/reactivity";
  import { progressPercent, rarityPercent, sessionHighlights } from "$lib/achievements";
  import type { Achievement } from "$lib/types";

  type Props = {
    unlocked: number;
    total: number;
    achievements: Achievement[];
    startedAt: number;
  };

  let { unlocked, total, achievements, startedAt }: Props = $props();

  const brokenIcons = new SvelteSet<string>();
  const percent = $derived(progressPercent(unlocked, total));
  const highlights = $derived(sessionHighlights(achievements, startedAt));
  const showingSession = $derived(highlights.unlockedThisSession.length > 0);
  const rows = $derived(showingSession ? highlights.unlockedThisSession : highlights.upNext);
</script>

<section class="achievements" aria-label="Achievements">
  <div class="head">
    <span class="title">Achievements</span>
    <span class="count"><strong>{unlocked}</strong> of {total}</span>
  </div>
  <div class="bar" aria-hidden="true"><div class="fill" style:width="{percent}%"></div></div>

  {#if rows.length > 0}
    <p class="subtitle">
      {#if showingSession}
        Unlocked this session
      {:else}
        Closest to unlock
      {/if}
    </p>
    <ul>
      {#each rows as achievement (achievement.id)}
        <li class:locked={achievement.unlockedAt === null}>
          <span class="icon" aria-hidden="true">
            {#if achievement.icon && !brokenIcons.has(achievement.icon)}
              <img src={achievement.icon} alt="" onerror={() => achievement.icon && brokenIcons.add(achievement.icon)} />
            {:else}
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 21h8M12 17v4M7 4h10v5a5 5 0 0 1-10 0z" /><path d="M17 5h3v2a3 3 0 0 1-3 3M7 5H4v2a3 3 0 0 0 3 3" /></svg>
            {/if}
          </span>
          <span class="text">
            <span class="name">{achievement.name}</span>
            {#if achievement.description}
              <span class="description">{achievement.description}</span>
            {/if}
          </span>
          {#if !showingSession && achievement.rarity !== null}
            <span class="rarity">{rarityPercent(achievement.rarity)}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {:else if unlocked === total}
    <p class="subtitle done">Every achievement unlocked.</p>
  {/if}
</section>

<style>
  .achievements {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
    border-radius: 20px;
    background: rgba(255, 255, 255, 0.06);
  }

  .head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .title {
    font-size: 15px;
    font-weight: 600;
  }

  .count {
    font-size: 13px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .count strong {
    font-size: 15px;
    color: var(--text);
  }

  .bar {
    height: 5px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.12);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    border-radius: inherit;
    background: var(--text);
  }

  .subtitle {
    margin: 6px 0 0;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  .subtitle.done {
    color: #ffd60a;
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  li {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .icon {
    flex: 0 0 auto;
    width: 40px;
    height: 40px;
    border-radius: 9px;
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

  li.locked .icon img {
    filter: grayscale(1);
    opacity: 0.5;
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .name {
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .description {
    font-size: 12px;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .rarity {
    flex: 0 0 auto;
    font-size: 12px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }
</style>
