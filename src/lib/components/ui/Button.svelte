<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Variant = "primary" | "secondary" | "ghost" | "danger";
  type Size = "sm" | "md";

  type Props = HTMLButtonAttributes & {
    variant?: Variant;
    size?: Size;
    fullWidth?: boolean;
    loading?: boolean;
    children?: Snippet;
  };

  let {
    variant = "secondary",
    size = "md",
    fullWidth = false,
    loading = false,
    type = "button",
    disabled,
    children,
    class: className = "",
    ...rest
  }: Props = $props();
</script>

<button
  {...rest}
  {type}
  class="btn variant-{variant} size-{size} {fullWidth ? 'full' : ''} {className}"
  disabled={disabled || loading}
  data-loading={loading || undefined}
>
  {#if loading}
    <span class="spinner" aria-hidden="true"></span>
  {/if}
  {@render children?.()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 0 12px;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    color: inherit;
    background: transparent;
    font-size: 13px;
    font-weight: 500;
    line-height: 1;
    white-space: nowrap;
    transition:
      background-color 80ms ease,
      border-color 80ms ease,
      color 80ms ease,
      box-shadow 80ms ease,
      transform 60ms ease;
  }

  .btn.full {
    width: 100%;
  }

  .size-sm {
    height: 24px;
    padding: 0 9px;
    font-size: 12px;
  }

  .size-md {
    height: var(--size-control);
  }

  .variant-primary {
    color: var(--accent-fg);
    background: var(--accent);
    box-shadow: 0 1px 0 rgba(0, 0, 0, 0.04), inset 0 1px 0 rgba(255, 255, 255, 0.18);
  }

  .variant-primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .variant-primary:active:not(:disabled) {
    background: var(--accent-active);
    transform: translateY(0.5px);
  }

  .variant-secondary {
    color: var(--fg);
    background: var(--bg-surface);
    border-color: var(--border);
    box-shadow: var(--shadow-sm);
  }

  .variant-secondary:hover:not(:disabled) {
    background: var(--bg-surface-hover);
  }

  .variant-secondary:active:not(:disabled) {
    background: var(--bg-input-soft);
    transform: translateY(0.5px);
  }

  .variant-ghost {
    color: var(--fg-secondary);
    background: transparent;
  }

  .variant-ghost:hover:not(:disabled) {
    color: var(--fg);
    background: var(--bg-surface-hover);
  }

  .variant-danger {
    color: var(--fg-on-accent);
    background: var(--danger);
    box-shadow: 0 1px 0 rgba(0, 0, 0, 0.04), inset 0 1px 0 rgba(255, 255, 255, 0.18);
  }

  .variant-danger:hover:not(:disabled) {
    background: var(--danger-hover);
  }

  .variant-danger:active:not(:disabled) {
    transform: translateY(0.5px);
  }

  .btn:disabled {
    opacity: 0.5;
    box-shadow: none;
  }

  .btn[data-loading] {
    cursor: progress;
  }

  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    border-radius: 999px;
    opacity: 0.85;
    animation: spin 720ms linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
