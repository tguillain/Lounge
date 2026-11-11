<script lang="ts">
  import { onMount } from "svelte";

  type Props = {
    connected: boolean;
    onSave: (key: string) => Promise<void>;
    onForget: () => Promise<void>;
    onOpenKeyPage: () => void;
    onClose: () => void;
  };

  let { connected, onSave, onForget, onOpenKeyPage, onClose }: Props = $props();

  let key = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let input = $state<HTMLInputElement>();

  const submit = async (event: SubmitEvent) => {
    event.preventDefault();
    if (busy || key.trim() === "") {
      return;
    }
    busy = true;
    error = null;
    try {
      await onSave(key);
    } catch (failure) {
      error = String(failure);
    } finally {
      busy = false;
    }
  };

  const forget = async () => {
    busy = true;
    try {
      await onForget();
    } finally {
      busy = false;
    }
  };

  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
  };

  onMount(() => input?.focus());
</script>

<div class="backdrop" role="presentation" onclick={onClose}></div>
<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="steam-key-title" tabindex="-1" onkeydown={handleKeydown}>
  <h2 id="steam-key-title">Connect Steam friends</h2>
  <ol>
    <li>
      Get a free Web API key from Steam.
      <button type="button" class="link" onclick={onOpenKeyPage}>Open the Steam key page</button>
    </li>
    <li>Paste it below. Make sure your friends list is public in your Steam privacy settings.</li>
  </ol>

  <form onsubmit={submit}>
    <label for="steam-key">Steam Web API key</label>
    <input
      id="steam-key"
      bind:this={input}
      bind:value={key}
      type="password"
      autocomplete="off"
      spellcheck="false"
      placeholder="32 characters"
      aria-describedby="steam-key-note"
      disabled={busy}
    />
    <p id="steam-key-note" class="note">Lounge keeps this key in Windows Credential Manager, never in a file.</p>
    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}
    <div class="buttons">
      <button type="submit" class="primary" disabled={busy || key.trim() === ""}>{busy ? "Checking…" : "Save"}</button>
      <button type="button" class="secondary" onclick={onClose}>Cancel</button>
      {#if connected}
        <button type="button" class="danger" onclick={forget} disabled={busy}>Disconnect</button>
      {/if}
    </div>
  </form>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 30;
    background: rgba(10, 10, 12, 0.6);
  }

  .dialog {
    position: fixed;
    z-index: 31;
    left: 50%;
    top: 50%;
    width: min(520px, calc(100vw - 32px));
    padding: 28px;
    border-radius: 24px;
    background: #1c1c20;
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.55);
    transform: translate(-50%, -50%);
    user-select: text;
  }

  h2 {
    margin: 0 0 14px;
    font-size: 22px;
    font-weight: 700;
  }

  ol {
    margin: 0 0 20px;
    padding-left: 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    font-size: 15px;
    line-height: 1.45;
    color: var(--text-secondary);
  }

  .link {
    display: block;
    margin-top: 4px;
    padding: 0;
    border: 0;
    background: none;
    color: #64d2ff;
    font-size: 15px;
    font-weight: 600;
    text-decoration: underline;
    cursor: pointer;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  label {
    font-size: 14px;
    font-weight: 600;
  }

  input {
    height: 48px;
    padding: 0 14px;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: #0a0a0c;
    color: var(--text);
    font: inherit;
    font-size: 15px;
    letter-spacing: 0.04em;
  }

  input:focus {
    outline: 2px solid var(--text);
    outline-offset: 2px;
  }

  .note {
    margin: 0;
    font-size: 13px;
    color: var(--text-tertiary);
  }

  .error {
    margin: 4px 0 0;
    font-size: 14px;
    color: #ff8a80;
  }

  .buttons {
    display: flex;
    gap: 10px;
    margin-top: 12px;
  }

  .buttons button {
    height: 44px;
    padding: 0 20px;
    border-radius: 999px;
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
  }

  .primary {
    border: 0;
    background: var(--text);
    color: var(--bg);
  }

  .secondary {
    border: 1px solid rgba(255, 255, 255, 0.16);
    background: rgba(255, 255, 255, 0.08);
  }

  .danger {
    margin-left: auto;
    border: 1px solid rgba(255, 69, 58, 0.35);
    background: rgba(255, 69, 58, 0.16);
    color: #ff8a80;
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
