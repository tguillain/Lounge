<script lang="ts">
  import { imageUrl } from "$lib/api";
  import { downloadLabel } from "$lib/downloads";
  import { lastPlayedLabel, sizeLabel, storeLabel } from "$lib/format";
  import type { AchievementsResult, Game } from "$lib/types";

  type Props = {
    game: Game;
    running: boolean;
    now: Date;
    error: string | null;
    achievements: AchievementsResult | null;
    isBroken: (url: string) => boolean;
    onBroken: (url: string) => void;
    onPrimary: () => void;
    onAchievements: () => void;
  };

  let { game, running, now, error, achievements, isBroken, onBroken, onPrimary, onAchievements }: Props = $props();

  const logo = $derived(imageUrl(game.images.logo));
  const logoUrl = $derived(logo && !isBroken(logo) ? logo : null);
  const lastPlayed = $derived(lastPlayedLabel(game.lastPlayed, now.getTime() / 1000));
  const size = $derived(sizeLabel(game.sizeOnDisk));
</script>

<section class="hero" aria-label="Selected game">
  <div class="title-slot">
    {#key game.id}
      {#if logoUrl}
        <img class="logo" class:light={game.images.logoIsDark} src={logoUrl} alt={game.name} onerror={() => onBroken(logoUrl)} />
      {:else}
        <h1>{game.name}</h1>
      {/if}
    {/key}
  </div>
  <div class="meta">
    <span class="pill">{storeLabel(game.store)}</span>
    {#if running}
      <span class="live"><span class="live-dot"></span>Now playing</span>
    {:else if game.download}
      <span class="downloading">{downloadLabel(game.download)}</span>
    {:else if lastPlayed}
      <span>Last played {lastPlayed}</span>
    {:else}
      <span>Never played</span>
    {/if}
    {#if size}
      <span class="dot" aria-hidden="true">·</span>
      <span>{size}</span>
    {/if}
    {#if achievements?.status === "ready"}
      <span class="dot" aria-hidden="true">·</span>
      <button
        type="button"
        class="trophies"
        class:complete={achievements.unlocked === achievements.total}
        aria-label="{achievements.unlocked} of {achievements.total} achievements"
        onclick={onAchievements}
      >
        <svg aria-hidden="true" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 21h8M12 17v4M7 4h10v5a5 5 0 0 1-10 0z" /><path d="M17 5h3v2a3 3 0 0 1-3 3M7 5H4v2a3 3 0 0 0 3 3" /></svg>
        {achievements.unlocked}/{achievements.total}
      </button>
    {/if}
  </div>
  <div class="actions">
    <button type="button" class="primary" onclick={onPrimary}>
      {#if running}
        <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14M13 6l6 6-6 6" /></svg>
        Back to game
      {:else}
        <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24"><path d="M6 3.8v16.4a1 1 0 0 0 1.5.9l13.2-8.2a1 1 0 0 0 0-1.8L7.5 2.9A1 1 0 0 0 6 3.8z" fill="currentColor" /></svg>
        Play
      {/if}
    </button>
  </div>
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</section>

<style>
  .hero {
    flex: 0 1 640px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 20px;
  }

  .title-slot {
    height: 180px;
    display: flex;
    align-items: flex-end;
  }

  .logo {
    max-width: 520px;
    max-height: 100%;
    object-fit: contain;
    object-position: left bottom;
    filter: drop-shadow(0 6px 30px rgba(0, 0, 0, 0.45));
    animation: fade-in 0.4s var(--ease);
  }

  .logo.light {
    filter: brightness(0) invert(1) drop-shadow(0 6px 30px rgba(0, 0, 0, 0.45));
  }

  h1 {
    margin: 0;
    font-size: 64px;
    line-height: 1.05;
    font-weight: 700;
    letter-spacing: -0.035em;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    overflow: hidden;
    animation: fade-in 0.4s var(--ease);
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 16px;
    color: #d1d1d6;
  }

  .pill {
    padding: 5px 12px;
    border-radius: 999px;
    background: var(--glass);
    border: 1px solid var(--glass-border);
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
  }

  .dot {
    color: var(--line);
  }

  .trophies {
    padding: 0;
    border: 0;
    background: none;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: inherit;
    color: inherit;
    font-variant-numeric: tabular-nums;
    cursor: pointer;
    animation: fade-in 0.3s var(--ease);
  }

  .trophies.complete {
    color: #ffd60a;
    font-weight: 600;
  }

  .downloading {
    font-weight: 600;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .live {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    color: var(--live-text);
  }

  .live-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: var(--live);
    box-shadow: 0 0 0 4px rgba(48, 209, 88, 0.25);
  }

  .actions {
    display: flex;
    gap: 14px;
    margin-top: 12px;
  }

  .primary {
    height: 58px;
    padding: 0 34px;
    border: 0;
    border-radius: 999px;
    background: var(--text);
    color: var(--bg);
    font-size: 18px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
  }

  .error {
    margin: 0;
    font-size: 15px;
    color: #ff8a80;
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
  }
</style>
