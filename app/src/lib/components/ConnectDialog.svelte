<script lang="ts">
  import { onMount } from "svelte";
  import type { FriendNetwork } from "$lib/types";

  type Props = {
    network: FriendNetwork;
    connected: boolean;
    onSave: (secret: string) => Promise<void>;
    onForget: () => Promise<void>;
    onOpenPage: (page: "key" | "sign-in" | "token") => void;
    onClose: () => void;
  };

  let { network, connected, onSave, onForget, onOpenPage, onClose }: Props = $props();

  let secret = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let input = $state<HTMLInputElement>();

  const steam = $derived(network === "steam");

  const submit = async (event: SubmitEvent) => {
    event.preventDefault();
    if (busy || secret.trim() === "") {
      return;
    }
    busy = true;
    error = null;
    try {
      await onSave(secret);
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
<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="connect-title" tabindex="-1" onkeydown={handleKeydown}>
  {#if steam}
    <h2 id="connect-title">Connect Steam friends</h2>
    <ol>
      <li>
        Get a free Web API key from Steam.
        <button type="button" class="link" onclick={() => onOpenPage("key")}>Open the Steam key page</button>
      </li>
      <li>Paste it below. Make sure your friends list is public in your Steam privacy settings.</li>
    </ol>
  {:else}
    <h2 id="connect-title">Connect PlayStation friends</h2>
    <ol>
      <li>
        Sign in to playstation.com in your browser.
        <button type="button" class="link" onclick={() => onOpenPage("sign-in")}>Open playstation.com</button>
      </li>
      <li>
        Then open Sony's token page and copy the <code>npsso</code> value it shows.
        <button type="button" class="link" onclick={() => onOpenPage("token")}>Open the token page</button>
      </li>
      <li>Paste it below. Lounge uses it once to sign in, then only reads your friends list.</li>
    </ol>
  {/if}

  <form onsubmit={submit}>
    <label for="connect-secret">{steam ? "Steam Web API key" : "NPSSO token"}</label>
    <input
      id="connect-secret"
      bind:this={input}
      bind:value={secret}
      type="password"
      autocomplete="off"
      spellcheck="false"
      placeholder={steam ? "32 characters" : "64 characters"}
      aria-describedby="connect-note"
      disabled={busy}
    />
    <p id="connect-note" class="note">
      {#if steam}
        Lounge keeps this key in Windows Credential Manager, never in a file.
      {:else}
        Lounge swaps it for a sign-in that lasts about two months and keeps that in Windows Credential Manager, never in a file.
        This uses PlayStation's unofficial app API, which Sony may change at any time.
      {/if}
    </p>
    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}
    <div class="buttons">
      <button type="submit" class="primary" disabled={busy || secret.trim() === ""}>{busy ? "Checking…" : steam ? "Save" : "Connect"}</button>
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
    width: min(540px, calc(100vw - 32px));
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

  code {
    padding: 1px 5px;
    border-radius: 5px;
    background: rgba(255, 255, 255, 0.1);
    font-size: 14px;
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
    line-height: 1.4;
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
