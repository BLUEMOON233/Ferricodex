<script lang="ts">
  import { SearchField, Select, StatePanel, Skeleton, Badge, Button } from "$lib/components/ui";
  import { formatCount } from "$lib/formatting";
  import { t } from "$lib/i18n.svelte";
  import TranscriptMessage from "./TranscriptMessage.svelte";
  import type { CodexTranscript, CodexTranscriptMessage } from "$lib/codex";

  type Props = {
    transcript: CodexTranscript | null;
    error: string;
    loading: boolean;
    query: string;
    roleFilter: string;
    roleOptions: string[];
    filteredMessages: CodexTranscriptMessage[];
    filterActive: boolean;
    onQueryChange: (value: string) => void;
    onRoleChange: (value: string) => void;
    onClearFilters: () => void;
  };

  let {
    transcript,
    error,
    loading,
    query,
    roleFilter,
    roleOptions,
    filteredMessages,
    filterActive,
    onQueryChange,
    onRoleChange,
    onClearFilters,
  }: Props = $props();
</script>

<section class="transcript">
  <header class="head">
    <h3>{t("transcript.title")}</h3>
    <div class="status">
      {#if loading}
        <Badge variant="muted">{t("transcript.badge.loading")}</Badge>
      {:else if transcript?.truncated}
        <Badge variant="warning">{t("transcript.badge.truncated")}</Badge>
      {:else if transcript}
        <Badge variant="muted">
          {t("transcript.badge.messages", { count: transcript.messages.length })}
        </Badge>
      {:else}
        <Badge variant="muted">{t("transcript.badge.onDemand")}</Badge>
      {/if}
    </div>
  </header>

  {#if loading && !transcript}
    <div class="skeleton-stack">
      <Skeleton height="56px" radius="var(--radius-md)" />
      <Skeleton height="56px" radius="var(--radius-md)" />
      <Skeleton height="56px" radius="var(--radius-md)" />
    </div>
  {:else if error}
    <StatePanel variant="error">{error}</StatePanel>
  {:else if transcript}
    {#if !transcript.exists}
      <StatePanel variant="muted">{t("transcript.notFound", { path: transcript.path })}</StatePanel>
    {:else}
      <div class="filters">
        <SearchField
          value={query}
          placeholder={t("transcript.searchPlaceholder")}
          oninput={(event) => onQueryChange((event.currentTarget as HTMLInputElement).value)}
        />
        <label class="role-filter">
          <span>{t("transcript.role")}</span>
          <Select
            value={roleFilter}
            onchange={(event) =>
              onRoleChange((event.currentTarget as HTMLSelectElement).value)}
          >
            <option value="all">{t("transcript.role.all")}</option>
            {#each roleOptions as role}
              <option value={role}>{role}</option>
            {/each}
          </Select>
        </label>
      </div>

      <div class="meta-row">
        {#if filterActive}
          <span>
            {t("transcript.showingFilteredOf", {
              shown: formatCount(filteredMessages.length),
              total: formatCount(transcript.messages.length),
            })}
          </span>
          <Button variant="ghost" size="sm" onclick={onClearFilters}>
            {t("transcript.clearFilters")}
          </Button>
        {:else}
          <span class="dim">
            {t("transcript.showingAll", { count: transcript.messages.length })}
            {#if transcript.invalidLineCount > 0}
              · {t("transcript.invalidLines", { count: transcript.invalidLineCount })}
            {/if}
          </span>
        {/if}
      </div>

      {#if transcript.truncated}
        <p class="hint">{t("transcript.partialNotice")}</p>
      {/if}

      {#if transcript.messages.length === 0}
        <StatePanel variant="muted">{t("transcript.empty")}</StatePanel>
      {:else if filteredMessages.length === 0}
        <StatePanel variant="muted">{t("transcript.filterEmpty")}</StatePanel>
      {:else}
        <div class="message-list">
          {#each filteredMessages as message (message.lineNumber)}
            <TranscriptMessage {message} />
          {/each}
        </div>
      {/if}
    {/if}
  {:else}
    <StatePanel variant="muted">{t("transcript.notLoaded")}</StatePanel>
  {/if}
</section>

<style>
  .transcript {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 10px;
    min-width: 0;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    min-width: 0;
  }

  .head h3 {
    color: var(--fg);
    font-size: 13px;
    font-weight: 600;
    line-height: 18px;
  }

  .filters {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(120px, 160px);
    gap: 8px;
  }

  .role-filter {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    gap: 6px;
    color: var(--fg-tertiary);
    font-size: 12px;
  }

  .meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    color: var(--fg-secondary);
    font-size: 11px;
  }

  .dim {
    color: var(--fg-tertiary);
  }

  .hint {
    color: var(--fg-tertiary);
    font-size: 11px;
    line-height: 1.5;
  }

  .message-list {
    display: grid;
    gap: 6px;
  }

  .skeleton-stack {
    display: grid;
    gap: 6px;
  }
</style>
