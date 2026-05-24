<script lang="ts">
  import { Search } from "@lucide/svelte";
  import type { HTMLInputAttributes } from "svelte/elements";
  import type { Snippet } from "svelte";

  type Props = HTMLInputAttributes & {
    value?: string;
    iconSize?: number;
    trailing?: Snippet;
  };

  let {
    value = $bindable(""),
    placeholder = "Search",
    iconSize = 14,
    trailing,
    class: className = "",
    ...rest
  }: Props = $props();

  let inputEl: HTMLInputElement | undefined = $state();

  export function focus() {
    inputEl?.focus();
    inputEl?.select();
  }
</script>

<label class="search-field {className}">
  <Search size={iconSize} class="search-icon" />
  <input
    {...rest}
    bind:this={inputEl}
    bind:value
    {placeholder}
    type="search"
  />
  {#if trailing}
    <span class="search-trailing">{@render trailing()}</span>
  {/if}
</label>

<style>
  .search-field {
    display: flex;
    align-items: center;
    gap: 7px;
    height: var(--size-control);
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--fg-tertiary);
    background: var(--bg-input);
    box-shadow: var(--shadow-sm);
    transition:
      border-color 80ms ease,
      box-shadow 80ms ease;
  }

  .search-field:hover {
    border-color: var(--border-strong);
  }

  .search-field:focus-within {
    border-color: var(--accent);
    outline: 2px solid var(--ring);
    outline-offset: -1px;
  }

  .search-field :global(.search-icon) {
    flex: 0 0 auto;
    color: var(--fg-tertiary);
  }

  .search-field input {
    width: 100%;
    min-width: 0;
    height: 100%;
    border: 0;
    color: var(--fg);
    background: transparent;
    font-size: 13px;
    outline: 0;
  }

  .search-field input::placeholder {
    color: var(--fg-tertiary);
  }

  .search-field input::-webkit-search-cancel-button {
    appearance: none;
  }

  .search-trailing {
    display: flex;
    flex: 0 0 auto;
    align-items: center;
    color: var(--fg-tertiary);
  }
</style>
