<script lang="ts">
  import { Archive, Database, History, Settings, ShieldCheck } from "@lucide/svelte";
  import { formatCount } from "$lib/formatting";
  import { t } from "$lib/i18n.svelte";

  type Filter = "active" | "archived" | "settings";

  type Props = {
    filter: Filter;
    activeCount: number;
    archivedCount: number;
    onSelect: (filter: Filter) => void;
  };

  let { filter, activeCount, archivedCount, onSelect }: Props = $props();
</script>

<aside class="sidebar" aria-label={t("sidebar.brand")}>
  <div class="brand">
    <span class="brand-mark" aria-hidden="true">
      <History size={16} strokeWidth={2.2} />
    </span>
    <div class="brand-text">
      <strong>{t("sidebar.eyebrow")}</strong>
      <span>{t("sidebar.brand")}</span>
    </div>
  </div>

  <nav class="nav-list" aria-label={t("sidebar.brand")}>
    <button
      type="button"
      class="nav-item"
      class:active={filter === "active"}
      onclick={() => onSelect("active")}
    >
      <Database size={15} />
      <span>{t("filter.active")}</span>
      <span class="nav-count">{formatCount(activeCount)}</span>
    </button>
    <button
      type="button"
      class="nav-item"
      class:active={filter === "archived"}
      onclick={() => onSelect("archived")}
    >
      <Archive size={15} />
      <span>{t("filter.archived")}</span>
      <span class="nav-count">{formatCount(archivedCount)}</span>
    </button>
    <button
      type="button"
      class="nav-item"
      class:active={filter === "settings"}
      onclick={() => onSelect("settings")}
    >
      <Settings size={15} />
      <span>{t("filter.settings")}</span>
    </button>
  </nav>

  <div class="sidebar-spacer"></div>

  <section class="storage-note">
    <ShieldCheck size={15} />
    <div class="storage-text">
      <strong>{t("sidebar.brand")}</strong>
      <span>{t("settings.about.description")}</span>
    </div>
  </section>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 18px;
    min-height: 0;
    padding: 16px 12px;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--separator);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 4px 6px 0;
  }

  .brand-mark {
    display: grid;
    width: 28px;
    height: 28px;
    place-items: center;
    border-radius: 7px;
    color: var(--accent-fg);
    background: linear-gradient(180deg, var(--accent), var(--accent-active));
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.15) inset, var(--shadow-sm);
  }

  .brand-text {
    display: grid;
    min-width: 0;
    line-height: 1.2;
  }

  .brand-text strong {
    font-size: 13px;
    font-weight: 600;
  }

  .brand-text span {
    color: var(--fg-tertiary);
    font-size: 11px;
  }

  .nav-list {
    display: grid;
    gap: 1px;
    margin-top: 4px;
  }

  .nav-item {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    width: 100%;
    height: 28px;
    gap: 8px;
    padding: 0 8px;
    border-radius: 6px;
    color: var(--fg-secondary);
    background: transparent;
    text-align: left;
    transition:
      background-color 80ms ease,
      color 80ms ease;
  }

  .nav-item:hover {
    color: var(--fg);
    background: var(--bg-surface-hover);
  }

  .nav-item.active {
    color: var(--accent-fg);
    background: var(--accent);
  }

  .nav-item.active :global(svg),
  .nav-item.active .nav-count {
    color: var(--accent-fg);
  }

  .nav-item :global(svg) {
    color: var(--fg-tertiary);
  }

  .nav-item:hover :global(svg) {
    color: var(--fg-secondary);
  }

  .nav-item span {
    overflow: hidden;
    font-size: 13px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nav-count {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }

  .sidebar-spacer {
    flex: 1;
  }

  .storage-note {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    padding: 10px;
    border-radius: var(--radius-md);
    color: var(--fg-secondary);
    background: var(--bg-surface);
    border: 1px solid var(--separator);
  }

  .storage-note :global(svg) {
    margin-top: 1px;
    color: var(--accent);
  }

  .storage-text {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .storage-text strong {
    font-size: 12px;
    font-weight: 600;
    color: var(--fg);
  }

  .storage-text span {
    font-size: 11px;
    color: var(--fg-tertiary);
    line-height: 1.45;
  }
</style>
