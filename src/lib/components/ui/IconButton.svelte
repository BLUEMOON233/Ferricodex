<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Variant = "default" | "danger" | "filled";
  type Size = "sm" | "md" | "lg";

  type Props = HTMLButtonAttributes & {
    variant?: Variant;
    size?: Size;
    children?: Snippet;
  };

  let {
    variant = "default",
    size = "md",
    type = "button",
    children,
    class: className = "",
    ...rest
  }: Props = $props();
</script>

<button
  {...rest}
  {type}
  class="icon-btn variant-{variant} size-{size} {className}"
>
  {@render children?.()}
</button>

<style>
  .icon-btn {
    display: inline-grid;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    color: var(--fg-secondary);
    background: transparent;
    transition:
      background-color 80ms ease,
      color 80ms ease,
      border-color 80ms ease;
  }

  .size-sm {
    width: 24px;
    height: 24px;
  }

  .size-md {
    width: var(--size-icon-button);
    height: var(--size-icon-button);
  }

  .size-lg {
    width: 32px;
    height: 32px;
  }

  .variant-default:hover:not(:disabled) {
    color: var(--fg);
    background: var(--bg-surface-hover);
  }

  .variant-default:active:not(:disabled) {
    background: var(--bg-input-soft);
  }

  .variant-filled {
    color: var(--fg);
    background: var(--bg-surface);
    border-color: var(--border);
    box-shadow: var(--shadow-sm);
  }

  .variant-filled:hover:not(:disabled) {
    background: var(--bg-surface-hover);
  }

  .variant-danger {
    color: var(--danger);
  }

  .variant-danger:hover:not(:disabled) {
    background: var(--danger-soft);
  }

  .icon-btn:disabled {
    opacity: 0.4;
  }
</style>
