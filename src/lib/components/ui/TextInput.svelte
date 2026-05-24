<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  type Props = HTMLInputAttributes & {
    value?: string;
  };

  let {
    value = $bindable(""),
    type = "text",
    class: className = "",
    ...rest
  }: Props = $props();
</script>

<input
  {...rest}
  {type}
  bind:value
  class="text-input {className}"
/>

<style>
  .text-input {
    width: 100%;
    height: var(--size-control);
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--fg);
    background: var(--bg-input);
    box-shadow: var(--shadow-sm);
    font-size: 13px;
    transition:
      border-color 80ms ease,
      box-shadow 80ms ease;
  }

  .text-input::placeholder {
    color: var(--fg-tertiary);
  }

  .text-input:hover {
    border-color: var(--border-strong);
  }

  .text-input:focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: -1px;
    border-color: var(--accent);
  }

  .text-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
