<script lang="ts">
  import { onMount } from "svelte";
  import BatteryIndicator from "$lib/components/BatteryIndicator.svelte";
  import SessionAchievements from "$lib/components/SessionAchievements.svelte";
  import Hints, { type Hint } from "$lib/components/Hints.svelte";
  import {
    getAudioOutputs,
    getPreferences,
    getSession,
    getVolume,
    imageUrl,
    loadAchievements,
    onControllerAction,
    openLounge,
    openLoungeSettings,
    quitGame,
    resumeGame,
    runPower,
    setAudioOutput,
    setMuted,
    setVolume,
  } from "$lib/api";
  import { currentOutputLabel, markCurrent, outputTitle } from "$lib/audio";
  import { clockLabel, dateLabel, placeholderArt, playTimeLabel, totalPlayTimeLabel } from "$lib/format";
  import type { PadAction } from "$lib/input/gamepad-mapping";
  import { confirmationOf } from "$lib/power";
  import { FIRST_SPOT, QUICK_ROWS, itemAt, moveSpot, spotOf, stepVolume, type QuickItem, type Spot } from "$lib/quick-menu";
  import { playSound, type Sound } from "$lib/sounds";
  import type { AchievementsResult, AudioOutput, Session, Volume } from "$lib/types";

  type SheetEntry = { id: string; label: string; value?: string; danger?: boolean; run: () => void };
  type Sheet =
    | { kind: "outputs" }
    | { kind: "confirm"; title: string; note: string; action: string; back: QuickItem; run: () => Promise<void> };

  const CLOCK_REFRESH_MS = 15_000;
  const KEY_ACTIONS: Record<string, PadAction> = {
    ArrowUp: "up",
    ArrowDown: "down",
    ArrowLeft: "left",
    ArrowRight: "right",
    Enter: "confirm",
    Escape: "back",
  };
  const SOUNDS: Partial<Record<PadAction, Sound>> = { up: "move", down: "move", left: "move", right: "move", confirm: "select", back: "back" };

  let session = $state<Session | null>(null);
  let now = $state(new Date());
  let spot = $state<Spot>(FIRST_SPOT);
  let sheet = $state<Sheet | null>(null);
  let sheetIndex = $state(0);
  let outputs = $state<AudioOutput[] | null>(null);
  let volume = $state<Volume | null>(null);
  let achievements = $state<AchievementsResult | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let artBroken = $state(false);
  let soundsOn = true;

  const focused = $derived(sheet ? null : itemAt(spot));
  const gameName = $derived(session?.game?.name ?? "your game");
  const timeLine = $derived.by(() => {
    if (!session) {
      return null;
    }
    const played = playTimeLabel(session.startedAt, now.getTime());
    const total = session.game?.playTime ? totalPlayTimeLabel(session.game.playTime) : null;
    return total ? `${played} · ${total} in total` : played;
  });
  const artUrl = $derived(artBroken ? null : imageUrl(session?.game?.images.hero ?? session?.game?.images.header ?? null));
  const outputLabel = $derived(currentOutputLabel(outputs) ?? "…");
  const currentOutput = $derived(outputs?.find((output) => output.current) ?? null);
  const soundOn = $derived(volume ? !volume.muted : true);

  const run = async (task: () => Promise<void>) => {
    if (busy) {
      return;
    }
    busy = true;
    error = null;
    try {
      await task();
    } catch (failure) {
      error = String(failure);
    } finally {
      busy = false;
    }
  };

  const openSheet = (next: Sheet, index = 0) => {
    sheet = next;
    sheetIndex = index;
    error = null;
  };

  const closeSheet = () => {
    if (sheet?.kind === "confirm") {
      spot = spotOf(sheet.back);
    } else if (sheet?.kind === "outputs") {
      spot = spotOf("output");
    }
    sheet = null;
  };

  const confirmPower = (choice: "restart" | "shut-down") => {
    const confirmation = confirmationOf(choice);
    if (confirmation) {
      openSheet({ kind: "confirm", ...confirmation, back: choice, run: () => runPower(choice) }, 1);
    }
  };

  const pickOutput = (output: AudioOutput) =>
    run(async () => {
      if (output.current || !outputs) {
        return;
      }
      await setAudioOutput(output);
      outputs = markCurrent(outputs, output.id);
      volume = await getVolume().catch(() => volume);
      if (soundsOn) {
        playSound("select");
      }
    });

  const changeVolume = (level: number) =>
    run(async () => {
      if (volume) {
        volume = { ...volume, level };
      }
      volume = await setVolume(level);
    });

  const toggleSound = () =>
    run(async () => {
      volume = await setMuted(soundOn);
    });

  const activate = (item: QuickItem) => {
    switch (item) {
      case "settings":
        run(openLoungeSettings);
        break;
      case "lounge":
        run(openLounge);
        break;
      case "quit":
        openSheet(
          { kind: "confirm", title: `Quit ${gameName}?`, note: "Any unsaved progress will be lost.", action: "Quit game", back: "quit", run: quitGame },
          1,
        );
        break;
      case "mute":
        toggleSound();
        break;
      case "output":
        openSheet({ kind: "outputs" }, Math.max(0, outputs?.findIndex((output) => output.current) ?? 0));
        break;
      case "volume":
        toggleSound();
        break;
      case "sleep":
        run(() => runPower("sleep"));
        break;
      case "restart":
      case "shut-down":
        confirmPower(item);
        break;
    }
  };

  const sheetEntries = $derived.by<SheetEntry[]>(() => {
    if (sheet?.kind === "confirm") {
      const confirm = sheet;
      return [
        { id: "confirm", label: confirm.action, danger: true, run: () => run(confirm.run) },
        { id: "cancel", label: "Cancel", run: closeSheet },
      ];
    }
    if (sheet?.kind === "outputs") {
      if (outputs?.length === 0) {
        return [{ id: "none", label: "No sound output found", run: closeSheet }];
      }
      return (outputs ?? []).map((output) => ({
        id: output.id,
        label: outputTitle(output),
        value: output.current ? "In use" : undefined,
        run: () => pickOutput(output),
      }));
    }
    return [];
  });

  const handleAction = (action: PadAction) => {
    const sound = SOUNDS[action];
    if (soundsOn && sound && !busy) {
      playSound(sound);
    }
    if (sheet) {
      if (action === "up" || action === "down") {
        const step = action === "down" ? 1 : -1;
        sheetIndex = (sheetIndex + step + sheetEntries.length) % sheetEntries.length;
      } else if (action === "confirm") {
        sheetEntries[sheetIndex]?.run();
      } else if (action === "back") {
        closeSheet();
      }
      return;
    }
    if ((action === "left" || action === "right") && focused === "volume" && volume) {
      changeVolume(stepVolume(volume.level, action));
    } else if (action === "up" || action === "down" || action === "left" || action === "right") {
      spot = moveSpot(spot, action);
    } else if (action === "confirm" && focused) {
      activate(focused);
    } else if (action === "back") {
      run(resumeGame);
    }
  };

  const handleKeydown = (event: KeyboardEvent) => {
    const action = KEY_ACTIONS[event.key];
    if (action) {
      event.preventDefault();
      handleAction(action);
    }
  };

  const hints = $derived<Hint[]>(
    sheet
      ? [
          { action: "confirm", label: "Select" },
          { action: "back", label: "Back" },
        ]
      : [
          { action: "navigate", label: focused === "volume" ? "Volume" : "Move" },
          { action: "confirm", label: focused === "volume" ? (soundOn ? "Mute" : "Unmute") : "Select" },
          { action: "back", label: "Resume" },
        ],
  );

  const focusOn = (item: QuickItem) => {
    if (!sheet) {
      spot = spotOf(item);
    }
  };

  onMount(() => {
    getSession()
      .then((loaded) => {
        session = loaded;
        if (loaded?.game?.store === "steam") {
          loadAchievements(loaded.game)
            .then((result) => (achievements = result))
            .catch(() => (achievements = null));
        }
      })
      .catch((failure) => (error = String(failure)));
    getPreferences()
      .then((preferences) => (soundsOn = preferences.sounds))
      .catch(() => {});
    getAudioOutputs()
      .then((loaded) => (outputs = loaded))
      .catch(() => (outputs = []));
    getVolume()
      .then((loaded) => (volume = loaded))
      .catch(() => (volume = null));

    const clock = setInterval(() => (now = new Date()), CLOCK_REFRESH_MS);
    const stopListening = onControllerAction(handleAction);

    return () => {
      clearInterval(clock);
      stopListening.then((stop) => stop());
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} oncontextmenu={(event) => event.preventDefault()} />

{#snippet speakerIcon()}
  {#if currentOutput?.kind === "headphones" || currentOutput?.kind === "headset" || currentOutput?.kind === "controller"}
    <svg aria-hidden="true" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M4 15v-3a8 8 0 0 1 16 0v3" /><rect x="3" y="14" width="4.5" height="7" rx="1.6" /><rect x="16.5" y="14" width="4.5" height="7" rx="1.6" /></svg>
  {:else if currentOutput?.kind === "display"}
    <svg aria-hidden="true" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="12" rx="2" /><path d="M8 20h8M12 16v4" /></svg>
  {:else}
    <svg aria-hidden="true" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><rect x="6" y="3" width="12" height="18" rx="2.5" /><circle cx="12" cy="14" r="3" /><circle cx="12" cy="7.5" r="1" fill="currentColor" /></svg>
  {/if}
{/snippet}

<aside class="panel" aria-label="Quick Menu">
  <header>
    <span class="when">
      <span class="clock">{clockLabel(now)}</span>
      <span class="date">{dateLabel(now)}</span>
    </span>
    <span class="corner">
      <BatteryIndicator />
      <button
        type="button"
        class="gear"
        class:focused={focused === "settings"}
        aria-label="Lounge settings"
        disabled={busy}
        onpointerenter={() => focusOn("settings")}
        onclick={() => activate("settings")}
      >
        <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" /></svg>
      </button>
    </span>
  </header>

  {#if sheet}
    <section class="sheet" aria-label={sheet.kind === "outputs" ? "Sound output" : sheet.title}>
      <p class="sheet-title">{sheet.kind === "outputs" ? "Sound output" : sheet.title}</p>
      {#if sheet.kind === "confirm"}
        <p class="sheet-note">{sheet.note}</p>
      {:else if outputs?.length === 1}
        <p class="sheet-note">Turn on your TV or plug in headphones to see more outputs here.</p>
      {/if}
      <div class="sheet-list" role="menu">
        {#each sheetEntries as entry, index (entry.id)}
          <button
            type="button"
            role="menuitem"
            class="sheet-entry"
            class:focused={index === sheetIndex}
            class:danger={entry.danger}
            disabled={busy}
            onpointerenter={() => (sheetIndex = index)}
            onclick={entry.run}
          >
            <span>{entry.label}</span>
            {#if entry.value}
              <span class="value">{entry.value}</span>
            {/if}
          </button>
        {/each}
      </div>
    </section>
  {:else}
    <section class="game" aria-label="Now playing">
      <div class="game-head">
        <span class="art" style:background={placeholderArt(gameName)}>
          {#if artUrl}
            <img src={artUrl} alt="" onerror={() => (artBroken = true)} />
          {/if}
        </span>
        <span class="details">
          <span class="live"><span class="dot"></span>Now playing</span>
          <span class="name">{session?.game?.name ?? "Game in progress"}</span>
          {#if timeLine}
            <span class="time">{timeLine}</span>
          {/if}
        </span>
      </div>
      <div class="pair">
        <button
          type="button"
          class="pill primary"
          class:focused={focused === "lounge"}
          disabled={busy}
          onpointerenter={() => focusOn("lounge")}
          onclick={() => activate("lounge")}>Back to Lounge</button
        >
        <button
          type="button"
          class="pill danger"
          class:focused={focused === "quit"}
          disabled={busy}
          onpointerenter={() => focusOn("quit")}
          onclick={() => activate("quit")}>Quit game</button
        >
      </div>
    </section>

    <div class="tiles">
      <button
        type="button"
        class="tile"
        class:on={soundOn}
        class:focused={focused === "mute"}
        aria-pressed={soundOn}
        disabled={busy || !volume}
        onpointerenter={() => focusOn("mute")}
        onclick={() => activate("mute")}
      >
        {#if soundOn}
          <svg aria-hidden="true" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M4 9v6h4l5 4V5L8 9z" /><path d="M16.5 8.5a5 5 0 0 1 0 7" /></svg>
        {:else}
          <svg aria-hidden="true" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M4 9v6h4l5 4V5L8 9z" /><path d="M16 9l5 6M21 9l-5 6" /></svg>
        {/if}
        <span class="tile-text">
          <span class="tile-label">Sound</span>
          <span class="tile-value">{volume ? (soundOn ? "On" : "Muted") : "…"}</span>
        </span>
      </button>
      <button
        type="button"
        class="tile"
        class:focused={focused === "output"}
        disabled={busy}
        onpointerenter={() => focusOn("output")}
        onclick={() => activate("output")}
      >
        {@render speakerIcon()}
        <span class="tile-text">
          <span class="tile-label">Output</span>
          <span class="tile-value">{outputLabel}</span>
        </span>
      </button>
    </div>

    <div
      class="volume"
      class:focused={focused === "volume"}
      class:muted={!soundOn}
      role="group"
      aria-label="Volume"
      onpointerenter={() => focusOn("volume")}
    >
      <div class="volume-head">
        <label for="volume-slider">Volume</label>
        <span class="volume-value">{volume ? (soundOn ? `${volume.level} %` : "Muted") : "…"}</span>
      </div>
      <div class="volume-row">
        <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 9v6h4l5 4V5L8 9z" /></svg>
        <input
          id="volume-slider"
          type="range"
          min="0"
          max="100"
          step="1"
          value={volume?.level ?? 0}
          disabled={!volume}
          style:--level="{volume?.level ?? 0}%"
          onchange={(event) => changeVolume(Number(event.currentTarget.value))}
        />
        <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 9v6h4l5 4V5L8 9z" /><path d="M16.5 8.5a5 5 0 0 1 0 7" /><path d="M19 6a8.5 8.5 0 0 1 0 12" /></svg>
      </div>
    </div>

    {#if achievements?.status === "ready" && session}
      <div class="achievements">
        <SessionAchievements
          unlocked={achievements.unlocked}
          total={achievements.total}
          achievements={achievements.achievements}
          startedAt={session.startedAt}
        />
      </div>
    {/if}

    <div class="power">
      {#each QUICK_ROWS[4] as item (item)}
        <button
          type="button"
          class="power-button"
          class:danger={item === "shut-down"}
          class:focused={focused === item}
          disabled={busy}
          onpointerenter={() => focusOn(item)}
          onclick={() => activate(item)}
        >
          {#if item === "shut-down"}
            <svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M12 3v8" /><path d="M6.3 6.8a8 8 0 1 0 11.4 0" /></svg>
          {/if}
          {item === "sleep" ? "Sleep" : item === "restart" ? "Restart" : "Shut down"}
        </button>
      {/each}
    </div>
  {/if}

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}

  <footer>
    <Hints mode="playstation" {hints} />
  </footer>
</aside>

<style>
  :global(html),
  :global(body) {
    background: transparent;
  }

  .panel {
    position: fixed;
    inset: 24px;
    padding: 28px;
    border-radius: 30px;
    background: rgba(28, 28, 32, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    gap: 20px;
    overflow: hidden;
    animation: slide-in 0.25s var(--ease);
  }

  header {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
  }

  .when {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .clock {
    font-size: 52px;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
  }

  .date {
    font-size: 15px;
    color: var(--text-tertiary);
  }

  .corner {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  button {
    border: 0;
    font: inherit;
    cursor: pointer;
    transition:
      background 0.15s ease,
      color 0.15s ease,
      box-shadow 0.15s ease,
      transform 0.15s ease;
  }

  button:disabled {
    cursor: progress;
  }

  button:focus-visible {
    outline: none;
  }

  .focused {
    box-shadow:
      0 0 0 3px rgba(28, 28, 32, 1),
      0 0 0 5.5px var(--text);
  }

  .gear {
    width: 44px;
    height: 44px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
    display: grid;
    place-items: center;
  }

  .game {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 16px;
    border-radius: 20px;
    background: rgba(255, 255, 255, 0.06);
  }

  .game-head {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
  }

  .art {
    position: relative;
    flex: 0 0 auto;
    width: 56px;
    height: 56px;
    border-radius: 12px;
    overflow: hidden;
  }

  .art img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .details {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .live {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--live-text);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 999px;
    background: var(--live);
  }

  .name {
    font-size: 17px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .time {
    font-size: 13px;
    color: var(--text-tertiary);
  }

  .pair {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .pill {
    height: 48px;
    border-radius: 14px;
    font-size: 14px;
    font-weight: 600;
  }

  .pill.primary {
    background: var(--text);
    color: var(--bg);
  }

  .pill.danger,
  .power-button.danger {
    background: rgba(255, 69, 58, 0.16);
    border: 1px solid rgba(255, 69, 58, 0.35);
    color: #ff8a80;
  }

  .pill.danger.focused,
  .power-button.danger.focused {
    background: #ff453a;
    border-color: #ff453a;
    color: #fff;
  }

  .tiles {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
  }

  .tile {
    height: 76px;
    padding: 0 16px;
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text);
    display: flex;
    align-items: center;
    gap: 12px;
    text-align: left;
    min-width: 0;
  }

  .tile.on {
    background: var(--text);
    border-color: var(--text);
    color: var(--bg);
  }

  .tile svg {
    flex: 0 0 auto;
  }

  .tile-text {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .tile-label {
    font-size: 15px;
    font-weight: 600;
  }

  .tile-value {
    font-size: 13px;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tile.on .tile-value {
    color: #48484a;
  }

  .volume {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px 18px;
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-tertiary);
    transition: box-shadow 0.15s ease;
  }

  .volume-head {
    display: flex;
    justify-content: space-between;
    font-size: 14px;
  }

  .volume-head label {
    font-weight: 600;
    color: var(--text);
  }

  .volume-value {
    font-variant-numeric: tabular-nums;
  }

  .volume-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  input[type="range"] {
    flex: 1;
    height: 28px;
    margin: 0;
    appearance: none;
    background: transparent;
    cursor: pointer;
  }

  input[type="range"]::-webkit-slider-runnable-track {
    height: 6px;
    border-radius: 999px;
    background: linear-gradient(to right, var(--text) var(--level), rgba(255, 255, 255, 0.16) var(--level));
  }

  .volume.muted input[type="range"]::-webkit-slider-runnable-track {
    background: linear-gradient(to right, #636366 var(--level), rgba(255, 255, 255, 0.16) var(--level));
  }

  input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 20px;
    height: 20px;
    margin-top: -7px;
    border-radius: 999px;
    background: #fff;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
  }

  .achievements {
    min-height: 0;
    overflow: hidden;
  }

  .power {
    margin-top: auto;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
  }

  .power-button {
    height: 56px;
    border-radius: 16px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text);
    font-size: 14px;
    font-weight: 500;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }

  .power-button.danger {
    font-weight: 600;
  }

  .sheet {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 20px;
    border-radius: 20px;
    background: rgba(255, 255, 255, 0.06);
    animation: rise 0.2s var(--ease);
  }

  .sheet-title {
    margin: 0;
    padding: 0 4px;
    font-size: 22px;
    font-weight: 600;
  }

  .sheet-note {
    margin: 0;
    padding: 0 4px;
    font-size: 15px;
    color: var(--text-tertiary);
  }

  .sheet-list {
    margin-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .sheet-entry {
    height: 52px;
    padding: 0 18px;
    border-radius: 14px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text);
    font-size: 15px;
    font-weight: 600;
    text-align: left;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .sheet-entry.danger {
    background: rgba(255, 69, 58, 0.16);
    border-color: rgba(255, 69, 58, 0.35);
    color: #ff8a80;
  }

  .sheet-entry.danger.focused {
    background: #ff453a;
    border-color: #ff453a;
    color: #fff;
  }

  .value {
    min-width: 0;
    font-size: 15px;
    color: var(--text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .error {
    margin: 0;
    padding: 0 4px;
    font-size: 14px;
    color: #ff8a80;
  }

  footer {
    margin-top: auto;
  }

  .power + .error,
  .power ~ footer {
    margin-top: 0;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }

  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateX(24px);
    }
  }
</style>
