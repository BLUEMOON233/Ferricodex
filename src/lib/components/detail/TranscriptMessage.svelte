<script lang="ts">
  import { Bot, Cog, Terminal, User, Wrench } from "@lucide/svelte";
  import type { CodexTranscriptMessage } from "$lib/codex";

  type Props = {
    message: CodexTranscriptMessage;
  };

  let { message }: Props = $props();

  function iconFor(role: string) {
    const r = role.toLowerCase();
    if (r === "user") return User;
    if (r === "assistant" || r === "model") return Bot;
    if (r === "tool" || r === "function") return Wrench;
    if (r === "system") return Cog;
    return Terminal;
  }

  const RoleIcon = $derived(iconFor(message.role));

  const variantClass = $derived.by(() => {
    const r = message.role.toLowerCase();
    if (r === "user") return "user";
    if (r === "assistant" || r === "model") return "assistant";
    if (r === "tool" || r === "function") return "tool";
    if (r === "system") return "system";
    return "other";
  });
</script>

<article class="message variant-{variantClass}">
  <header class="meta">
    <span class="role">
      <RoleIcon size={11} />
      <span>{message.role}</span>
    </span>
    <span class="meta-right">
      <span class="line">L{message.lineNumber}</span>
      {#if message.timestamp}
        <time datetime={message.timestamp}>{message.timestamp}</time>
      {/if}
    </span>
  </header>
  <pre class="content">{message.text}</pre>
</article>

<style>
  .message {
    display: grid;
    gap: 6px;
    min-width: 0;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    border: 1px solid var(--separator);
  }

  .variant-user {
    background: var(--accent-soft);
    border-color: transparent;
  }

  .variant-tool {
    background: var(--bg-input-soft);
  }

  .variant-system {
    background: var(--warning-soft);
    border-color: transparent;
  }

  .meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 11px;
    line-height: 1.4;
  }

  .role {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--fg-secondary);
    font-weight: 600;
    text-transform: capitalize;
  }

  .variant-user .role {
    color: var(--accent);
  }

  .variant-system .role {
    color: var(--warning);
  }

  .meta-right {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--fg-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .content {
    overflow-x: auto;
    margin: 0;
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
