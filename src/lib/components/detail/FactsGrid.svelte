<script lang="ts">
  type Fact = {
    label: string;
    value: string;
    title?: string;
    mono?: boolean;
    wide?: boolean;
  };

  type Props = {
    facts: Fact[];
    columns?: 2 | 3;
  };

  let { facts, columns = 2 }: Props = $props();
</script>

<dl class="facts cols-{columns}">
  {#each facts as fact (fact.label)}
    <div class="fact" class:wide={fact.wide}>
      <dt>{fact.label}</dt>
      <dd class:mono={fact.mono} title={fact.title ?? fact.value}>{fact.value}</dd>
    </div>
  {/each}
</dl>

<style>
  .facts {
    display: grid;
    gap: 1px;
    margin: 0;
    padding: 1px;
    border-radius: var(--radius-md);
    background: var(--separator);
    overflow: hidden;
    border: 1px solid var(--separator);
  }

  .cols-2 {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .cols-3 {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .fact {
    display: grid;
    gap: 3px;
    min-width: 0;
    padding: 9px 11px;
    background: var(--bg-surface);
  }

  .fact.wide {
    grid-column: 1 / -1;
  }

  dt {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-weight: 500;
    line-height: 15px;
    letter-spacing: 0.005em;
  }

  dd {
    overflow: hidden;
    margin: 0;
    color: var(--fg);
    font-size: 13px;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  dd.mono {
    font-family: var(--font-mono);
    font-size: 12px;
  }
</style>
