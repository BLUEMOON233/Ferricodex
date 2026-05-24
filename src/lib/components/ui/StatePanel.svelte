<script lang="ts">
  import type { Snippet } from "svelte";

  type Variant = "default" | "error" | "muted";
  type Tone = "neutral" | "accent" | "warning";

  type Props = {
    variant?: Variant;
    tone?: Tone;
    icon?: Snippet;
    children?: Snippet;
    class?: string;
  };

  let {
    variant = "default",
    tone = "neutral",
    icon,
    children,
    class: className = "",
  }: Props = $props();
</script>

<div class="state-panel variant-{variant} tone-{tone} {className}">
  {#if icon}
    <span class="state-icon">{@render icon()}</span>
  {/if}
  <div class="state-body">
    {@render children?.()}
  </div>
</div>

<style>
  .state-panel {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--fg-secondary);
    background: var(--bg-surface);
    font-size: 13px;
    line-height: 1.5;
  }

  .variant-error {
    color: var(--danger);
    background: var(--danger-soft);
    border-color: transparent;
  }

  .variant-muted {
    background: var(--bg-input-soft);
    border-color: transparent;
  }

  .tone-accent {
    color: var(--accent);
    background: var(--accent-soft);
    border-color: transparent;
  }

  .tone-warning {
    color: var(--warning);
    background: var(--warning-soft);
    border-color: transparent;
  }

  .state-icon {
    display: flex;
    flex: 0 0 auto;
    align-items: center;
    margin-top: 1px;
  }

  .state-body {
    flex: 1;
    min-width: 0;
  }
</style>
