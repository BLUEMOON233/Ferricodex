<script lang="ts">
  import { Archive } from "@lucide/svelte";
  import { Checkbox } from "$lib/components/ui";
  import { t } from "$lib/i18n.svelte";
  import type { Session } from "$lib/codex";

  type Props = {
    session: Session;
    active: boolean;
    checked: boolean;
    onSelect: (session: Session) => void;
    onToggle: (session: Session, checked: boolean) => void;
  };

  let { session, active, checked, onSelect, onToggle }: Props = $props();
</script>

<div class="session-row" class:active class:checked>
  <span class="row-check">
    <Checkbox
      checked={checked}
      aria-label={t("session.checkboxLabel", { title: session.title })}
      onchange={(event) => onToggle(session, (event.currentTarget as HTMLInputElement).checked)}
    />
  </span>

  <button
    type="button"
    class="row-button"
    onclick={() => onSelect(session)}
  >
    <span class="row-main">
      <span class="row-status" aria-hidden="true">
        {#if session.archived}
          <Archive size={11} />
        {:else}
          <span class="dot"></span>
        {/if}
      </span>
      <span class="row-title" title={session.title}>{session.title}</span>
      <span class="row-time">{session.updatedAtLabel}</span>
    </span>
    <span class="row-preview" title={session.preview}>{session.preview}</span>
  </button>
</div>

<style>
  .session-row {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: stretch;
    gap: 0;
    padding: 0 6px;
    border-radius: var(--radius-sm);
    color: inherit;
    transition: background-color 60ms ease;
  }

  .session-row:hover {
    background: var(--selection-inactive);
  }

  .session-row.active {
    color: var(--selection-fg);
    background: var(--accent);
  }

  .session-row.active:hover {
    background: var(--accent-hover);
  }

  .row-check {
    display: grid;
    place-items: center;
    width: 22px;
    cursor: default;
  }

  .row-button {
    display: grid;
    gap: 2px;
    min-width: 0;
    padding: 6px 6px 7px;
    border: 0;
    background: transparent;
    text-align: left;
    color: inherit;
  }

  .row-main {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 7px;
    min-width: 0;
  }

  .row-status {
    display: grid;
    place-items: center;
    width: 12px;
    color: var(--fg-tertiary);
  }

  .session-row.active .row-status {
    color: rgba(255, 255, 255, 0.9);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--success);
  }

  .session-row.active .dot {
    background: var(--accent-fg);
  }

  .row-title {
    overflow: hidden;
    font-size: 13px;
    font-weight: 500;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-time {
    flex: 0 0 auto;
    color: var(--fg-tertiary);
    font-size: 11px;
    line-height: 16px;
    font-variant-numeric: tabular-nums;
  }

  .session-row.active .row-time {
    color: rgba(255, 255, 255, 0.85);
  }

  .row-preview {
    overflow: hidden;
    color: var(--fg-tertiary);
    font-size: 12px;
    line-height: 17px;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding-left: 19px;
  }

  .session-row.active .row-preview {
    color: rgba(255, 255, 255, 0.85);
  }
</style>
