<script lang="ts">
  import { Trash2, X } from "@lucide/svelte";
  import { Button } from "$lib/components/ui";
  import { t } from "$lib/i18n.svelte";

  type Props = {
    selectedCount: number;
    visibleCount: number;
    canSelectVisible: boolean;
    busy: boolean;
    onSelectVisible: () => void;
    onClear: () => void;
    onTrash: () => void;
  };

  let {
    selectedCount,
    visibleCount,
    canSelectVisible,
    busy,
    onSelectVisible,
    onClear,
    onTrash,
  }: Props = $props();

  const visible = $derived(selectedCount > 0);
</script>

{#if visible}
  <div class="bulk-toolbar" role="toolbar" aria-label={t("bulk.selected", { count: selectedCount })}>
    <div class="bulk-top-row">
      <span>{t("bulk.selected", { count: selectedCount })}</span>
      <Button variant="ghost" size="sm" onclick={onClear}>
        <X size={12} />
        {t("bulk.clearSelection")}
      </Button>
    </div>

    <div class="bulk-actions">
      <Button
        variant="ghost"
        size="sm"
        disabled={!canSelectVisible}
        onclick={onSelectVisible}
      >
        {t("bulk.selectVisible")}
      </Button>
      <Button variant="danger" size="sm" loading={busy} onclick={onTrash}>
        <Trash2 size={12} />
        {t("bulk.moveToTrash")}
      </Button>
    </div>
  </div>
{/if}

<style>
  .bulk-toolbar {
    display: grid;
    gap: 6px;
    margin-bottom: 8px;
    padding: 8px 10px;
    border-radius: var(--radius-md);
    background: var(--accent-soft);
    color: var(--fg);
  }

  .bulk-top-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    color: var(--fg);
    font-size: 12px;
    font-weight: 500;
    line-height: 1.4;
  }

  .bulk-actions {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 6px;
  }
</style>
