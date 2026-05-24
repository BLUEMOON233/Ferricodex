<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  type Props = HTMLInputAttributes & {
    checked?: boolean;
    indeterminate?: boolean;
  };

  let {
    checked = $bindable(false),
    indeterminate = false,
    class: className = "",
    ...rest
  }: Props = $props();

  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (inputEl) {
      inputEl.indeterminate = indeterminate;
    }
  });
</script>

<span class="checkbox {className}">
  <input
    {...rest}
    bind:this={inputEl}
    bind:checked
    type="checkbox"
  />
  <span class="checkbox-mark" aria-hidden="true">
    {#if indeterminate}
      <svg viewBox="0 0 12 12" width="10" height="10">
        <path d="M2.5 6h7" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" />
      </svg>
    {:else}
      <svg viewBox="0 0 12 12" width="10" height="10">
        <path
          d="M2.5 6.3 5 8.8l4.5-5.6"
          fill="none"
          stroke="currentColor"
          stroke-width="1.7"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    {/if}
  </span>
</span>

<style>
  .checkbox {
    position: relative;
    display: inline-grid;
    width: 16px;
    height: 16px;
    place-items: center;
  }

  .checkbox input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    opacity: 0;
    cursor: pointer;
  }

  .checkbox-mark {
    display: grid;
    width: 16px;
    height: 16px;
    place-items: center;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-xs);
    color: transparent;
    background: var(--bg-input);
    box-shadow: var(--shadow-sm);
    transition:
      background-color 80ms ease,
      border-color 80ms ease,
      color 80ms ease;
  }

  .checkbox input:hover + .checkbox-mark {
    border-color: var(--accent);
  }

  .checkbox input:focus-visible + .checkbox-mark {
    outline: 2px solid var(--ring);
    outline-offset: 1px;
  }

  .checkbox input:checked + .checkbox-mark,
  .checkbox input:indeterminate + .checkbox-mark {
    color: var(--accent-fg);
    background: var(--accent);
    border-color: var(--accent);
  }

  .checkbox input:disabled + .checkbox-mark {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
