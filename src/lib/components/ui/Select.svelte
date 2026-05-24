<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLSelectAttributes } from "svelte/elements";

  type Props = HTMLSelectAttributes & {
    value?: string;
    children?: Snippet;
  };

  let { value = $bindable(""), children, class: className = "", ...rest }: Props = $props();
</script>

<div class="select-wrap {className}">
  <select {...rest} bind:value class="select">
    {@render children?.()}
  </select>
  <svg
    class="select-caret"
    width="10"
    height="10"
    viewBox="0 0 12 12"
    aria-hidden="true"
  >
    <path
      d="M3.2 4.6 6 7.4l2.8-2.8"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</div>

<style>
  .select-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
  }

  .select {
    width: 100%;
    height: var(--size-control);
    padding: 0 28px 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--fg);
    background: var(--bg-input);
    box-shadow: var(--shadow-sm);
    font-size: 13px;
    appearance: none;
    cursor: pointer;
    transition:
      border-color 80ms ease,
      box-shadow 80ms ease;
  }

  .select:hover {
    border-color: var(--border-strong);
  }

  .select:focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: -1px;
    border-color: var(--accent);
  }

  .select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .select-caret {
    position: absolute;
    right: 9px;
    pointer-events: none;
    color: var(--fg-tertiary);
  }
</style>
