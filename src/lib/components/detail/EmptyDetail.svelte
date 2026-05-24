<script lang="ts">
  import { Inbox } from "@lucide/svelte";
  import { t } from "$lib/i18n.svelte";

  type Props = {
    title?: string;
    description?: string;
  };

  let { title, description }: Props = $props();

  const resolvedTitle = $derived(title ?? t("detail.empty.title"));
  const resolvedDescription = $derived(description ?? t("detail.empty.hint"));
</script>

<div class="empty">
  <span class="empty-icon" aria-hidden="true">
    <Inbox size={28} strokeWidth={1.6} />
  </span>
  <strong>{resolvedTitle}</strong>
  <span>{resolvedDescription}</span>
</div>

<style>
  .empty {
    display: grid;
    place-items: center;
    gap: 8px;
    width: 100%;
    margin: auto;
    padding: 48px 24px;
    text-align: center;
    color: var(--fg-tertiary);
  }

  .empty-icon {
    display: grid;
    width: 56px;
    height: 56px;
    place-items: center;
    border-radius: 999px;
    background: var(--bg-input-soft);
    color: var(--fg-tertiary);
  }

  .empty strong {
    color: var(--fg-secondary);
    font-size: 14px;
    font-weight: 600;
  }

  .empty span {
    max-width: 320px;
    font-size: 12px;
    line-height: 1.5;
  }
</style>
