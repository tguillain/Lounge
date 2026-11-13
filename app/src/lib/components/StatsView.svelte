<script lang="ts">
  import { imageUrl } from "$lib/api";
  import { placeholderArt } from "$lib/format";
  import {
    barLabel,
    differenceLabel,
    durationLabel,
    mostPlayed,
    streaks,
    thisWeek,
    totalMinutes,
    weekLabel,
    weekTotals,
    type StatsPeriod,
  } from "$lib/stats";
  import type { Game, PlaySession } from "$lib/types";

  type Props = {
    games: Game[];
    sessions: PlaySession[] | null;
    now: Date;
    period: StatsPeriod;
    focusedIndex: number | null;
    isBroken: (url: string) => boolean;
    onBroken: (url: string) => void;
    onFocus: (index: number) => void;
    onOpen: (game: Game) => void;
    onPeriod: (period: StatsPeriod) => void;
  };

  let { games, sessions, now, period, focusedIndex, isBroken, onBroken, onFocus, onOpen, onPeriod }: Props = $props();

  const history = $derived(sessions ?? []);
  const counting = $derived(history.length > 0);
  const days = $derived(thisWeek(history, now));
  const weeks = $derived(weekTotals(history, now, 12));
  const weekMinutes = $derived(weeks.at(-1)?.minutes ?? 0);
  const lastWeekMinutes = $derived(weeks.at(-2)?.minutes ?? 0);
  const difference = $derived(differenceLabel(weekMinutes, lastWeekMinutes));
  const streak = $derived(streaks(history, now));
  const ranked = $derived(mostPlayed(games, history, now, period));
  const topMinutes = $derived(ranked[0]?.minutes ?? 1);
  const busiestDay = $derived(Math.max(60, ...days.map((day) => day.minutes)));
  const busiestWeek = $derived(Math.max(60, ...weeks.map((week) => week.minutes)));
  const playedWeeks = $derived(weeks.filter((week) => week.minutes > 0));
  const average = $derived(
    playedWeeks.length > 0 ? Math.round(playedWeeks.reduce((sum, week) => sum + week.minutes, 0) / playedWeeks.length) : 0,
  );
  const total = $derived(totalMinutes(games));
  const playedGames = $derived(games.filter((game) => (game.playTime ?? 0) > 0).length);

  const rows: HTMLButtonElement[] = [];

  $effect(() => {
    if (focusedIndex !== null) {
      rows[focusedIndex]?.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  });

  const artOf = (game: Game): string | null => {
    const url = imageUrl(game.images.header ?? game.images.cover);
    return url && !isBroken(url) ? url : null;
  };
</script>

<section class="stats" aria-label="Stats">
  <div class="title">
    <h1>Stats</h1>
    {#if total > 0}
      <span class="count">{durationLabel(total)} across {playedGames} {playedGames === 1 ? "game" : "games"}</span>
    {/if}
  </div>

  <div class="grid">
    <section class="card week" aria-label="This week">
      <div class="week-summary">
        <span class="label">This week</span>
        <span class="big">{durationLabel(weekMinutes)}</span>
        {#if difference}
          <span class="difference" class:down={weekMinutes < lastWeekMinutes}>{difference}</span>
        {/if}
        {#if streak.current > 0 || streak.best > 0}
          <span class="streak">
            <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2c1 4 5 5.5 5 11a5 5 0 0 1-10 0c0-2.5 1.2-4 2.5-5 .3 2 1.5 3 2.5 3-1-3 0-6 0-9z" /></svg>
            <span>
              <strong>{streak.current === 0 ? "No streak today" : `${streak.current}-day streak`}</strong>
              <small>Best: {streak.best} {streak.best === 1 ? "day" : "days"}</small>
            </span>
          </span>
        {/if}
      </div>
      {#if counting}
        <div class="days" role="img" aria-label="Play time per day this week">
          {#each days as day, index (index)}
            <span class="day" class:today={day.today} class:future={day.future}>
              <span class="value">{day.future ? "" : barLabel(day.minutes)}</span>
              <span class="bar" style:height="{day.minutes === 0 ? 4 : Math.max(8, Math.round((day.minutes / busiestDay) * 150))}px"></span>
              <span class="letter">{day.label}</span>
            </span>
          {/each}
        </div>
      {:else}
        <p class="empty">Lounge counts your sessions from now on. Play a game and come back: this week, your streak and the weekly chart fill in as you go.</p>
      {/if}
    </section>

    <section class="card top" aria-label="Most played">
      <div class="card-heading">
        <h2>Most played</h2>
        <div class="periods" role="group" aria-label="Period">
          <button type="button" class:active={period === "recent"} aria-pressed={period === "recent"} onclick={() => onPeriod("recent")}>Last 2 weeks</button>
          <button type="button" class:active={period === "all"} aria-pressed={period === "all"} onclick={() => onPeriod("all")}>All time</button>
        </div>
      </div>
      {#if ranked.length === 0}
        <p class="empty">{period === "recent" ? "Nothing played in the last two weeks." : "No play time yet."}</p>
      {:else}
        <ol>
          {#each ranked as entry, index (entry.game.id)}
            {@const art = artOf(entry.game)}
            <li>
              <button
                type="button"
                class="game"
                class:focused={index === focusedIndex}
                bind:this={rows[index]}
                onpointerenter={() => onFocus(index)}
                onclick={() => onOpen(entry.game)}
              >
                <span class="rank">{index + 1}</span>
                <span class="art" style:background={art ? undefined : placeholderArt(entry.game.name)}>
                  {#if art}
                    <img src={art} alt="" loading="lazy" onerror={() => onBroken(art)} />
                  {/if}
                </span>
                <span class="game-text">
                  <span class="game-line"><span class="name">{entry.game.name}</span><span class="minutes">{durationLabel(entry.minutes)}</span></span>
                  <span class="track"><span class="fill" style:width="{Math.round((entry.minutes / topMinutes) * 100)}%"></span></span>
                </span>
              </button>
            </li>
          {/each}
        </ol>
      {/if}
    </section>
  </div>

  <section class="card weeks" aria-label="Last 12 weeks">
    <div class="card-heading">
      <h2>Last 12 weeks</h2>
      {#if average > 0}
        <span class="note">{durationLabel(average)} a week on average</span>
      {/if}
    </div>
    {#if counting}
      <div class="week-bars" role="img" aria-label="Play time per week over the last 12 weeks">
        {#each weeks as week, index (index)}
          <span class="week-bar" class:current={week.current}>
            <span class="value">{barLabel(week.minutes)}</span>
            <span class="bar" style:height="{week.minutes === 0 ? 4 : Math.max(8, Math.round((week.minutes / busiestWeek) * 96))}px"></span>
            <span class="letter">{index % 4 === 0 || week.current ? weekLabel(week) : ""}</span>
          </span>
        {/each}
      </div>
    {:else}
      <p class="empty">Your weeks appear here once Lounge has seen you play.</p>
    {/if}
  </section>
</section>

<style>
  .stats {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 20px;
    margin-top: 32px;
    overflow-y: auto;
    padding-bottom: 24px;
    scrollbar-width: none;
    animation: rise 0.3s var(--ease);
  }

  .stats::-webkit-scrollbar {
    display: none;
  }

  .title {
    display: flex;
    align-items: baseline;
    gap: 16px;
  }

  h1 {
    margin: 0;
    font-size: 48px;
    font-weight: 700;
    letter-spacing: -0.03em;
  }

  .count {
    font-size: 17px;
    color: var(--text-tertiary);
  }

  .grid {
    display: grid;
    grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr);
    gap: 20px;
  }

  @media (max-width: 1100px) {
    .grid {
      grid-template-columns: minmax(0, 1fr);
    }
  }

  .card {
    padding: 24px;
    border-radius: 22px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .week {
    display: flex;
    gap: 32px;
  }

  .week-summary {
    flex: 0 0 210px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .label {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-tertiary);
  }

  .big {
    font-size: 44px;
    font-weight: 700;
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
  }

  .difference {
    font-size: 14px;
    font-weight: 600;
    color: var(--live-text);
  }

  .difference.down {
    color: var(--text-tertiary);
  }

  .streak {
    margin-top: auto;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-radius: 14px;
    background: rgba(255, 159, 10, 0.12);
    border: 1px solid rgba(255, 159, 10, 0.3);
    color: #ff9f0a;
  }

  .streak span {
    display: flex;
    flex-direction: column;
    color: var(--text);
  }

  .streak strong {
    font-size: 15px;
  }

  .streak small {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .days {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: flex-end;
    gap: 12px;
    min-height: 210px;
  }

  .day,
  .week-bar {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .value {
    font-size: 12px;
    color: var(--text-tertiary);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .bar {
    width: 100%;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.28);
  }

  .day.today .bar,
  .week-bar.current .bar {
    background: var(--text);
  }

  .day.future .bar {
    background: rgba(255, 255, 255, 0.08);
  }

  .letter {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-tertiary);
    white-space: nowrap;
  }

  .day.today .letter {
    color: var(--text);
  }

  .empty {
    margin: auto 0;
    max-width: 440px;
    font-size: 15px;
    line-height: 1.45;
    color: var(--text-secondary);
  }

  .top {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .card-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  h2 {
    margin: 0;
    font-size: 17px;
    font-weight: 600;
  }

  .note {
    font-size: 13px;
    color: var(--text-tertiary);
  }

  .periods {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
  }

  .periods button {
    padding: 6px 12px;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .periods button.active {
    background: var(--text);
    color: var(--bg);
    font-weight: 600;
  }

  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .game {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 6px 10px;
    margin: 0 -10px;
    width: calc(100% + 20px);
    border: 0;
    border-radius: 14px;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .game.focused {
    background: rgba(255, 255, 255, 0.12);
    box-shadow: inset 0 0 0 2px var(--text);
  }

  .game:focus-visible {
    outline: none;
  }

  .rank {
    width: 20px;
    font-size: 15px;
    font-weight: 700;
    color: var(--text-tertiary);
    text-align: right;
  }

  .art {
    flex: 0 0 auto;
    width: 72px;
    height: 34px;
    border-radius: 8px;
    overflow: hidden;
  }

  .art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .game-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .game-line {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 15px;
    font-weight: 600;
  }

  .name {
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .minutes {
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }

  .track {
    height: 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.1);
    overflow: hidden;
  }

  .fill {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: rgba(255, 255, 255, 0.45);
  }

  li:first-child .fill {
    background: var(--text);
  }

  .weeks {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .week-bars {
    display: flex;
    align-items: flex-end;
    gap: 10px;
    min-height: 140px;
  }

  .week-bar .bar {
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.22);
  }

  .week-bar .letter {
    font-size: 11px;
    font-weight: 500;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(24px);
    }
  }
</style>
