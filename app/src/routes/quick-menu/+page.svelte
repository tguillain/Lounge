<script lang="ts">
  import { onMount } from "svelte";
  import Hints from "$lib/components/Hints.svelte";
  import { getSession, imageUrl, onControllerAction, openLounge, quitGame, resumeGame } from "$lib/api";
  import { clockLabel, dateLabel, placeholderArt, playTimeLabel } from "$lib/format";
  import type { PadAction } from "$lib/input/gamepad-mapping";
  import type { Session } from "$lib/types";

  type Entry = { id: string; label: string; danger?: boolean; run: () => void };

  const CLOCK_REFRESH_MS = 15_000;
  const KEY_ACTIONS: Record<string, PadAction> = {
    ArrowUp: "up",
    ArrowDown: "down",
    Enter: "confirm",
    Escape: "back",
  };

  let session = $state<Session | null>(null);
  let now = $state(new Date());
  let focusedIndex = $state(0);
  let confirmingQuit = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);
  let headerBroken = $state(false);

  const gameName = $derived(session?.game?.name ?? "your game");
  const headerUrl = $derived(headerBroken ? null : imageUrl(session?.game?.images.header ?? null));

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

  const askToQuit = () => {
    confirmingQuit = true;
    focusedIndex = 1;
  };

  const cancelQuit = () => {
    confirmingQuit = false;
    focusedIndex = 2;
  };

  const entries = $derived<Entry[]>(
    confirmingQuit
      ? [
          { id: "confirm-quit", label: "Quit game", danger: true, run: () => run(quitGame) },
          { id: "cancel", label: "Cancel", run: cancelQuit },
        ]
      : [
          { id: "resume", label: "Resume game", run: () => run(resumeGame) },
          { id: "lounge", label: "Back to Lounge", run: () => run(openLounge) },
          { id: "quit", label: "Quit game", danger: true, run: askToQuit },
        ],
  );

  const handleAction = (action: PadAction) => {
    if (action === "up" || action === "down") {
      const step = action === "down" ? 1 : -1;
      focusedIndex = (focusedIndex + step + entries.length) % entries.length;
    } else if (action === "confirm") {
      entries[focusedIndex]?.run();
    } else if (action === "back") {
      if (confirmingQuit) {
        cancelQuit();
      } else {
        run(resumeGame);
      }
    }
  };

  const handleKeydown = (event: KeyboardEvent) => {
    const action = KEY_ACTIONS[event.key];
    if (action) {
      event.preventDefault();
      handleAction(action);
    }
  };

  onMount(() => {
    getSession()
      .then((loaded) => (session = loaded))
      .catch((failure) => (error = String(failure)));

    const clock = setInterval(() => (now = new Date()), CLOCK_REFRESH_MS);
    const stopListening = onControllerAction(handleAction);

    return () => {
      clearInterval(clock);
      stopListening.then((stop) => stop());
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} oncontextmenu={(event) => event.preventDefault()} />

<aside class="panel" aria-label="Quick Menu">
  <header>
    <span class="clock">{clockLabel(now)}</span>
    <span class="date">{dateLabel(now)}</span>
  </header>

  <section class="now-playing" aria-label="Now playing">
    <span class="art" style:background={placeholderArt(gameName)}>
      {#if headerUrl}
        <img src={headerUrl} alt="" onerror={() => (headerBroken = true)} />
      {/if}
    </span>
    <span class="details">
      <span class="live"><span class="dot"></span>Now playing</span>
      <span class="name">{session?.game?.name ?? "Game in progress"}</span>
      {#if session}
        <span class="time">{playTimeLabel(session.startedAt, now.getTime())}</span>
      {/if}
    </span>
  </section>

  {#if confirmingQuit}
    <div class="confirm" role="alert">
      <p class="confirm-title">Quit {gameName}?</p>
      <p class="confirm-body">Any unsaved progress will be lost.</p>
    </div>
  {/if}

  <div class="menu" role="menu">
    {#each entries as entry, index (entry.id)}
      <button
        type="button"
        role="menuitem"
        class:focused={index === focusedIndex}
        class:danger={entry.danger}
        disabled={busy}
        onpointerenter={() => (focusedIndex = index)}
        onclick={entry.run}
      >
        {entry.label}
      </button>
    {/each}
  </div>

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}

  <footer>
    <Hints
      mode="playstation"
      hints={[
        { action: "confirm", label: "Select" },
        { action: "back", label: confirmingQuit ? "Cancel" : "Resume" },
      ]}
    />
  </footer>
</aside>

<style>
  .panel {
    height: 100vh;
    padding: 32px 28px 28px;
    background: #141418;
    border-left: 1px solid rgba(255, 255, 255, 0.08);
    display: flex;
    flex-direction: column;
    gap: 24px;
    animation: slide-in 0.25s var(--ease);
  }

  header {
    display: flex;
    flex-direction: column;
    gap: 4px;
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

  .now-playing {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    border-radius: 20px;
    background: rgba(255, 255, 255, 0.06);
  }

  .art {
    position: relative;
    flex: 0 0 auto;
    width: 120px;
    aspect-ratio: 460 / 215;
    border-radius: 10px;
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

  .confirm {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 0 4px;
  }

  .confirm-title {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
  }

  .confirm-body {
    margin: 0;
    font-size: 15px;
    color: var(--text-tertiary);
  }

  .menu {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  button {
    height: 56px;
    padding: 0 18px;
    border: 0;
    border-radius: 14px;
    background: transparent;
    font-size: 17px;
    font-weight: 500;
    text-align: left;
    cursor: pointer;
    transition:
      background 0.15s ease,
      color 0.15s ease;
  }

  button.danger {
    color: #ff8a80;
  }

  button.focused {
    background: var(--text);
    color: var(--bg);
    font-weight: 600;
  }

  button.danger.focused {
    background: #ff453a;
    color: #fff;
  }

  button:disabled {
    cursor: progress;
    opacity: 0.6;
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

  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateX(24px);
    }
  }
</style>
