<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    children?: Snippet;
    class?: string;
  };

  let { children, class: className = "" }: Props = $props();
</script>

<kbd class="kbd {className}">{@render children?.()}</kbd>

<style>
  .kbd {
    display: inline-flex;
    min-width: 18px;
    align-items: center;
    justify-content: center;
    padding: 0 4px;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--fg-secondary);
    background: var(--bg-surface);
    box-shadow: 0 1px 0 rgba(0, 0, 0, 0.04);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 500;
    line-height: 16px;
  }
</style>
