<script lang="ts">
  import type { Snippet } from "svelte";

  type Variant = "neutral" | "accent" | "success" | "warning" | "danger" | "muted";

  type Props = {
    variant?: Variant;
    children?: Snippet;
    class?: string;
  };

  let { variant = "neutral", children, class: className = "" }: Props = $props();
</script>

<span class="badge variant-{variant} {className}">
  {@render children?.()}
</span>

<style>
  .badge {
    display: inline-flex;
    flex: 0 0 auto;
    align-items: center;
    gap: 4px;
    padding: 1px 7px;
    border: 1px solid transparent;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 500;
    line-height: 16px;
    letter-spacing: 0.005em;
    white-space: nowrap;
  }

  .variant-neutral {
    color: var(--fg-secondary);
    background: var(--bg-input-soft);
    border-color: var(--border);
  }

  .variant-muted {
    color: var(--fg-tertiary);
    background: transparent;
    border-color: var(--border);
  }

  .variant-accent {
    color: var(--accent);
    background: var(--accent-soft);
  }

  .variant-success {
    color: var(--success);
    background: var(--success-soft);
  }

  .variant-warning {
    color: var(--warning);
    background: var(--warning-soft);
  }

  .variant-danger {
    color: var(--danger);
    background: var(--danger-soft);
  }
</style>
